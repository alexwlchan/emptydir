use std::fs;
use std::path::Path;

use walkdir::WalkDir;

/// Recurse through a given root directory, and delete any "empty" directories.
///
/// Returns the number of directories deleted.
///
pub fn emptydir(root: &Path) -> u32 {
    let iterator = WalkDir::new(root)
        .contents_first(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter(|e| crate::can_be_deleted::can_be_deleted(e.path()));

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
