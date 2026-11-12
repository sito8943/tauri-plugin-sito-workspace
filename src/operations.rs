use std::fs;

use crate::models::{
  CreateEntryRequest, RenameEntryRequest, TrashEntryRequest, WorkspaceEntry,
};
use crate::workspace::{entry_from_path, workspace_path};
use crate::{Error, Result};

/// Fails with `already_exists` when anything (even a broken symlink) sits at `path`.
fn ensure_free(path: &std::path::Path, display_path: &str) -> Result<()> {
  if fs::symlink_metadata(path).is_ok() {
    return Err(Error::already_exists(display_path));
  }
  Ok(())
}

/// Creates an empty file. Never overwrites: an existing entry fails with `already_exists`.
pub fn create_file(request: CreateEntryRequest) -> Result<WorkspaceEntry> {
  let path = workspace_path(request.path)?;
  let display_path = path.to_string_lossy().into_owned();
  fs::OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(&path)
    .map_err(|error| Error::from_io(&display_path, error))?;
  entry_from_path(&path)
}

/// Creates one directory (the parent must exist). An existing entry fails with `already_exists`.
pub fn create_directory(request: CreateEntryRequest) -> Result<WorkspaceEntry> {
  let path = workspace_path(request.path)?;
  let display_path = path.to_string_lossy().into_owned();
  fs::create_dir(&path).map_err(|error| Error::from_io(&display_path, error))?;
  entry_from_path(&path)
}

/// Renames or moves an entry. Never overwrites: an existing target fails with `already_exists`.
pub fn rename_entry(request: RenameEntryRequest) -> Result<WorkspaceEntry> {
  let from = workspace_path(request.from)?;
  let to = workspace_path(request.to)?;
  let display_from = from.to_string_lossy().into_owned();
  let display_to = to.to_string_lossy().into_owned();

  fs::symlink_metadata(&from).map_err(|error| Error::from_io(&display_from, error))?;
  // A case-only rename on a case-insensitive disk points at the same entry: allow it.
  let same_entry = match (fs::canonicalize(&from), fs::canonicalize(&to)) {
    (Ok(left), Ok(right)) => left == right,
    _ => false,
  };
  if !same_entry {
    ensure_free(&to, &display_to)?;
  }

  fs::rename(&from, &to).map_err(|error| Error::from_io(&display_from, error))?;
  entry_from_path(&to)
}

/// Moves an entry to the system trash (recoverable), never deleting it permanently.
pub fn trash_entry(request: TrashEntryRequest) -> Result<()> {
  let path = workspace_path(request.path)?;
  let display_path = path.to_string_lossy().into_owned();
  fs::symlink_metadata(&path).map_err(|error| Error::from_io(&display_path, error))?;

  #[cfg(target_os = "macos")]
  let result = {
    // NSFileManager avoids Finder's AppleScript (and its permission prompt).
    use trash::macos::{DeleteMethod, TrashContextExtMacos};
    let mut context = trash::TrashContext::default();
    context.set_delete_method(DeleteMethod::NsFileManager);
    context.delete(&path)
  };
  #[cfg(not(target_os = "macos"))]
  let result = trash::delete(&path);

  result.map_err(|error| Error::trash_failed(&display_path, error.to_string()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ErrorCode;

  fn path_string(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
  }

  #[test]
  fn creates_files_and_directories_without_overwriting() {
    let directory = tempfile::tempdir().expect("creates a temporary directory");
    let file = directory.path().join("new.txt");
    let folder = directory.path().join("folder");

    let entry = create_file(CreateEntryRequest {
      path: path_string(&file),
    })
    .expect("creates the file");
    assert_eq!(entry.name, "new.txt");
    assert_eq!(fs::read_to_string(&file).expect("reads the file"), "");

    fs::write(&file, "keep").expect("writes content");
    let error = create_file(CreateEntryRequest {
      path: path_string(&file),
    })
    .expect_err("refuses to overwrite");
    assert_eq!(error.code, ErrorCode::AlreadyExists);
    assert_eq!(fs::read_to_string(&file).expect("reads the file"), "keep");

    create_directory(CreateEntryRequest {
      path: path_string(&folder),
    })
    .expect("creates the directory");
    assert!(folder.is_dir());
    let error = create_directory(CreateEntryRequest {
      path: path_string(&folder),
    })
    .expect_err("refuses an existing directory");
    assert_eq!(error.code, ErrorCode::AlreadyExists);
  }

  #[test]
  fn renames_without_overwriting_the_target() {
    let directory = tempfile::tempdir().expect("creates a temporary directory");
    let from = directory.path().join("a.txt");
    let to = directory.path().join("b.txt");
    fs::write(&from, "a").expect("writes a");
    fs::write(&to, "b").expect("writes b");

    let error = rename_entry(RenameEntryRequest {
      from: path_string(&from),
      to: path_string(&to),
    })
    .expect_err("refuses an existing target");
    assert_eq!(error.code, ErrorCode::AlreadyExists);
    assert_eq!(fs::read_to_string(&to).expect("reads b"), "b");

    let renamed = directory.path().join("c.txt");
    let entry = rename_entry(RenameEntryRequest {
      from: path_string(&from),
      to: path_string(&renamed),
    })
    .expect("renames");
    assert_eq!(entry.name, "c.txt");
    assert!(!from.exists());
    assert_eq!(fs::read_to_string(&renamed).expect("reads c"), "a");
  }

  #[test]
  fn reports_a_missing_source() {
    let directory = tempfile::tempdir().expect("creates a temporary directory");
    let error = rename_entry(RenameEntryRequest {
      from: path_string(&directory.path().join("missing")),
      to: path_string(&directory.path().join("other")),
    })
    .expect_err("fails");
    assert_eq!(error.code, ErrorCode::NotFound);
  }
}
