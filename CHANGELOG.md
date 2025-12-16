# Changelog

## v1.3.0 - 2025-12-16

If `emptydir` looks at a directory but there's a reason the directory can't be deleted, it now prints the reason.

Example:

```console
$ emptydir ~/Desktop
directory is not empty; contains 3 entries:
  - makeup-tips.html
  - paste_images.py
  - Screenshot 2024-12-31 at 10.46.41.png
```

Previously, this would simply report "no empty directories found".

This reason is only printed for the initial target of `emptydir`, if nothing can be deleted.

## v1.2.2 - 2025-08-16

If `emptydir` tries to delete a directory but gets an error, it now prints that error to stderr. Previously the error would be silently ignored.

This fixes a bug where emptydir could appear to do nothing -- it would report "no empty directories found", but actually it had found empty directories it was unable to delete.

## v1.2.1 - 2024-12-01

Don't delete the `.git` directory or any subdirectories.

Messing with the internal structure `.git` can cause issues for Git, so just leave it as-is, even if it contains empty folders.

## v1.2.0 - 2024-08-21

Delete empty parent directories.

If the target directory is the only entry in an otherwise empty directory, then the parent directory will also be deleted (and emptydir will keep going through parent directories until it finds one which is non-empty).

## v1.1.3 - 2024-08-21

Delete empty folders which only contain a `.jekyll-cache` folder.

## v1.1.2 - 2024-08-16

Delete empty folders which only contain an `.ipynb_checkpoints` folder.

## v1.1.1 - 2024-07-27

Delete empty folders which only contain a `desktop.ini` file.

## v1.1.0 - 2024-07-27

Delete empty folders which only contain [a `Thumbs.db` file](https://en.wikipedia.org/wiki/Windows_thumbnail_cache#Thumbs.db).

## v1.0.0 - 2024-06-18

Initial release.
