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

struct CLIParams {
    input_path: Option<String>,
}

fn event_thread(event_file: Option<String>, snd: mpsc::Sender<processor::Keydata>) {
    let lig = input_grabber_linux::InputGrabber::new(event_file);
    lig.run(snd);
}

fn timer_thread(rcv: mpsc::Receiver<processor::Keydata>, ui_send: mpsc::Sender<processor::UiData>) {
    let mut p = processor::Processor::new(ui_send);
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
        thread::sleep(Duration::from_millis(1000));
    }
}

fn ui_thread(rcv: mpsc::Receiver<processor::UiData>)
{
    let mut u = ui::UI::new(rcv);
    u.run();
}

fn get_params() -> CLIParams {
    let matches = App::new("Keyboard Statistics")
        .version("0.1.0")
        .author("Philip Karlsson Gisslow <ripxorip@gmail.com>")
        .about("Program used to log/measure Keyboard metrics")
        .arg(Arg::with_name("input_file")
                 .short("i")
                 .long("input_file")
                 .takes_value(true)
                 .required(false)
                 .help("The input file to listen to. (Hint: See /dev/input/by-path)"))
        .get_matches();
    let input_path = match matches.value_of("input_file") {
        Some(s) => {
            Some(String::from(s))
        }
        None => {
            None
        }
    };
    CLIParams {input_path}
}

fn main() {
    let params = get_params();

    let (ui_send, ui_recv) = mpsc::channel();
    let _ui_thread_handle = thread::spawn(move || {
        ui_thread(ui_recv);
    });
    let (send, recv) = mpsc::channel();
    let _timer_thread_handle = thread::spawn(move || {
        timer_thread(recv, ui_send);
    });

    let _event_thread_handle = thread::spawn(move || {
        event_thread(params.input_path, send);
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
