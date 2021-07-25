use evdev_rs::Device;
use std::fs;
use evdev_rs::ReadFlag;
use std::sync::mpsc;

use crate::processor;

pub struct InputGrabber {}

impl InputGrabber {

    pub fn new() -> InputGrabber {
        InputGrabber{}
    }

    pub fn run(&self, snd: mpsc::Sender<processor::Keydata>) {
        /* Idea: move out and spawn a new thread for each "kbd" instead */
        let files = fs::read_dir("/dev/input/by-path").unwrap();
        let mut file = String::new();

        let mut ffound = false;
        for f in files {
            let s = String::from(f.unwrap().path().to_string_lossy());
            if s.contains("kbd") && s.contains("usb") {
                file = s;
                ffound = true;
            }
        }

        if !ffound {
            panic!("Failed to find input device for listening..");
        }


        let file = fs::File::open(file).unwrap();
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
