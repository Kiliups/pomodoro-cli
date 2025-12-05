use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::theme::Base16;
use crate::theme::Theme;
use sqlx::FromRow;
use sqlx::sqlite::SqlitePool;
use std::str::FromStr;

#[derive(Debug, FromRow, Clone)]
pub struct Project {
    name: String,
    focus_seconds: u32,
    total_seconds: u32,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            name,
            focus_seconds: 0,
            total_seconds: 0,
        }
    }

    pub async fn create(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        // check if we need to migrate from old schema
        let has_old_schema: Result<(i32,), _> =
            sqlx::query_as("SELECT seconds FROM projects LIMIT 1")
                .fetch_one(pool)
                .await;

        if has_old_schema.is_ok() {
            // migrate: rename seconds to focus_seconds and add total_seconds
            sqlx::query("ALTER TABLE projects RENAME COLUMN seconds TO focus_seconds")
                .execute(pool)
                .await?;
            sqlx::query("ALTER TABLE projects ADD COLUMN total_seconds INTEGER DEFAULT 0")
                .execute(pool)
                .await?;
            // set total_seconds to the same value as focus_seconds initially
            sqlx::query("UPDATE projects SET total_seconds = focus_seconds")
                .execute(pool)
                .await?;
            return Ok(());
        }

        // create table with new schema if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS projects (
            name TEXT NOT NULL UNIQUE PRIMARY KEY ,
            focus_seconds INTEGER,
            total_seconds INTEGER
        )
        "#,
        )
        .execute(pool)
        .await?;

        let project_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects")
            .fetch_one(pool)
            .await?;

        if project_count.0 == 0 {
            let project = Project::new(String::from("none"));
            project.insert(pool).await?;
        }

        Ok(())
    }

    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Project>, sqlx::Error> {
        let projects: Vec<Project> =
            sqlx::query_as::<_, Project>("SELECT name, focus_seconds, total_seconds FROM projects")
                .fetch_all(pool)
                .await?;
        Ok(projects)
    }

    pub async fn get_by_name(name: &str, pool: &SqlitePool) -> Result<Project, sqlx::Error> {
        let projects: Project = sqlx::query_as::<_, Project>(
            "SELECT name, focus_seconds, total_seconds FROM projects WHERE name= ?",
        )
        .bind(name)
        .fetch_one(pool)
        .await?;
        Ok(projects)
    }

    pub async fn insert(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO projects (name,focus_seconds,total_seconds) VALUES (?, ?,?)")
            .bind(self.name.as_str())
            .bind(self.focus_seconds)
            .bind(self.total_seconds)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE projects SET focus_seconds=?, total_seconds=? WHERE name= ?")
            .bind(self.focus_seconds)
            .bind(self.total_seconds)
            .bind(self.name.as_str())
            .execute(pool)
            .await?;
        Ok(())
    }

    pub fn ui(frame: &mut Frame, projects: &Vec<Project>) {
        let size = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(size);

        let title = Paragraph::new("PROJECT PROGRESS")
            .style(
                Style::default()
                    .fg(Color::from_str(Theme::default().get_color(Base16::Base05)).unwrap())
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        let mut project_lines = Vec::new();

        project_lines.push(Line::from(vec![Span::styled(
            "┌────────────────┬────────────────┬────────────────┐",
            Style::default().fg(Color::Gray),
        )]));
        project_lines.push(Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:<14}", "project"),
                Style::default()
                    .fg(Color::from_str(Theme::default().get_color(Base16::Base05)).unwrap())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:<14}", "focus time"),
                Style::default()
                    .fg(Color::from_str(Theme::default().get_color(Base16::Base05)).unwrap())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:<14}", "total time"),
                Style::default()
                    .fg(Color::from_str(Theme::default().get_color(Base16::Base05)).unwrap())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │", Style::default().fg(Color::Gray)),
        ]));
        project_lines.push(Line::from(vec![Span::styled(
            "├────────────────┼────────────────┼────────────────┤",
            Style::default().fg(Color::Gray),
        )]));

        for project in projects {
            let hours = project.focus_seconds / 3600;
            let minutes = (project.focus_seconds % 3600) / 60;
            let seconds = project.focus_seconds % 60;

            let total_hours = project.total_seconds / 3600;
            let total_minutes = (project.total_seconds % 3600) / 60;
            let total_seconds = project.total_seconds % 60;

            let time_str = if hours > 0 {
                format!("{}h {:02}m {:02}s", hours, minutes, seconds)
            } else {
                format!("{}m {:02}s", minutes, seconds)
            };

            let total_time_str = if total_hours > 0 {
                format!(
                    "{}h {:02}m {:02}s",
                    total_hours, total_minutes, total_seconds
                )
            } else {
                format!("{}m {:02}s", total_minutes, total_seconds)
            };

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
                        .fg(Color::from_str(Theme::default().get_color(Base16::Base0B)).unwrap())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" │ ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:<14}", time_str),
                    Style::default().fg(Color::White),
                ),
                Span::styled(" │ ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:<14}", total_time_str),
                    Style::default().fg(Color::White),
                ),
                Span::styled(" │", Style::default().fg(Color::Gray)),
            ]));
        }

        project_lines.push(Line::from(vec![Span::styled(
            "└────────────────┴────────────────┴────────────────┘",
            Style::default().fg(Color::Gray),
        )]));

        let projects_widget = Paragraph::new(project_lines).alignment(Alignment::Center);
        frame.render_widget(projects_widget, chunks[1]);

        let footer = Paragraph::new("press [p] to return...")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_focus_seconds(&self) -> u32 {
        self.focus_seconds
    }

    pub fn set_focus_seconds(&mut self, seconds: u32) {
        self.focus_seconds = seconds;
    }

    pub fn get_total_seconds(&self) -> u32 {
        self.total_seconds
    }

    pub fn set_total_seconds(&mut self, seconds: u32) {
        self.total_seconds = seconds;
    }
}
