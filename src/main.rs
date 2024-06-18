#![deny(warnings)]

use walkdir::WalkDir;

mod can_be_deleted;

fn main() -> Result<(), std::io::Error> {
    let iterator = WalkDir::new(".")
        .contents_first(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter(|e| can_be_deleted::can_be_deleted(e.path()));

    for entry in iterator {
        println!("{}", entry.path().display());
    }

    Ok(())
}
