use crate::project::Project;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};
use rodio::Decoder;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

#[derive(PartialEq)]
pub enum Mode {
    Focus,
    Break,
    LongBreak,
}

pub struct Pomodoro {
    // todo not pub
    mode: Mode,
    pub focus: u32,
    pub break_time: u32,
    pub long_break: u32,
    pub cycles: u32,
    pub project: Project,
    current_cycle: u32,
    remaining_secs: u32,
    pub running: bool,
    pub last_tick: Instant,
}

impl Pomodoro {
    pub fn new(
        focus: u32,
        break_time: u32,
        long_break: u32,
        cycles: u32,
        project: Project,
    ) -> Self {
        Self {
            mode: Mode::Focus,
            focus,
            break_time,
            long_break,
            cycles,
            project,
            current_cycle: 1,
            remaining_secs: focus * 60,
            running: false,
            last_tick: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        let tick_rate = Duration::from_secs(1);

        if self.last_tick.elapsed() >= tick_rate {
            if self.running && self.remaining_secs > 0 {
                self.remaining_secs -= 1;
                if self.mode == Mode::Focus {
                    self.project.seconds += 1;
                }
            } else if self.running && self.remaining_secs == 0 {
                self.notify();
                self.next();
            }
            self.last_tick = Instant::now();
        }
    }

    pub fn reset(&mut self) {
        self.current_cycle = 0;
        self.mode = Mode::Focus;
        self.remaining_secs = self.focus * 60;
        self.running = false;
    }

    pub fn mode_name(&self) -> &'static str {
        match self.mode {
            Mode::Focus => "FOCUS",
            Mode::Break => "BREAK",
            Mode::LongBreak => "LONG BREAK",
        }
    }

    pub fn mode_color(&self) -> Color {
        match self.mode {
            Mode::Focus => Color::Rgb(205, 218, 253),
            Mode::Break => Color::Rgb(149, 213, 178), //todo
            Mode::LongBreak => Color::Rgb(82, 183, 136), // todo
        }
    }

    pub fn next(&mut self) {
        match self.mode {
            Mode::Focus => {
                if self.current_cycle == self.cycles {
                    self.mode = Mode::LongBreak;
                    self.remaining_secs = self.long_break * 60;
                } else {
                    self.mode = Mode::Break;
                    self.remaining_secs = self.break_time * 60;
                }
            }
            Mode::Break => {
                self.current_cycle += 1;
                self.mode = Mode::Focus;
                self.remaining_secs = self.focus * 60;
            }
            Mode::LongBreak => {
                self.current_cycle = 1;
                self.mode = Mode::Focus;
                self.remaining_secs = self.focus * 60;
            }
        }
    }

    pub fn toggle(&mut self) {
        self.running = !self.running;
    }

    pub fn notify(&self) {
        std::thread::spawn(move || {
            if let Ok(file) = File::open("./notification.mp3") {
                let buf_reader = BufReader::new(file);
                if let Ok(source) = Decoder::new(buf_reader) {
                    if let Ok(mut stream_handle) = rodio::OutputStreamBuilder::open_default_stream()
                    {
                        stream_handle.log_on_drop(false);
                        let sink = rodio::Sink::connect_new(stream_handle.mixer());
                        sink.append(source);
                        sink.sleep_until_end();
                    }
                }
            }
        });
    }

    pub fn ui(&self, frame: &mut Frame) {
        let size = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(5),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(size);

        // title
        let title = Paragraph::new(self.mode_name())
            .style(
                Style::default()
                    .fg(self.mode_color())
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // ASCII
        let timer_lines = draw_timer_ascii(self.remaining_secs);
        let timer_text: Vec<Line> = timer_lines
            .iter()
            .map(|line| {
                Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(self.mode_color()),
                ))
            })
            .collect();

        let timer = Paragraph::new(timer_text).alignment(Alignment::Center);
        frame.render_widget(timer, chunks[1]);

        // info
        let status = if self.running { "running" } else { "paused" };
        let info = format!(
            "cycle: {}/{} | status: {} | [space] pause/play | [r] reset | [s] skip | [p] projects | [q] quit",
            self.current_cycle, self.cycles, status
        );

        let info_widget = Paragraph::new(info)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(info_widget, chunks[3]);
    }
}

fn format_time(secs: u32) -> String {
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}", mins, secs)
}

fn draw_timer_ascii(remaining: u32) -> Vec<String> {
    let time_str = format_time(remaining);
    let chars: Vec<char> = time_str.chars().collect();

    let mut lines = vec![String::new(); 5];

    for ch in chars {
        let digit_lines = match ch {
            '0' => vec![" ███ ", "█   █", "█   █", "█   █", " ███ "],
            '1' => vec!["  █  ", " ██  ", "  █  ", "  █  ", "█████"],
            '2' => vec![" ███ ", "█   █", "  ██ ", "██   ", "█████"],
            '3' => vec![" ███ ", "█   █", "  ██ ", "█   █", " ███ "],
            '4' => vec!["█   █", "█   █", "█████", "    █", "    █"],
            '5' => vec!["█████", "█    ", "████ ", "    █", "████ "],
            '6' => vec![" ███ ", "█    ", "████ ", "█   █", " ███ "],
            '7' => vec!["█████", "    █", "   █ ", "  █  ", "  █  "],
            '8' => vec![" ███ ", "█   █", " ███ ", "█   █", " ███ "],
            '9' => vec![" ███ ", "█   █", " ████", "    █", " ███ "],
            ':' => vec!["   ", " █ ", "   ", " █ ", "   "],
            _ => vec!["     ", "     ", "     ", "     ", "     "],
        };

        for i in 0..5 {
            lines[i].push_str(digit_lines[i]);
            lines[i].push(' ');
        }
    }

    lines
}
