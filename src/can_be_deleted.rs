use std::collections::HashSet;
use std::ffi::OsString;
use std::fmt;
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
fn is_in_git_repository(path: &Path) -> bool {
    path.ancestors()
        .any(|ancestor| ancestor.file_name().map_or(false, |name| name == ".git"))
}

/// DeleteDecision describes whether a directory can be deleted.
#[derive(Debug)]
pub enum DeleteDecision {
    CanDelete,
    CannotDelete(Reason),
}

// Reason explains why a directory cannot be deleted.
#[derive(Debug)]
pub enum Reason {
    NotEmpty(Vec<OsString>),
    InGitRepository,
    CannotListContents(io::Error),
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reason::NotEmpty(entries) => {
                write!(
                    f,
                    "directory is not empty; contains {} entr{}:",
                    entries.len(),
                    if entries.len() == 1 { "y" } else { "ies" }
                )?;

                // Sort the entries for consistent output.
                let mut sorted: Vec<_> = entries.iter().collect();
                sorted.sort_by_key(|s| s.to_string_lossy());

                for entry in sorted {
                    write!(f, "\n  - {}", entry.to_string_lossy())?;
                }

                Ok(())
            }

            Reason::CannotListContents(err) => {
                write!(f, "unable to list directory contents: {}", err)
            }

            Reason::InGitRepository => {
                write!(f, "directory is inside a .git repository")
            }
        }
    }
}

/// can_be_deleted checks whether a directory can be deleted.
pub fn can_be_deleted(dir_path: &Path) -> DeleteDecision {
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
    if is_in_git_repository(dir_path) {
        return DeleteDecision::CannotDelete(Reason::InGitRepository);
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

    match get_names_in_directory(dir_path) {
        Ok(names) if names.is_subset(&deletable_names) => DeleteDecision::CanDelete,
        Ok(names) => {
            let remaining_entries: Vec<OsString> =
                names.difference(&deletable_names).cloned().collect();
            DeleteDecision::CannotDelete(Reason::NotEmpty(remaining_entries))
        }
        Err(e) => DeleteDecision::CannotDelete(Reason::CannotListContents(e)),
    }
}

#[cfg(test)]
mod test_can_be_deleted {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;

    fn test_dir() -> PathBuf {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir_path = tmp_dir.path();
        dir_path.to_owned()
    }

    fn create_dir(path: &PathBuf) {
        fs::create_dir_all(path).unwrap();
    }

    fn create_file(path: PathBuf) {
        fs::write(&path, "this file is for testing").unwrap();
    }

    #[test]
    fn a_dir_cant_be_deleted_if_we_cant_read_the_contents() {
        let dir_path = Path::new("/does/not/exist");
        assert!(matches!(
            can_be_deleted(&dir_path),
            DeleteDecision::CannotDelete(Reason::CannotListContents(_))
        ));
    }

    #[test]
    fn an_empty_dir_can_be_deleted() {
        let dir_path = test_dir();

        // Create the directory, but don't put anything in it
        create_dir(&dir_path);

        assert!(matches!(
            can_be_deleted(&dir_path),
            DeleteDecision::CanDelete
        ));
    }

    #[test]
    fn a_directory_with_extra_entries_cannot_be_deleted() {
        let dir_path = test_dir();

        // Create the directory, then add a text file
        create_dir(&dir_path);

        create_file(dir_path.join("greeting.txt"));

        match can_be_deleted(&dir_path) {
            DeleteDecision::CannotDelete(Reason::NotEmpty(entries)) => {
                assert_eq!(entries, vec![OsString::from("greeting.txt")]);
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn a_directory_with_only_safe_to_delete_entries_can_be_deleted() {
        let dir_path = test_dir();

        // Create the directory, then add subdirectories
        create_dir(&dir_path);

        create_dir(&dir_path.join(".venv"));
        create_dir(&dir_path.join("__pycache__"));
        create_file(dir_path.join(".DS_Store"));

        assert!(matches!(
            can_be_deleted(&dir_path),
            DeleteDecision::CanDelete
        ));
    }

    #[test]
    fn a_directory_with_mix_of_safe_and_unsafe_entries_cannot_be_deleted() {
        let dir_path = test_dir();

        create_dir(&dir_path);

        create_file(dir_path.join(".DS_Store"));
        create_file(dir_path.join("greeting.txt"));

        match can_be_deleted(&dir_path) {
            DeleteDecision::CannotDelete(Reason::NotEmpty(entries)) => {
                // `.DS_Store` is allowed, `greeting.txt` is not
                assert_eq!(entries, vec![OsString::from("greeting.txt")]);
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn safe_to_delete_entries_are_case_insensitive() {
        let dir_path = test_dir();

        create_dir(&dir_path);

        create_file(dir_path.join(".ds_store"));

        assert!(matches!(
            can_be_deleted(&dir_path),
            DeleteDecision::CanDelete
        ));
    }

    #[test]
    fn the_dot_git_folder_cannot_be_deleted() {
        let dir_path = test_dir();
        let git_dir = dir_path.join(".git");

        create_dir(&git_dir);

        assert!(matches!(
            can_be_deleted(&git_dir),
            DeleteDecision::CannotDelete(Reason::InGitRepository)
        ));
    }

    #[test]
    fn any_subdir_of_the_dot_git_folder_cannot_be_deleted() {
        let dir_path = test_dir();
        let refs_dir = dir_path.join(".git/refs");

        create_dir(&refs_dir);

        assert!(matches!(
            can_be_deleted(&refs_dir),
            DeleteDecision::CannotDelete(Reason::InGitRepository)
        ));
    }
}
