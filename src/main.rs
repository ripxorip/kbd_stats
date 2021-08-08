use std::io;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

use termion::event::Key;
use termion::input::TermRead;

use clap::{Arg, App};

mod ui;
mod processor;
mod input_grabber_linux;


fn event_thread(event_file: Option<String>, snd: mpsc::Sender<processor::Keydata>) {
    let lig = input_grabber_linux::InputGrabber::new(event_file);
    lig.run(snd);
}

fn timer_thread(rcv: mpsc::Receiver<processor::Keydata>,
                ui_send: mpsc::Sender<processor::UiData>,
                output_file: Option<String>,
                notify_keys: Option<u32>) {
    let mut p = processor::Processor::new(ui_send, output_file, notify_keys);
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
                            panic!("Failed to read from input event, make sure to specify the correct file using --input");
                        }
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(1000));
    }
}

fn ui_thread(rcv: mpsc::Receiver<processor::UiData>)
{
    let mut u = ui::UI::new(rcv);
    u.run();
}

fn get_params() -> (Option<String>, Option<String>, Option<u32>) {
    let matches = App::new("Keyboard Statistics")
        .version("0.1.0")
        .author("Philip Karlsson Gisslow <ripxorip@gmail.com>")
        .about("Program used to log/measure Keyboard metrics")
        .arg(Arg::with_name("input_file")
                 .short("i")
                 .long("input_file")
                 .takes_value(true)
                 .required(true) // Can be false for dev (therefore the Option)
                 .help("The input file to listen to. (Hint: See /dev/input/by-path)"))
        .arg(Arg::with_name("output_file")
                 .short("o")
                 .long("output_file")
                 .takes_value(true)
                 .required(false)
                 .help("Write the statistics to file"))
        .arg(Arg::with_name("notify_keys")
                 .short("n")
                 .long("notify_keys")
                 .takes_value(true)
                 .required(false)
                 .help("The number of keypresses needed to trigger a notification (interval)"))
        .get_matches();

    let notify_keys = match matches.value_of("notify_keys") {
        Some(s) => {
            Some(s.parse::<u32>().unwrap())
        }
        None => {
            None
        }
    };
    let input_path = match matches.value_of("input_file") {
        Some(s) => {
            Some(String::from(s))
        }
        None => {
            None
        }
    };
    let output_file = match matches.value_of("output_file") {
        Some(s) => {
            Some(String::from(s))
        }
        None => {
            None
        }
    };
    (input_path, output_file, notify_keys)
}

fn main() {
    let (input_path, output_file, notify_keys) = get_params();

    let (ui_send, ui_recv) = mpsc::channel();
    let _ui_thread_handle = thread::spawn(move || {
        ui_thread(ui_recv);
    });
    let (send, recv) = mpsc::channel();
    let _timer_thread_handle = thread::spawn(move || {
        timer_thread(recv, ui_send, output_file, notify_keys);
    });

    let _event_thread_handle = thread::spawn(move || {
        event_thread(input_path, send);
    });

    loop {
        let stdin = io::stdin();
        for evt in stdin.keys() {
            if let Ok(key) = evt {
                if key == Key::Char('q') {
                    /* FIXME: Exit gracefully using the thread handles */
                    return;
                }
            }
        }
    }
}
