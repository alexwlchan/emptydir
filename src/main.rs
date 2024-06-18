#![deny(warnings)]

use std::path::Path;

use clap::Parser;
use colored::*;

mod can_be_deleted;
mod emptydir;

#[derive(Parser)]
#[command(version, about = "Look for empty directories and delete them", long_about = None)]
struct Cli {
    /// Path to the directory to inspect
    #[arg(default_value_t = String::from("."))]
    root: String,
}

fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();

    let root = Path::new(&cli.root);
    let count_deleted = emptydir::emptydir(root);

    match count_deleted {
        0 => println!("{}", "No empty directories found".blue()),
        1 => println!("{}", "1 directory deleted".green()),
        _ => {
            let message = format!("{} directories deleted", count_deleted);
            println!("{}", message.green());
        }
    }

    Ok(())
}
