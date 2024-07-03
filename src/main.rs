use anyhow::Result;
use clap::Parser;
use rusqlite::Connection;
use std::{
    env,
    fmt::Display,
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

    #[clap(long, short, action)]
    all: bool,

    #[clap(long = "top", short = 'x', value_name = "NUM")]
    topten: Option<Option<u8>>,
}

struct TopCmd {
    count: u32,
    command: String,
}

impl Display for TopCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.count, self.command)
    }
}

fn main() {
    let cli = Cli::parse();

    let home_dir = env::var("HOMEPATH").expect("No Home directory");
    let mut default_db_path = PathBuf::from(&home_dir);
    default_db_path.push(".xhdb");

    // let database = Database;
    // let connection = database;

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

    if cli.all {
        let commands = get_all_commands(&connection).expect("Couldn't get all commands.");
        let mut stdout = std::io::stdout().lock();
        for cmd in commands {
            writeln!(stdout, "{cmd}").unwrap();
        }
        stdout.flush().unwrap();
    }

    if let Some(top) = cli.topten {
        let num = top.unwrap_or(10);
        let commands = get_top_commands(&connection, num).unwrap();

        let mut stdout = std::io::stdout().lock();

        for (idx, cmd) in commands.iter().enumerate() {
            let index = idx + 1;
            writeln!(stdout, "{index}.\t{cmd}").unwrap();
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

fn get_all_commands(connection: &Connection) -> Result<Vec<String>> {
    let mut values =
        connection.prepare("SELECT command FROM commands ORDER BY timestamp_ms DESC")?;
    let rows = values.query_map([], |row| row.get(0))?;
    let mut commands: Vec<String> = Vec::new();
    for command in rows {
        commands.push(command?);
    }
    Ok(commands)
}

fn get_unique_commands(connection: &Connection) -> Result<Vec<String>> {
    let mut values = connection.prepare(
        "SELECT DISTINCT(TRIM(LTRIM(command))) FROM commands ORDER BY timestamp_ms DESC",
    )?;
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

fn get_top_commands(connection: &Connection, number: u8) -> Result<Vec<TopCmd>> {
    let mut stmt = connection.prepare(
        "
        SELECT count(command),
            TRIM(command, '\n')
        FROM commands
        GROUP by command
        ORDER by count(command) DESC
        LIMIT ?
        ",
    )?;

    let mut rows = stmt.query([number])?;

    let mut top = Vec::new();
    while let Some(row) = rows.next()? {
        top.push(TopCmd {
            count: row.get(0)?,
            command: row.get(1)?,
        });
    }
    Ok(top)
}
