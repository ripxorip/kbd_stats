use notify_rust::Notification;
use crate::ui;
use std::sync::mpsc;

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
    test: i32,
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
}

impl Processor {

    pub fn new(sender: mpsc::Sender<UiData>) -> Processor {
        //ui::test_ui().unwrap();
        Processor{timer: 0, keys_total: 0, tx: sender, current_keys: Some(Vec::new())}
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

        /* Test the GUI */
        self.tx.send(UiData{test: 0}).unwrap();

        /*
        Notification::new()
            .summary("Firefox News")
            .body("This will almost look like a real firefox notification.")
            .show().unwrap();
        */
    }
}
