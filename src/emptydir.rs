use std::fs;
use std::path::Path;

use colored::*;
use walkdir::WalkDir;

#[derive(Debug, PartialEq)]
pub struct EmptydirResult {
    pub count_deleted: u32,
    pub count_errors: u32,
}

/// Recurse through a given root directory, and delete any "empty" directories.
///
/// Returns the number of directories deleted.
///
pub fn emptydir(root: &Path) -> EmptydirResult {
    let directories_to_delete = WalkDir::new(root)
        .contents_first(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter(|e| crate::can_be_deleted::can_be_deleted(e.path()));

    let mut count_deleted: u32 = 0;
    let mut count_errors: u32 = 0;

    for dir in directories_to_delete {
        match fs::remove_dir_all(dir.path()) {
            Ok(_) => {
                println!("{}", dir.path().display());
                count_deleted += 1;
            }
            Err(e) => {
                let message = format!(
                    "Tried to delete {}, but got error: {}",
                    dir.path().display(),
                    e
                );
                eprintln!("{}", message.red());
                count_errors += 1;
            }
        };
    }

    // Now work our way upward through the parent directories, and
    // delete any of those which are empty.
    let mut current_parent = root.parent();

    while let Some(parent) = current_parent {
        if !crate::can_be_deleted::can_be_deleted(parent) {
            break;
        }

        match fs::remove_dir_all(parent) {
            Ok(_) => {
                println!("{}", parent.display());
                count_deleted += 1;
            }
            Err(e) => {
                let message = format!("Tried to delete {}, but got error: {}", parent.display(), e);
                eprintln!("{}", message.red());
                count_errors += 1;
            }
        };

        current_parent = parent.parent();
    }

    EmptydirResult {
        count_deleted,
        count_errors,
    }
}

#[cfg(test)]
mod test_emptydir {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;

    fn test_dir() -> PathBuf {
        let tmp_dir = tempfile::tempdir().unwrap();
        let path = tmp_dir.path();
        path.to_owned()
    }

    fn create_dir(dir: &PathBuf) {
        fs::create_dir_all(dir).unwrap();
    }

    fn create_file(path: &PathBuf) {
        create_dir(&path.parent().unwrap().to_path_buf());
        fs::write(&path, "this file is for testing").unwrap();
    }

    #[test]
    fn it_doesnt_delete_my_do_not_backup() {
        let dir = Path::new("/Users/alexwlchan/Desktop/do not back up");
        assert_eq!(
            emptydir(dir),
            EmptydirResult {
                count_deleted: 0,
                count_errors: 0
            }
        );
    }

    #[test]
    fn it_doesnt_delete_a_non_existent_directory() {
        let dir = Path::new("/does/not/exist");
        assert_eq!(
            emptydir(dir),
            EmptydirResult {
                count_deleted: 0,
                count_errors: 0
            }
        );
    }

    #[test]
    fn it_deletes_an_empty_dir() {
        let dir = test_dir();

        // Create the directory, but don't put anything in it
        create_dir(&dir);

        assert_eq!(
            emptydir(&dir),
            EmptydirResult {
                count_deleted: 1,
                count_errors: 0
            }
        );
        assert_eq!(dir.exists(), false);
    }

    #[test]
    fn it_ignores_a_dir_with_extra_entries() {
        let dir = test_dir();

        // Create the directory, then add a text file
        create_dir(&dir);

        create_file(&dir.join("greeting.txt"));

        assert_eq!(
            emptydir(&dir),
            EmptydirResult {
                count_deleted: 0,
                count_errors: 0
            }
        );
        assert_eq!(dir.exists(), true);
        assert_eq!(dir.join("greeting.txt").exists(), true);
    }

    #[test]
    fn it_deletes_a_dir_with_only_safe_to_delete_entries() {
        let dir = test_dir();

        //    .
        //    ├─ .ipynb_checkpoints/
        //    │   └─ analysis-checkpoint.ipynb
        //    │
        //    ├─ .venv/
        //    │   └─ bin/
        //    │       └─ mypython.py
        //    │
        //    ├─ __pycache__
        //    │   └─ myfile.pyc
        //    │
        //    └─ .DS_Store
        //
        create_dir(&dir);

        create_dir(&dir.join(".venv"));
        create_file(&dir.join(".venv/bin/mypython.py"));

        create_dir(&dir.join(".ipynb_checkpoints"));
        create_file(&dir.join(".ipynb_checkpoints/analysis-checkpoint.ipynb"));

        create_dir(&dir.join("__pycache__"));
        create_file(&dir.join("__pycache__/myfile.pyc"));

        create_file(&dir.join(".DS_Store"));

        assert_eq!(
            emptydir(&dir),
            EmptydirResult {
                count_deleted: 1,
                count_errors: 0
            }
        );
        assert_eq!(dir.exists(), false);
    }

    #[test]
    fn it_ignores_a_dir_with_a_mix_of_safe_and_unsafe_entries() {
        let dir = test_dir();

        create_dir(&dir);

        create_file(&dir.join(".DS_Store"));
        create_file(&dir.join("greeting.txt"));

        assert_eq!(
            emptydir(&dir),
            EmptydirResult {
                count_deleted: 0,
                count_errors: 0
            }
        );
        assert!(dir.exists());
        assert!(dir.join("greeting.txt").exists());
    }

    #[test]
    fn it_deletes_a_subdir_with_only_safe_to_delete_entries() {
        let dir = test_dir();
        let subdir = dir.join("subdir");

        //    .
        //    ├─ subdir/
        //    │   ├─ .venv/
        //    │   │   └─ bin/
        //    │   │       └─ mypython.py
        //    │   │
        //    │   ├─ __pycache__
        //    │   │   └─ myfile.pyc
        //    │   │
        //    │   └─ .DS_Store
        //    │
        //    └─ greeting.txt
        //
        create_dir(&subdir);

        create_dir(&subdir.join(".venv"));
        create_file(&subdir.join(".venv/bin/mypython.py"));

        create_dir(&subdir.join("__pycache__"));
        create_file(&subdir.join("__pycache__/myfile.pyc"));

        create_file(&subdir.join(".DS_Store"));

        create_file(&dir.join("greeting.txt"));

        assert_eq!(
            emptydir(&dir),
            EmptydirResult {
                count_deleted: 1,
                count_errors: 0
            }
        );
        assert_eq!(dir.exists(), true);
        assert_eq!(subdir.exists(), false);
        assert!(dir.join("greeting.txt").exists());
    }
}
