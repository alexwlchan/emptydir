#![deny(warnings)]

use std::fs;
use std::path::Path;

use colored::*;
use walkdir::WalkDir;

mod can_be_deleted;

/// Recurse through a given root directory, and delete any "empty" directories.
///
/// Returns the number of directories deleted.
///
fn emptydir(root: &Path) -> u32 {
    let iterator = WalkDir::new(root)
        .contents_first(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter(|e| can_be_deleted::can_be_deleted(e.path()));

    let mut count_deleted: u32 = 0;

    for entry in iterator {
        match fs::remove_dir_all(entry.path()) {
            Ok(_) => {
                println!("{}", entry.path().display());
                count_deleted += 1;
            }
            _ => (),
        };
    }

    count_deleted
}

fn main() -> Result<(), std::io::Error> {
    let count_deleted = emptydir(Path::new("."));

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
