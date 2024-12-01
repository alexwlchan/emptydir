use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::Path;

/// Return the names of files/folders inside a directory.
///
/// Names are lowercased for easy comparisons.
///
fn get_names_in_directory(dir: &Path) -> io::Result<HashSet<OsString>> {
    let mut names = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        names.push(entry.file_name().to_ascii_lowercase());
    }

    Ok(HashSet::from_iter(names))
}

/// Returns True if this path any ancestor is a `.git` folder,
/// False otherwise.
pub fn is_in_git_folder(path: &Path) -> bool {
    path.ancestors()
        .any(|ancestor| ancestor.file_name().map_or(false, |name| name == ".git"))
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

    // Don't delete subfolders of a `.git` directory.
    //
    // For example, if you delete `.git/refs`, then Git can't detect
    // the Git directory any more.  Observe:
    //
    //     $ git init .
    //     Initialized empty Git repository in tmp.bTrs8ZaWjc/.git/
    //
    //     $ rm -rf .git/refs
    //
    //     $ git status
    //     fatal: not a git repository (or any of the parent directories): .git
    //
    // Skipping these folders is fine.
    if is_in_git_folder(path) {
        return false;
    }

    // This is the list of entries which I consider safe to delete.
    //
    // *  .DS_Store stores some folder attributes used for showing the folder
    //    in the Finder, which I don't need to keep
    // *  `.ipynb_checkpoints` is a folder used by Jupyter Notebooks, but not
    //    important if I've deleted the notebooks
    // *  `.jekyll-cache` is a cache directory used by Jekyll sites, but
    //    can be easily regenerated and will be rebuilt regularly as part
    //    of the Jekyll build process
    // *  `.venv` is the name I use for virtual environments, which I can
    //    easily regenerate if necessary
    // *  `__pycache__` is the bytecode cache in Python projects, which is
    //    pointless if the original Python files have been removed
    // *  `Thumbs.db` is a file that contains thumbnails on Windows systems
    //
    // A directory is safe to delete if the ONLY things it contains are these entries;
    // any other entry should block the directory from being deleted.
    //
    let deletable_names = HashSet::from([
        OsString::from(".ds_store"),
        OsString::from(".ipynb_checkpoints"),
        OsString::from(".jekyll-cache"),
        OsString::from(".venv"),
        OsString::from("__pycache__"),
        OsString::from("desktop.ini"),
        OsString::from("thumbs.db"),
    ]);

    match get_names_in_directory(path) {
        Ok(names) if names.is_empty() => true,
        Ok(names) => names.is_subset(&deletable_names),
        Err(_) => false,
    }
}

#[cfg(test)]
mod test_can_be_deleted {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;

    fn test_dir() -> PathBuf {
        let tmp_dir = tempfile::tempdir().unwrap();
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

    #[test]
    fn the_dot_git_folder_cannot_be_deleted() {
        let path = test_dir();
        let git_dir = path.join(".git");

        create_dir(&git_dir);

        assert_eq!(can_be_deleted(&git_dir), false);
    }

    #[test]
    fn any_subdir_of_the_dot_git_folder_cannot_be_deleted() {
        let path = test_dir();
        let refs_dir = path.join(".git/refs");

        create_dir(&refs_dir);

        assert_eq!(can_be_deleted(&refs_dir), false);
    }
}
