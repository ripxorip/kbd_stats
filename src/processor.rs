use notify_rust::Notification;
use std::collections::VecDeque;
use std::sync::mpsc;
use crate::ui;
use std::collections::HashMap;

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
    pub info_string: String,
    pub key_freq: Vec<(String, u32)>,
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
    wpm: u32,
    characters: HashMap<String, u32>,
    output_file: Option<String>,
}

impl Processor {
    pub fn new(sender: mpsc::Sender<UiData>, output_file: Option<String>) -> Processor {

        let mut vd = VecDeque::<u32>::with_capacity(CIRC_BUF_SIZE);
        for _ in 0..CIRC_BUF_SIZE { vd.push_back(0); }
        let characters = HashMap::new();

        Processor{timer: 0,
                 keys_total: 0,
                 tx: sender,
                 current_keys: Some(Vec::new()),
                 wpm_circ_buf: vd,
                 wpm: 0,
                 characters,
                 output_file,}
    }

    pub fn process_key(&mut self, mut kd: Keydata) {
        kd.set_timestamp(self.timer);

        if *kd.get_kind() == KeyKind::Single { self.keys_total += 1;}

        let mut count = 1;
        if let Some(c) = self.characters.get(kd.get_symbol()) { count = *c + 1; }
        self.characters.insert(kd.get_symbol().clone(), count);

        if let Some(keys) = &mut self.current_keys { keys.push(kd); }
    }

    fn filter_keys(&mut self) {
        let opt_keys = self.current_keys.take();
        if let Some(keys) = opt_keys {
            self.current_keys = Some(keys
                                     .into_iter()
                                     .filter(|k| k.get_timestamp() > (self.timer - 60))
                                     .collect());
        }
    }

    fn calculate_wpm(&mut self) {
        if let Some(keys) = &mut self.current_keys {
            self.wpm = (keys.len() as u32)/5;
            self.wpm_circ_buf.pop_front();
            self.wpm_circ_buf.push_back(self.wpm);
        }
    }

    fn produce_graph_data(&self) -> Vec::<(f64, f64)> {
        let mut graph_data = Vec::<(f64, f64)>::new();
        let slice_size = CIRC_BUF_SIZE / ui::X_AXIS_SIZE;
        for i in 0..ui::X_AXIS_SIZE {
            let sum:u32 = self.wpm_circ_buf
                .iter()
                .rev()
                .skip(i*slice_size)
                .take(slice_size)
                .sum();

            let sum = sum / slice_size as u32;
            graph_data.push((i as f64, sum as f64));
        }
        graph_data
    }

    fn check_notify(&self) {
        /* TODO Implement when to notify (e.g every 10000th event ?) */
        let shall_notify = false;
        if shall_notify {
            Notification::new()
                .summary("WPM Alert")
                .body(&format!("Your current WPM is {}", self.wpm)[..])
                .show().unwrap();
        }
    }

    fn write_stats_to_file(&self) {
        /* TODO Implement the writing of the stats */
    }

    pub fn process_second(&mut self) {
        self.timer += 1;
        if self.timer > 60 { self.filter_keys(); }

        self.calculate_wpm();

        let graph_data = self.produce_graph_data();

        let mut char_vec: Vec<(&String, &u32)> = self.characters.iter().collect();
        char_vec.sort_by(|a, b| b.1.cmp(a.1));

        let info_string = String::from(format!("Current WPM: {} Total Keys: {}",
                                               self.wpm, self.keys_total));

        let mut key_freq = Vec::<(String, u32)>::new();
        char_vec.iter().for_each(|x| key_freq.push((x.0.clone(), *x.1)));

        self.tx.send(UiData{graph_data, info_string, key_freq}).unwrap();

        self.check_notify();

        if let Some(_) = self.output_file { self.write_stats_to_file() };
    }
}
