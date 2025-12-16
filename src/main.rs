#![deny(warnings)]

use std::path::Path;

use clap::Parser;
use colored::*;
use num_format::{Locale, ToFormattedString};

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
    let result = emptydir::emptydir(root);

    match (result.count_deleted, result.count_errors) {
        (0, 0) => match can_be_deleted::can_be_deleted(&root) {
            can_be_deleted::DeleteDecision::CannotDelete(reason) => {
                eprintln!("{}", reason.to_string().red());
            }
            _ => (),
        },
        (0, _) => println!("{}", "Unable to delete empty directories".red()),
        (1, _) => println!("{}", "1 directory deleted".green()),
        _ => {
            let message = format!(
                "{} directories deleted",
                result.count_deleted.to_formatted_string(&Locale::en)
            );
            println!("{}", message.green());
        }
    }

    Ok(())
}
