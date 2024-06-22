use anyhow::Result;
use clap::Parser;
use rusqlite::Connection;
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(short, long)]
    command: Option<String>,

    #[clap(short, long)]
    timestamp: Option<u128>,

    #[clap(short, long, value_name = "DBPATH")]
    database: Option<PathBuf>,

    #[clap(short, long, value_name = "FILEPATH")]
    migrate: Option<PathBuf>,

    #[clap(long, short, action)]
    unique: bool,

    #[clap(long = "top", short = 'x', value_name = "NUM")]
    topten: Option<u8>,
}

fn main() {
    let cli = Cli::parse();

    let home_dir = env::var("HOMEPATH").expect("No Home directory");
    let mut default_db_path = PathBuf::from(&home_dir);
    default_db_path.push(".xhdb");

    let db_path = cli.database.unwrap_or(default_db_path);

    let connection = initialize_database(db_path).expect("If no DB no function.");

    let timestamp = cli
        .timestamp
        .unwrap_or(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("We do be f'd.")
                .as_millis(),
        )
        .try_into()
        .expect("What could possibly go wrong.");

    if let Some(command) = cli.command.as_deref() {
        save_command_to_database(&connection, command, &timestamp)
            .expect("Couldn't insert command into database.");
    }

    if let Some(file_path) = cli.migrate {
        migrate_from_file(&connection, file_path, &timestamp).unwrap();
    }

    if cli.unique {
        let commands = get_unique_commands(&connection).expect("Couldn't get unique commands.");
        let mut stdout = std::io::stdout().lock();
        for cmd in commands {
            writeln!(stdout, "{cmd}").unwrap();
        }
        stdout.flush().unwrap();
    }
}

fn initialize_database(path: PathBuf) -> Result<Connection, anyhow::Error> {
    let connection = Connection::open(path)?;
    connection.execute(
        "CREATE TABLE IF NOT EXISTS commands(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            command TEXT,
            timestamp_ms INTEGER
        )",
        (),
    )?;
    Ok(connection)
}

fn save_command_to_database(
    connection: &Connection,
    command: &str,
    timestamp: &u64,
) -> Result<(), anyhow::Error> {
    connection.execute(
        "INSERT INTO commands (command, timestamp_ms)
        VALUES (?1, ?2)",
        (command, timestamp),
    )?;
    Ok(())
}

fn get_unique_commands(connection: &Connection) -> Result<Vec<String>> {
    let mut values = connection.prepare("SELECT DISTINCT(TRIM(LTRIM(command))) FROM commands")?;
    let rows = values.query_map([], |row| row.get(0))?;
    let mut commands: Vec<String> = Vec::new();
    for command in rows {
        commands.push(command?);
    }
    Ok(commands)
}

fn migrate_from_file(connection: &Connection, file: PathBuf, timestamp: &u64) -> Result<()> {
    let file = File::open(file)?;
    for line in BufReader::new(file).lines() {
        match line {
            Ok(line) => {
                save_command_to_database(connection, line.as_str(), timestamp).unwrap();
            }
            Err(e) => eprintln!("Error reading line: {e}"),
        }
    }
    Ok(())
}
