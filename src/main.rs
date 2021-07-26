use std::thread;
use std::time::Duration;
use std::sync::mpsc;

mod util;
mod ui;
mod processor;
mod input_grabber_linux;

fn event_thread(snd: mpsc::Sender<processor::Keydata>) {
    let lig = input_grabber_linux::InputGrabber::new();
    lig.run(snd);
}

fn timer_thread(rcv: mpsc::Receiver<processor::Keydata>) {
    let mut p = processor::Processor::new();
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
        /* Update GUI / stats? */
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
