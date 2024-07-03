use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use anyhow::Result;

use crate::db::Database;

pub fn migrate_from_file(db: &Database, file: PathBuf) -> Result<()> {
    let file = File::open(file)?;
    for line in BufReader::new(file).lines() {
        match line {
            Ok(line) => {
                db.save_command_to_database(line.as_str()).unwrap();
            }
            Err(e) => eprintln!("Error reading line: {e}"),
        }
    }
    Ok(())
}
