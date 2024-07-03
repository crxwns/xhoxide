use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use rusqlite::Connection;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn initialize_database(mut self, path: PathBuf) -> Result<()> {
        self.connection = Connection::open(path)?;
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS commands(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            command TEXT,
            timestamp_ms INTEGER
        )",
            (),
        )?;
        Ok(())
    }

    pub fn save_command_to_database(&self, command: &str) -> Result<()> {
        let timestamp: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("We do be f'd.")
            .as_millis()
            .try_into()
            .expect("What could possibly go wrong.");

        self.connection.execute(
            "INSERT INTO commands (command, timestamp_ms)
            VALUES (?1, ?2)",
            (command, timestamp),
        )?;
        Ok(())
    }
}
