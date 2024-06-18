#![deny(warnings)]

use colored::*;
use walkdir::WalkDir;

mod can_be_deleted;

fn main() -> Result<(), std::io::Error> {
    let iterator = WalkDir::new(".")
        .contents_first(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter(|e| can_be_deleted::can_be_deleted(e.path()));

    let mut count_deleted = 0;

    for entry in iterator {
        println!("{}", entry.path().display());
        count_deleted += 1;
    }

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
