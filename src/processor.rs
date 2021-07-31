use notify_rust::Notification;
use std::collections::VecDeque;
use std::sync::mpsc;
use crate::ui;

const CIRC_BUF_SIZE:usize = 3600;

#[derive(Debug, std::cmp::PartialEq)]
pub enum KeyKind {
    Single,
    Hold,
}

#[derive(Debug)]
pub struct Keydata {
    symbol: String,
    kind: KeyKind,
    timestamp: u64,
}

/// Struct used for sending messages to the ui
#[derive(Debug)]
pub struct UiData {
    pub graph_data: Vec<(f64, f64)>,
}

impl Keydata {
    pub fn new(symbol: String, kind: KeyKind) -> Keydata { Keydata{symbol, kind, timestamp: 0} }

    pub fn set_timestamp(&mut self, timestamp: u64) { self.timestamp = timestamp; }

    pub fn get_timestamp(&self) -> u64 { self.timestamp }

    pub fn get_kind(&self) -> &KeyKind { &self.kind }

    pub fn get_symbol(&self) -> &String { &self.symbol }
}

pub struct Processor {
    timer: u64,
    keys_total: u64,
    tx: mpsc::Sender<UiData>,
    current_keys: Option<Vec<Keydata>>,
    wpm_circ_buf: VecDeque<u32>,
}

impl Processor {

    pub fn new(sender: mpsc::Sender<UiData>) -> Processor {

        let mut vd = VecDeque::<u32>::with_capacity(CIRC_BUF_SIZE);
        for _ in 0..CIRC_BUF_SIZE { vd.push_back(0); }

        Processor{timer: 0,
                 keys_total: 0,
                 tx: sender,
                 current_keys: Some(Vec::new()),
                 wpm_circ_buf: vd}
    }

    pub fn process_key(&mut self, mut kd: Keydata) {
        kd.set_timestamp(self.timer);

        if *kd.get_kind() == KeyKind::Single { self.keys_total += 1;}

        if let Some(keys) = &mut self.current_keys { keys.push(kd); }
    }

    pub fn process_second(&mut self) {
        self.timer += 1;

        if self.timer > 60 {
            let opt_keys = self.current_keys.take();
            if let Some(keys) = opt_keys {
                self.current_keys = Some(keys
                                         .into_iter()
                                         .filter(|k| k.get_timestamp() > (self.timer - 60))
                                         .collect());
            }
        }

        if let Some(keys) = &mut self.current_keys {
            let wpm = (keys.len() as u32)/5;
            self.wpm_circ_buf.pop_front();
            self.wpm_circ_buf.push_back(wpm);
        }

        /* Update the GUI every Nth second */
        if 0 == (self.timer % 10)
        {
            let mut graph_data = Vec::<(f64, f64)>::new();
            let slice_size = CIRC_BUF_SIZE / ui::X_AXIS_SIZE;
            for i in 0..ui::X_AXIS_SIZE {
                let sum:u32 = self.wpm_circ_buf
                    .iter()
                    .skip(i*slice_size)
                    .take(slice_size)
                    .sum();

                let sum = sum / slice_size as u32;
                graph_data.push(((ui::X_AXIS_SIZE-(i+1)) as f64, sum as f64));
            }

            self.tx.send(UiData{graph_data}).unwrap();

            let wpm = self.wpm_circ_buf.back().unwrap();
            Notification::new()
                .summary("WPM Alert")
                .body(&format!("Your current WPM is {}", wpm)[..])
                .show().unwrap();
        }

    }
}
