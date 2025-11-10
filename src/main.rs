use clap::{Parser, Subcommand};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph},
};

#[derive(Parser)]
#[command(name = "pomodoro", subcommand_required = false)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start {
        #[arg(short = 'f', long, default_value_t = 25)]
        focus: u32,

        #[arg(short = 'b', long, default_value_t = 5)]
        break_time: u32,

        #[arg(short = 'c', long, default_value_t = 4)]
        cycles: u32,

        #[arg(short = 'l', long, default_value_t = 15)]
        long_break: u32,
    },
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
    elapsed_secs: u64,
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
            elapsed_secs: 0,
            running: false,
        }
    }

    fn reset(&mut self) {
        self.elapsed_secs = 0;
        self.running = false;
    }

    fn tick(&mut self) {
        /*if self.running && self.elapsed_secs < self.total_secs {
            self.elapsed_secs += 1;
        }*/
    }

    fn progress(&self) {
        /*self.elapsed_secs as f64 / self.total_secs as f64*/
    }

    fn mode_name(&self) -> &'static str {
        match self.mode {
            Mode::Focus => "FOCUS",
            Mode::Break => "BREAK",
            Mode::LongBreak => "LONG BREAK",
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start {
            focus,
            break_time,
            cycles,
            long_break,
        } => {
            let pomo = Pomodoro::new(*focus, *break_time, *long_break, *cycles);
            let mut terminal = ratatui::init();
            loop {
                terminal
                    .draw(|frame| {
                        let size = frame.area();
                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .margin(4)
                            .constraints([
                                Constraint::Length(3),
                                Constraint::Length(3),
                                Constraint::Min(1),
                            ])
                            .split(size);

                        frame.render_widget(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(pomo.mode_name()),
                            chunks[0],
                        );
                    })
                    .expect("failed to draw frame");
                if matches!(event::read().expect("failed to read event"), Event::Key(_)) {
                    break;
                }
            }
            ratatui::restore();
        }
    }
}
