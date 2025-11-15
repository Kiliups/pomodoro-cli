use clap::{Parser, error::Result};
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};
use rodio::Decoder;
use sqlx::FromRow;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "pomodoro", subcommand_required = false)]
struct Cli {
    #[arg(short = 'f', long, help = "Focus time in minutes")]
    focus: Option<u32>,

    #[arg(short = 'b', long, help = "Break time in minutes")]
    break_time: Option<u32>,

    #[arg(short = 'c', long, help = "Number of cycles before long break")]
    cycles: Option<u32>,

    #[arg(short = 'l', long, help = "Long break time in minutes")]
    long_break: Option<u32>,

    #[arg(short = 'p', long, help = "Project of this session")]
    project: Option<String>,
}

#[derive(PartialEq)]
enum Mode {
    Focus,
    Break,
    LongBreak,
}
#[derive(Debug, FromRow, Clone)]
struct Project {
    name: String,
    seconds: u32,
}

impl Project {
    fn new(name: String) -> Self {
        Self {
            name: name,
            seconds: 0,
        }
    }

    async fn get_all(pool: &SqlitePool) -> Result<Vec<Project>, sqlx::Error> {
        let projects: Vec<Project> =
            sqlx::query_as::<_, Project>("SELECT name, seconds FROM projects")
                .fetch_all(pool)
                .await?;
        Ok(projects)
    }

    async fn get_by_name(name: &str, pool: &SqlitePool) -> Result<Project, sqlx::Error> {
        let projects: Project =
            sqlx::query_as::<_, Project>("SELECT name, seconds FROM projects WHERE name= ?")
                .bind(name)
                .fetch_one(pool)
                .await?;
        Ok(projects)
    }

    async fn insert(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO projects (name,seconds) VALUES (?, ?)")
            .bind(self.name.as_str())
            .bind(self.seconds)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE projects SET name = ?, seconds=? WHERE name= ?")
            .bind(self.name.as_str())
            .bind(self.seconds)
            .bind(self.name.as_str())
            .execute(pool)
            .await?;
        Ok(())
    }
}

#[derive(Debug, FromRow)]
struct Config {
    id: i64,
    focus: u32,
    #[sqlx(rename = "break")]
    break_time: u32,
    long_break: u32,
    cycles: u32,
}

impl Config {
    fn new(focus: u32, break_time: u32, long_break: u32, cycles: u32) -> Self {
        Self {
            id: 1,
            focus: focus,
            break_time: break_time,
            long_break: long_break,
            cycles: cycles,
        }
    }

    async fn get_all(pool: &SqlitePool) -> Result<Config, sqlx::Error> {
        let config: Config = sqlx::query_as::<_, Config>(
            "SELECT id, focus, break, long_break, cycles FROM config WHERE id = ?",
        )
        .bind(1)
        .fetch_one(pool)
        .await?;
        Ok(config)
    }

    async fn insert(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO config (focus, break, long_break, cycles) VALUES (?, ?, ?, ?)")
            .bind(self.focus)
            .bind(self.break_time)
            .bind(self.long_break)
            .bind(self.cycles)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE config SET focus = ?, break=?, long_break=?, cycles=? WHERE id = ?")
            .bind(self.focus)
            .bind(self.break_time)
            .bind(self.long_break)
            .bind(self.cycles)
            .bind(1)
            .execute(pool)
            .await?;
        Ok(())
    }
}

struct Pomodoro {
    mode: Mode,
    focus: u32,
    break_time: u32,
    long_break: u32,
    cycles: u32,
    project: Project,
    current_cycle: u32,
    remaining_secs: u32,
    running: bool,
}

impl Pomodoro {
    fn new(focus: u32, break_time: u32, long_break: u32, cycles: u32, project: Project) -> Self {
        Self {
            mode: Mode::Focus,
            focus: focus,
            break_time: break_time,
            long_break: long_break,
            cycles: cycles,
            project: project,
            current_cycle: 1,
            remaining_secs: focus * 60,
            running: false,
        }
    }

    fn tick(&mut self) {
        if self.running && self.remaining_secs > 0 {
            self.remaining_secs -= 1;
            if self.mode == Mode::Focus {
                self.project.seconds += 1;
            }
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
        // todo save elapsed
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
        "cycle: {}/{} | status: {} | [space] pause/play | [r] reset | [s] skip | [p] projects | [q] quit",
        pomo.current_cycle, pomo.cycles, status
    );

    let info_widget = Paragraph::new(info)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(info_widget, chunks[3]);
}

fn ui_projects(frame: &mut Frame, projects: &Vec<Project>) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    // Title
    let title = Paragraph::new("PROJECT PROGRESS")
        .style(
            Style::default()
                .fg(Color::Rgb(205, 218, 253))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Projects list
    let mut project_lines = Vec::new();

    // Table header
    project_lines.push(Line::from(vec![Span::styled(
        "┌────────────────┬────────────────┐",
        Style::default().fg(Color::Gray),
    )]));
    project_lines.push(Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:<14}", "Project"),
            Style::default()
                .fg(Color::Rgb(205, 218, 253))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:<14}", "Time"),
            Style::default()
                .fg(Color::Rgb(205, 218, 253))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │", Style::default().fg(Color::Gray)),
    ]));
    project_lines.push(Line::from(vec![Span::styled(
        "├────────────────┼────────────────┤",
        Style::default().fg(Color::Gray),
    )]));

    // Table rows
    for project in projects {
        let hours = project.seconds / 3600;
        let minutes = (project.seconds % 3600) / 60;
        let seconds = project.seconds % 60;
        let time_str = if hours > 0 {
            format!("{}h {:02}m {:02}s", hours, minutes, seconds)
        } else {
            format!("{}m {:02}s", minutes, seconds)
        };

        // Truncate project name if too long
        let name_display = if project.name.len() > 14 {
            format!("{}...", &project.name[..11])
        } else {
            project.name.clone()
        };

        project_lines.push(Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:<14}", name_display),
                Style::default()
                    .fg(Color::Rgb(149, 213, 178))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:<14}", time_str),
                Style::default().fg(Color::White),
            ),
            Span::styled(" │", Style::default().fg(Color::Gray)),
        ]));
    }

    // Table footer
    project_lines.push(Line::from(vec![Span::styled(
        "└────────────────┴────────────────┘",
        Style::default().fg(Color::Gray),
    )]));

    // Projects list
    let projects_widget = Paragraph::new(project_lines).alignment(Alignment::Center);
    frame.render_widget(projects_widget, chunks[1]);

    // Footer
    let footer = Paragraph::new("press [p] to return...")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:./database.db?mode=rwc")
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS config (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            focus INTEGER,
            break INTEGER,
            long_break INTEGER,
            cycles INTEGER 
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS projects (
            name TEXT NOT NULL UNIQUE PRIMARY KEY ,
            seconds INTEGER 
        )
        "#,
    )
    .execute(&pool)
    .await?;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM config")
        .fetch_one(&pool)
        .await?;

    if count.0 == 0 {
        let config = Config::new(25, 5, 15, 4);
        config.insert(&pool).await?;
    }

    let project_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects")
        .fetch_one(&pool)
        .await?;

    if project_count.0 == 0 {
        let project = Project::new(String::from("none"));
        project.insert(&pool).await?;
    }

    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let cli = Cli::parse();
    let pool = init_db().await?;
    let mut config = Config::get_all(&pool).await?;

    if cli.focus.is_some()
        || cli.break_time.is_some()
        || cli.long_break.is_some()
        || cli.cycles.is_some()
    {
        if cli.focus.is_some() {
            config.focus = cli.focus.unwrap();
        }
        if cli.break_time.is_some() {
            config.break_time = cli.break_time.unwrap();
        }
        if cli.long_break.is_some() {
            config.long_break = cli.long_break.unwrap();
        }
        if cli.cycles.is_some() {
            config.cycles = cli.cycles.unwrap();
        }
        config.update(&pool).await?;
    }

    let mut project = Project::get_by_name("none", &pool).await?;
    if let Some(project_name) = cli.project {
        match Project::get_by_name(&project_name, &pool).await {
            Ok(existing_project) => {
                project = existing_project;
            }
            Err(_) => {
                project = Project::new(project_name.clone());
                project.insert(&pool).await?;
            }
        }
    }
    let mut all_projects = Project::get_all(&pool).await?;

    let mut pomo = Pomodoro::new(
        config.focus,
        config.break_time,
        config.long_break,
        config.cycles,
        project,
    );
    let mut terminal = ratatui::init();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_secs(1);
    let mut is_project = false;

    loop {
        if is_project {
            all_projects = all_projects
                .into_iter()
                .map(|project| {
                    if project.name == pomo.project.name {
                        pomo.project.clone()
                    } else {
                        project
                    }
                })
                .collect();
            terminal.draw(|frame| ui_projects(frame, &all_projects))?;
        } else {
            terminal.draw(|frame| ui(frame, &pomo))?;
        }

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if !is_project {
                        match key.code {
                            KeyCode::Char(' ') => pomo.toggle(),
                            KeyCode::Char('r') => pomo.reset(),
                            KeyCode::Char('s') => pomo.next(),
                            _ => {}
                        }
                    }
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            pomo.project.update(&pool).await?;
                            break;
                        }
                        KeyCode::Char('c') | KeyCode::Char('x')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            pomo.project.update(&pool).await?;
                            break;
                        }
                        KeyCode::Char('p') => {
                            is_project = !is_project;
                            pomo.running = false;
                        }
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
