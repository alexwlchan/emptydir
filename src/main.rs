#![deny(warnings)]

use std::path::Path;

use colored::*;

mod can_be_deleted;
mod emptydir;

fn main() -> Result<(), std::io::Error> {
    let count_deleted = emptydir::emptydir(Path::new("."));

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
