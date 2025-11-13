use clap::Parser;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};
use rodio::Decoder;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "pomodoro", subcommand_required = false)]
struct Cli {
    #[arg(
        short = 'f',
        long,
        default_value_t = 25,
        help = "Focus time in minutes"
    )]
    focus: u32,

    #[arg(short = 'b', long, default_value_t = 5, help = "Break time in minutes")]
    break_time: u32,

    #[arg(
        short = 'c',
        long,
        default_value_t = 4,
        help = "Number of cycles before long break"
    )]
    cycles: u32,

    #[arg(
        short = 'l',
        long,
        default_value_t = 15,
        help = "Long break time in minutes"
    )]
    long_break: u32,
}

enum Mode {
    Focus,
    Break,
    LongBreak,
}

struct Pomodoro {
    mode: Mode,
    focus: u32,
    break_time: u32,
    long_break: u32,
    cycles: u32,
    current_cycle: u32,
    remaining_secs: u32,
    running: bool,
}

impl Pomodoro {
    fn new(focus: u32, break_time: u32, long_break: u32, cycles: u32) -> Self {
        Self {
            mode: Mode::Focus,
            focus: focus,
            break_time: break_time,
            long_break: long_break,
            cycles: cycles,
            current_cycle: 1,
            remaining_secs: focus * 60,
            running: false,
        }
    }

    fn tick(&mut self) {
        if self.running && self.remaining_secs > 0 {
            self.remaining_secs -= 1;
        } else if self.running && self.remaining_secs == 0 {
            self.notify();
            self.next();
        }
    }

    fn reset(&mut self) {
        self.remaining_secs = self.focus * 60;
        self.running = false;
    }

    fn mode_name(&self) -> &'static str {
        match self.mode {
            Mode::Focus => "FOCUS",
            Mode::Break => "BREAK",
            Mode::LongBreak => "LONG BREAK",
        }
    }

    fn mode_color(&self) -> Color {
        match self.mode {
            Mode::Focus => Color::Rgb(205, 218, 253),
            Mode::Break => Color::Rgb(149, 213, 178), //todo
            Mode::LongBreak => Color::Rgb(82, 183, 136), // todo
        }
    }

    fn next(&mut self) {
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

    fn toggle(&mut self) {
        self.running = !self.running;
    }

    fn notify(&self) {
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

fn ui(frame: &mut Frame, pomo: &Pomodoro) {
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
    let title = Paragraph::new(pomo.mode_name())
        .style(
            Style::default()
                .fg(pomo.mode_color())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // ASCII
    let timer_lines = draw_timer_ascii(pomo.remaining_secs);
    let timer_text: Vec<Line> = timer_lines
        .iter()
        .map(|line| {
            Line::from(Span::styled(
                line.clone(),
                Style::default().fg(pomo.mode_color()),
            ))
        })
        .collect();

    let timer = Paragraph::new(timer_text).alignment(Alignment::Center);
    frame.render_widget(timer, chunks[1]);

    // info
    let status = if pomo.running { "running" } else { "paused" };
    let info = format!(
        "cycle: {}/{} | status: {} | [space] pause/play | [r] reset | [s] skip | [q] quit",
        pomo.current_cycle, pomo.cycles, status
    );

    let info_widget = Paragraph::new(info)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(info_widget, chunks[3]);
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let mut pomo = Pomodoro::new(cli.focus, cli.break_time, cli.long_break, cli.cycles);
    let mut terminal = ratatui::init();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_secs(1);

    loop {
        terminal.draw(|frame| ui(frame, &pomo))?;
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') | KeyCode::Char('x')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        KeyCode::Char(' ') => pomo.toggle(),
                        KeyCode::Char('r') => pomo.reset(),
                        KeyCode::Char('s') => pomo.next(),
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            pomo.tick();
            last_tick = Instant::now();
        }
    }

    ratatui::restore();
    Ok(())
}
