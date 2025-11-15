use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use sqlx::FromRow;
use sqlx::sqlite::SqlitePool;

#[derive(Debug, FromRow, Clone)]
pub struct Project {
    pub name: String,
    pub seconds: u32,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self { name, seconds: 0 }
    }

    pub async fn create(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS projects (
            name TEXT NOT NULL UNIQUE PRIMARY KEY ,
            seconds INTEGER 
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
            sqlx::query_as::<_, Project>("SELECT name, seconds FROM projects")
                .fetch_all(pool)
                .await?;
        Ok(projects)
    }

    pub async fn get_by_name(name: &str, pool: &SqlitePool) -> Result<Project, sqlx::Error> {
        let projects: Project =
            sqlx::query_as::<_, Project>("SELECT name, seconds FROM projects WHERE name= ?")
                .bind(name)
                .fetch_one(pool)
                .await?;
        Ok(projects)
    }

    pub async fn insert(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO projects (name,seconds) VALUES (?, ?)")
            .bind(self.name.as_str())
            .bind(self.seconds)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE projects SET seconds=? WHERE name= ?")
            .bind(self.seconds)
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
                    .fg(Color::Rgb(205, 218, 253))
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        let mut project_lines = Vec::new();

        project_lines.push(Line::from(vec![Span::styled(
            "┌────────────────┬────────────────┐",
            Style::default().fg(Color::Gray),
        )]));
        project_lines.push(Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:<14}", "project"),
                Style::default()
                    .fg(Color::Rgb(205, 218, 253))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:<14}", "time"),
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

        for project in projects {
            let hours = project.seconds / 3600;
            let minutes = (project.seconds % 3600) / 60;
            let seconds = project.seconds % 60;
            let time_str = if hours > 0 {
                format!("{}h {:02}m {:02}s", hours, minutes, seconds)
            } else {
                format!("{}m {:02}s", minutes, seconds)
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

        project_lines.push(Line::from(vec![Span::styled(
            "└────────────────┴────────────────┘",
            Style::default().fg(Color::Gray),
        )]));

        let projects_widget = Paragraph::new(project_lines).alignment(Alignment::Center);
        frame.render_widget(projects_widget, chunks[1]);

        let footer = Paragraph::new("press [p] to return...")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }
}
