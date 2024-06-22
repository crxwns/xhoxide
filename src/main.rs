use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(short, long)]
    command: Option<String>,

    #[clap(short, long)]
    timestamp: Option<u32>,

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

    if let Some(command) = cli.command.as_deref() {
        println!("Go a new command: {command}");
    }

    // You can check the value provided by positional arguments, or option arguments
    // if let Some(name) = cli.name.as_deref() {
    //     println!("Value for name: {name}");
    // }

    // if let Some(config_path) = cli.config.as_deref() {
    //     println!("Value for config: {}", config_path.display());
    // }

    // // You can see how many times a particular flag or argument occurred
    // // Note, only flags can have multiple occurrences
    // match cli.debug {
    //     0 => println!("Debug mode is off"),
    //     1 => println!("Debug mode is kind of on"),
    //     2 => println!("Debug mode is on"),
    //     _ => println!("Don't be crazy"),
    // }

    // // You can check for the existence of subcommands, and if found use their
    // // matches just as you would the top level cmd
    // match &cli.command {
    //     Some(Commands::Test { list }) => {
    //         if *list {
    //             println!("Printing testing lists...");
    //         } else {
    //             println!("Not printing testing lists...");
    //         }
    //     }
    //     None => {}
    // }

    // Continued program logic goes here...
}
