use clap::{Parser, error::Result};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;
mod config;
use config::Config;
mod pomodoro;
use pomodoro::Pomodoro;
mod project;
use project::Project;
mod theme;

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

async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:./database.db?mode=rwc")
        .await?;

    Config::create(&pool).await?;
    Project::create(&pool).await?;

    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let cli = Cli::parse();

    let pool = init_db().await?;
    let mut config = Config::get(&pool).await?;

    let mut config_changed = false;
    if let Some(focus) = cli.focus {
        config.set_focus(focus);
        config_changed = true;
    }
    if let Some(break_time) = cli.break_time {
        config.set_break_time(break_time);
        config_changed = true;
    }
    if let Some(long_break) = cli.long_break {
        config.set_long_break(long_break);
        config_changed = true;
    }
    if let Some(cycles) = cli.cycles {
        config.set_cycles(cycles);
        config_changed = true;
    }
    if config_changed {
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
        config.get_focus(),
        config.get_break_time(),
        config.get_long_break(),
        config.get_cycles(),
        project,
    );

    let mut terminal = ratatui::init();
    let mut is_project = false;

    loop {
        pomo.tick();

        if is_project {
            all_projects = all_projects
                .into_iter()
                .map(|project| {
                    if project.get_name() == pomo.get_project().get_name() {
                        pomo.get_project().clone()
                    } else {
                        project
                    }
                })
                .collect();
            terminal.draw(|frame| Project::ui(frame, &all_projects))?;
        } else {
            terminal.draw(|frame| pomo.ui(frame))?;
        }

        let timeout = Duration::from_secs(1).saturating_sub(pomo.get_last_tick().elapsed());

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
                            pomo.get_project().update(&pool).await?;
                            break;
                        }
                        KeyCode::Char('c') | KeyCode::Char('x')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            pomo.get_project().update(&pool).await?;
                            break;
                        }
                        KeyCode::Char('p') => {
                            is_project = !is_project;
                            pomo.set_running(false);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}
