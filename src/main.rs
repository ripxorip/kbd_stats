use evdev_rs::Device;
use std::fs::File;
use evdev_rs::ReadFlag;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

mod processor;

fn event_thread(snd: mpsc::Sender<processor::Keydata>) {
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

fn timer_thread(rcv: mpsc::Receiver<processor::Keydata>) {
    let p = processor::Processor::new();
    loop {
        loop {
            let res = rcv.try_recv();
            match res {
                Ok(key) => {
                    /* Process the key */
                    p.process_key(key);
                },
                Err(reason) => {
                    match reason {
                        mpsc::TryRecvError::Empty => {
                            p.process_second();
                            break;
                        },
                        mpsc::TryRecvError::Disconnected => {
                            panic!("Channel Disconnected");
                        }
                    }

                }
            }
        }
        /* Update GUI / stats */
        thread::sleep(Duration::from_millis(1000));
    }
}

fn main() {
    let (send, recv) = mpsc::channel();
    let _timer_thread_handle = thread::spawn(move || {
        timer_thread(recv);
    });
    event_thread(send);
}

// try_recv, thread, move to lib.rs, read from the book again
