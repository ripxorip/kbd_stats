use evdev_rs::Device;
use std::fs;
use evdev_rs::ReadFlag;
use std::sync::mpsc;
use regex::Regex;

use crate::processor;

pub struct InputGrabber {
    input_path: Option<String>
}

impl InputGrabber {

    pub fn new(input_path: Option<String>) -> InputGrabber {
        InputGrabber{input_path}
    }

    pub fn run(&self, snd: mpsc::Sender<processor::Keydata>) {
        let file = match &self.input_path {
            Some(s) => {
                fs::File::open(s).expect("Failed to open input_path")
            }
            None => {
                /* Idea: move out and spawn a new thread for each "kbd" instead */
                let files = fs::read_dir("/dev/input/by-path").unwrap();

                let re = Regex::new("usb.*kbd").unwrap();
                let path = files.into_iter().filter(|f| re.is_match(f.as_ref()
                                                    .unwrap()
                                                    .path()
                                                    .to_string_lossy()
                                                    .as_ref())).last().unwrap();

                fs::File::open(path.unwrap().path()).unwrap()
            }
        };

        let d = Device::new_from_file(file).unwrap();

        loop {
            let ev = d.next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING).map(|val| val.1);
            match ev {
                Ok(ev) => {
                    match ev.event_type() {
                        Some(et) => {
                            if (et == evdev_rs::enums::EventType::EV_KEY) && (ev.value > 0) {

                                let keykind = match ev.value {
                                    1 => {
                                        processor::KeyKind::Single
                                    },
                                    2 => {
                                        processor::KeyKind::Hold
                                    }
                                    _ => {
                                        panic!("Unknown keykind {}", ev.value);
                                    }
                                };

                                let kd = processor::Keydata::new(ev.event_code.to_string(), keykind);
                                snd.send(kd).unwrap();
                            }
                        },
                        None => (),
                    }
                }
                Err(_e) => (),
            }
        }
    }

}
