#[allow(dead_code)]

use crate::util::{
    event::{Event, Events},
    SinSignal,
};

use std::sync::mpsc;
use crate::processor;

use std::collections::VecDeque;

use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset},
    Terminal,
};

struct App {
    signal1: SinSignal,
    data1: Vec<(f64, f64)>,
    window: [f64; 2],
}

impl App {
    fn new() -> App {
        let mut signal1 = SinSignal::new(0.2, 0.5, 18.0);
        let data1 = signal1.by_ref().take(200).collect::<Vec<(f64, f64)>>();
        App {
            signal1,
            data1,
            window: [0.0, 20.0],
        }
    }

    fn update(&mut self) {
        for _ in 0..5 {
            self.data1.remove(0);
        }
        self.data1.extend(self.signal1.by_ref().take(5));
        self.window[0] += 1.0;
        self.window[1] += 1.0;
    }
}

pub fn test_ui() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    // App
    let mut app = App::new();

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                    ]
                    .as_ref(),
                )
                .split(size);
            let x_labels = vec![
                Span::styled(
                    format!("{}", app.window[0]),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{}", (app.window[0] + app.window[1]) / 2.0)),
                Span::styled(
                    format!("{}", app.window[1]),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ];
            let datasets = vec![
                Dataset::default()
                    .name("data1")
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&app.data1),
            ];

            let chart = Chart::new(datasets)
                .block(
                    Block::default()
                        .title(Span::styled(
                            "WPM",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ))
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .title("Time (Seconds)")
                        .style(Style::default().fg(Color::Gray))
                        .labels(x_labels)
                        .bounds(app.window),
                )
                .y_axis(
                    Axis::default()
                        .title("Y Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels(vec![
                            Span::styled("-20", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw("0"),
                            Span::styled("20", Style::default().add_modifier(Modifier::BOLD)),
                        ])
                        .bounds([-20.0, 20.0]),
                );
            f.render_widget(chart, chunks[2]);
        })?;

        match events.next()? {
            Event::Input(input) => {
                if input == Key::Char('q') {
                    break;
                }
            }
            Event::Tick => {
                app.update();
            }
        }
    }
    Ok(())
}

pub struct UI {
    rx: mpsc::Receiver<processor::UiData>,
    /* Incoming ringbuffer */
    in_buf: VecDeque<f64>,
    /* Window */
    x_axis_window: [f64; 2],
    /* X and Y */
    data: Vec<(f64, f64)>,
}

impl UI {
    pub fn new(rcv: mpsc::Receiver<processor::UiData>) -> UI {

        let mut vd = VecDeque::<f64>::with_capacity(20);
        let mut dv = Vec::<(f64, f64)>::with_capacity(20);

        for i in 0..20 {
            vd.push_back(i as f64);
            dv.push((0.00, 0.00));
        }

        UI{rx: rcv, in_buf: vd, x_axis_window: [0.0, 20.0], data: dv }
    }

    pub fn run(&mut self) {
        /* Start the UI and then loop */
        let stdout = io::stdout().into_raw_mode().unwrap();
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        loop {
            terminal.draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                        ]
                        .as_ref(),
                        )
                    .split(size);
                let x_labels = vec![
                    Span::styled(
                        format!("{}", self.x_axis_window[0]),
                        Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!("{}", (self.x_axis_window[0] + self.x_axis_window[1]) / 2.0)),
                        Span::styled(
                            format!("{}", self.x_axis_window[1]),
                            Style::default().add_modifier(Modifier::BOLD),
                            ),
                ];
                let datasets = vec![
                    Dataset::default()
                        .name("data")
                        .marker(symbols::Marker::Dot)
                        .style(Style::default().fg(Color::Cyan))
                        .data(&self.data),
                ];

                let chart = Chart::new(datasets)
                    .block(
                        Block::default()
                        .title(Span::styled(
                                "WPM",
                                Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                                ))
                        .borders(Borders::ALL),
                        )
                    .x_axis(
                        Axis::default()
                        .title("Time (Seconds)")
                        .style(Style::default().fg(Color::Gray))
                        .labels(x_labels)
                        .bounds(self.x_axis_window),
                        )
                    .y_axis(
                        Axis::default()
                        .title("Y Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels(vec![
                                Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                                Span::raw("0"),
                                Span::styled("20", Style::default().add_modifier(Modifier::BOLD)),
                        ])
                        .bounds([0.00, 20.00]),
                        );
                f.render_widget(chart, chunks[2]);
            }).unwrap();

            let _msg = self.rx.recv().unwrap();

            /* Shall push data from the msg instead */
            self.in_buf.pop_front();
            self.in_buf.push_back(5.00);

            for (i, it) in self.in_buf.iter().enumerate() { self.data[i] = (i as f64, *it); }
        }
    }
}
