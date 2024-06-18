use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;

/// Given a Result<DirEntry> from `fs::read_dir()`, try to get the
/// filename of the entry.
///
/// Filenames will be lowercased for easy comparisons.
///
fn file_name(dir_entry: io::Result<fs::DirEntry>) -> Option<String> {
    match dir_entry.map(|e| e.file_name().into_string()) {
        Ok(Ok(s)) => Some(s.to_lowercase()),
        _ => None,
    }
}

pub fn can_be_deleted(path: &Path) -> bool {
    // This is a folder where I put files that I explicitly don't want
    // to include in my backups.
    //
    // It may sometimes be empty, but I never want to delete it.
    // See https://overcast.fm/+R7DX9_W-Y/21:22 or my Obsidian note.
    match path.canonicalize() {
        Ok(p) if p == Path::new("/Users/alexwlchan/Desktop/do not back up") => return false,
        _ => (),
    };

    // This is the list of entries which I consider safe to delete.
    //
    // *  .DS_Store stores some folder attributes used for showing the folder
    //    in the Finder, which I don't need to keep
    // *  `__pycache__` is the bytecode cache in Python projects, which is
    //    pointless if the original Python files have been removed
    // *  `.venv` is the name I use for virtual environments, which I can
    //    easily regenerate if necessary
    //
    // A directory is safe to delete if the ONLY things it contains are these entries;
    // any other entry should block the directory from being deleted.
    //
    let deletable_entries: HashSet<Option<String>> = [".DS_Store", "__pycache__", ".venv"]
        .iter()
        .map(|&s| Some(s.to_lowercase().to_owned()))
        .collect();

    match fs::read_dir(path) {
        Ok(entries) => {
            let names: HashSet<Option<String>> = HashSet::from_iter(entries.map(|e| file_name(e)));

            names.is_subset(&deletable_entries)
        }
        _ => false,
    }
}

#[cfg(test)]
mod can_be_deleted_tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use crate::can_be_deleted::can_be_deleted;

    fn test_dir() -> PathBuf {
        let tmp_dir = tempdir::TempDir::new("testing").unwrap();
        let path = tmp_dir.path();
        path.to_owned()
    }

    fn create_dir(path: &PathBuf) {
        fs::create_dir_all(path).unwrap();
    }

    fn create_file(path: PathBuf) {
        fs::write(&path, "this file is for testing").unwrap();
    }

    #[test]
    fn it_doesnt_delete_my_do_not_backup() {
        let path = Path::new("/Users/alexwlchan/Desktop/do not back up");
        assert_eq!(can_be_deleted(&path), false);
    }

    #[test]
    fn a_dir_cant_be_deleted_if_we_cant_read_the_contents() {
        let path = Path::new("/does/not/exist");
        assert_eq!(can_be_deleted(&path), false);
    }

    #[test]
    fn an_empty_dir_can_be_deleted() {
        let path = test_dir();

        // Create the directory, but don't put anything in it
        create_dir(&path);

        assert_eq!(can_be_deleted(&path), true);
    }

    #[test]
    fn a_directory_with_extra_entries_cannot_be_deleted() {
        let path = test_dir();

        // Create the directory, then add a text file
        create_dir(&path);

        create_file(path.join("greeting.txt"));

        assert_eq!(can_be_deleted(&path), false);
    }

    #[test]
    fn a_directory_with_only_safe_to_delete_entries_can_be_deleted() {
        let path = test_dir();

        // Create the directory, then add subdirectories
        create_dir(&path);

        create_dir(&path.join(".venv"));
        create_dir(&path.join("__pycache__"));
        create_file(path.join(".DS_Store"));

        assert_eq!(can_be_deleted(&path), true);
    }

    #[test]
    fn a_directory_with_mix_of_safe_and_unsafe_entries_cannot_be_deleted() {
        let path = test_dir();

        create_dir(&path);

        create_file(path.join(".DS_Store"));
        create_file(path.join("greeting.txt"));

        assert_eq!(can_be_deleted(&path), false);
    }

    #[test]
    fn safe_to_delete_entries_are_case_insensitive() {
        let path = test_dir();

        create_dir(&path);

        create_file(path.join(".ds_store"));

        assert_eq!(can_be_deleted(&path), true);
    }
}
