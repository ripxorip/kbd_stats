#[allow(dead_code)]

use std::collections::VecDeque;
use std::sync::mpsc;
use crate::processor;

use std::io;
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout, Corner},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{BarChart, Axis, Block, Borders, Chart, Dataset, List, ListItem},
    Terminal,
};

pub const X_AXIS_SIZE:usize = 20;

pub struct UI {
    rx: mpsc::Receiver<processor::UiEvent>,
    /* Window */
    x_axis_window: [f64; 2],
    /* X and Y */
    data: Vec<(f64, f64)>,
    /* Circular buffer for all the info strings */
    info_buf: VecDeque<String>,
    /* The barchart vector type */
    barchart_vec: Vec<(String, u32)>,
}

impl UI {
    pub fn new(rcv: mpsc::Receiver<processor::UiEvent>) -> UI {

        let mut dv = Vec::<(f64, f64)>::with_capacity(X_AXIS_SIZE);

        for _ in 0..X_AXIS_SIZE {
            dv.push((0.00, 0.00));
        }

        let mut info_buf = VecDeque::<String>::new();

        for _ in 0..100 {
            info_buf.push_back(String::from(""));
        }

        let barchart_vec = Vec::<(String, u32)>::new();

        UI{rx: rcv, x_axis_window: [0.0, X_AXIS_SIZE as f64], data: dv , info_buf, barchart_vec}
    }

    pub fn run(&mut self) {
        /* Start the UI and then loop */
        let stdout = io::stdout().into_raw_mode().unwrap();
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.clear().unwrap();

        loop {
            terminal.draw(|f| {
                /* Graph */
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
                        .title("Time (1 Hour)")
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
                                Span::styled("60", Style::default().add_modifier(Modifier::BOLD)),
                        ])
                        .bounds([0.00, 60.00]),
                        );
                f.render_widget(chart, chunks[2]);

                /* Info list */
                let mut info_items = Vec::<ListItem>::new();
                self.info_buf.iter().rev().for_each(|s| {info_items.push(ListItem::new(vec![Spans::from(&s[..])]))});
                let list = List::new(info_items)
                    .block(Block::default().borders(Borders::ALL).title("Info"))
                    .start_corner(Corner::BottomLeft);
                f.render_widget(list, chunks[1]);

                let mut bcd = Vec::<(&str, u64)>::new();
                self.barchart_vec.iter().for_each(|x| bcd.push((&x.0[..], x.1 as u64)));

                /* Bar chart */
                let barchart = BarChart::default()
                    .block(Block::default().title("Keys").borders(Borders::ALL))
                    .data(&bcd)
                    .bar_width(9)
                    .bar_style(Style::default().fg(Color::Yellow))
                    .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));
                f.render_widget(barchart, chunks[0]);
            }).unwrap();

            /* Get new data for plotting */
            let msg = match self.rx.recv().unwrap() {
                processor::UiEvent::NewData(m) => {
                    m
                }
                processor::UiEvent::Kill => {
                    break;
                }
            };

            self.data = msg.graph_data;
            self.info_buf.pop_front();
            self.info_buf.push_back(msg.info_string);
            self.barchart_vec = msg.key_freq;
        }
    }
}
