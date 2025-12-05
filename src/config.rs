use sqlx::FromRow;
use sqlx::sqlite::SqlitePool;

#[derive(Debug, FromRow)]
pub struct Config {
    id: i64,
    focus: u32,
    #[sqlx(rename = "break")]
    break_time: u32,
    long_break: u32,
    cycles: u32,
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

    pub fn get_focus(&self) -> u32 {
        self.focus
    }

    pub fn get_break_time(&self) -> u32 {
        self.break_time
    }

    pub fn get_long_break(&self) -> u32 {
        self.long_break
    }

    pub fn get_cycles(&self) -> u32 {
        self.cycles
    }

    pub fn set_focus(&mut self, focus: u32) {
        self.focus = focus;
    }

    pub fn set_break_time(&mut self, break_time: u32) {
        self.break_time = break_time;
    }

    pub fn set_long_break(&mut self, long_break: u32) {
        self.long_break = long_break;
    }

    pub fn set_cycles(&mut self, cycles: u32) {
        self.cycles = cycles;
    }
}
