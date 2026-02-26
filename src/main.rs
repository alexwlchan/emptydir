#![deny(warnings)]

use std::fs;
use std::path::Path;
use std::process;

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

    let root = fs::canonicalize(Path::new(&cli.root))?;
    let result = emptydir::emptydir(&root);

    match (result.count_deleted, result.count_errors) {
        (0, 0) => match can_be_deleted::can_be_deleted(&root) {
            can_be_deleted::DeleteDecision::CannotDelete(reason) => {
                eprintln!("{}", reason.to_string().red());
                process::exit(1);
            }
            _ => (),
        },
        (0, _) => {
            println!("{}", "Unable to delete empty directories".red());
            process::exit(1);
        }
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

#[cfg(test)]
mod test_emptydir {
    use std::fs;
    use std::path::PathBuf;

    use assert_cmd::Command;
    use predicates::prelude::*;

    fn test_dir() -> PathBuf {
        let tmp_dir = tempfile::tempdir().unwrap();
        let path = tmp_dir.path();
        path.to_owned()
    }

    fn create_dir(dir: &PathBuf) {
        fs::create_dir_all(dir).unwrap();
    }

    #[expect(
        deprecated,
        reason = "cargo_bin is deprecated, cargo_bin! is not, `use` does not differenciate them"
    )]
    #[test]
    fn it_prints_the_version() {
        // Match strings like `emptydir 1.2.3`
        let is_version_string =
            predicate::str::is_match(r"^emptydir [0-9]+\.[0-9]+\.[0-9]+\n$").unwrap();

        Command::cargo_bin("emptydir")
            .unwrap()
            .arg("--version")
            .assert()
            .success()
            .stdout(is_version_string)
            .stderr("");
    }

    #[expect(
        deprecated,
        reason = "cargo_bin is deprecated, cargo_bin! is not, `use` does not differenciate them"
    )]
    #[test]
    fn it_deletes_dot_directory() {
        let dir = test_dir();

        // Create the directory, but don't put anything in it
        create_dir(&dir);

        Command::cargo_bin("emptydir")
            .unwrap()
            .current_dir(&dir)
            .arg(".")
            .assert()
            .success();
        assert_eq!(dir.exists(), false);
    }
}
