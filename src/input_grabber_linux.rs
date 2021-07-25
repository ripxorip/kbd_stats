use evdev_rs::Device;
use std::fs::File;
use evdev_rs::ReadFlag;
use std::sync::mpsc;

use crate::processor;

pub struct InputGrabber {}

impl InputGrabber {

    pub fn new() -> InputGrabber {
        InputGrabber{}
    }

    pub fn run(&self, snd: mpsc::Sender<processor::Keydata>) {
        let file = File::open("/dev/input/event16").unwrap();
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
