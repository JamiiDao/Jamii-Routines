use std::str::FromStr;

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use crate::RoutinesResult;

#[derive(Clone)]
pub struct AppDb {
    pub db: SqlitePool,
}

impl AppDb {
    pub async fn init(path: &str) -> RoutinesResult<Self> {
        let path = "sqlite://".to_string() + path;

        println!("sqlite: {path}");

        let options = SqliteConnectOptions::from_str(&path)
            .unwrap()
            .create_if_missing(true);

        let db = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        Ok(Self { db })
    }

    pub async fn create_tables_if_missing(&self) -> RoutinesResult<()> {
        sqlx::query(
            r#"
               CREATE TABLE IF NOT EXISTS users (
                    name TEXT PRIMARY KEY,
                    passkey String,
                    create_time TEXT NOT NULL,
                    email_verified INTEGER NOT NULL DEFAULT 0,
                    
                    email_auth_code TEXT,
                    email_auth_time BLOB
                );

                CREATE TABLE IF NOT EXISTS sessions (
                    token TEXT PRIMARY KEY,
                    user_name TEXT NOT NULL,
                    created_at BLOB NOT NULL,

                    FOREIGN KEY(user_name) REFERENCES users(name) ON DELETE CASCADE
                );
                "#,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
