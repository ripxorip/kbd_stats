use evdev_rs::Device;
use std::fs;
use evdev_rs::ReadFlag;
use std::sync::mpsc;
use regex::Regex;

use crate::processor;

pub struct InputGrabber {}

impl InputGrabber {

    pub fn new() -> InputGrabber {
        InputGrabber{}
    }

    pub fn run(&self, snd: mpsc::Sender<processor::Keydata>) {
        /* Idea: move out and spawn a new thread for each "kbd" instead */
        let files = fs::read_dir("/dev/input/by-path").unwrap();

        let re = Regex::new("usb.*kbd").unwrap();
        let path = files.into_iter().filter(|f| re.is_match(f.as_ref()
                                            .unwrap()
                                            .path()
                                            .to_string_lossy()
                                            .as_ref())).last().unwrap();

        let file = fs::File::open(path.unwrap().path()).unwrap();
        let d = Device::new_from_file(file).unwrap();

        loop {
            let ev = d.next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING).map(|val| val.1);
            match ev {
                Ok(ev) => {
                    match ev.event_type() {
                        Some(et) => {
                            if (et == evdev_rs::enums::EventType::EV_KEY) && (ev.value > 0) {
                                let kd = processor::Keydata {symbol: ev.event_code.to_string() };
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
