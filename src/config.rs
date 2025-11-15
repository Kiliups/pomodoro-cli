use sqlx::FromRow;
use sqlx::sqlite::SqlitePool;

#[derive(Debug, FromRow)]
pub struct Config {
    // todo not pub
    id: i64,
    pub focus: u32,
    #[sqlx(rename = "break")]
    pub break_time: u32,
    pub long_break: u32,
    pub cycles: u32,
}

impl Config {
    pub fn new(focus: u32, break_time: u32, long_break: u32, cycles: u32) -> Self {
        Self {
            id: 1,
            focus,
            break_time,
            long_break,
            cycles,
        }
    }

    pub async fn create(pool: &SqlitePool) -> Result<(), sqlx::Error> {
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
        .execute(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM config")
            .fetch_one(pool)
            .await?;

        if count.0 == 0 {
            let config = Config::new(25, 5, 15, 4);
            config.insert(pool).await?;
        }

        Ok(())
    }

    pub async fn get(pool: &SqlitePool) -> Result<Config, sqlx::Error> {
        let config: Config = sqlx::query_as::<_, Config>(
            "SELECT id, focus, break, long_break, cycles FROM config WHERE id = ?",
        )
        .bind(1)
        .fetch_one(pool)
        .await?;
        Ok(config)
    }

    pub async fn insert(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO config (focus, break, long_break, cycles) VALUES (?, ?, ?, ?)")
            .bind(self.focus)
            .bind(self.break_time)
            .bind(self.long_break)
            .bind(self.cycles)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE config SET focus = ?, break=?, long_break=?, cycles=? WHERE id = ?")
            .bind(self.focus)
            .bind(self.break_time)
            .bind(self.long_break)
            .bind(self.cycles)
            .bind(self.id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
