use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::models::{
  LineEnding, ListDirectoryRequest, ReadTextFileRequest, StatEntryRequest,
  TextDocumentSnapshot, WorkspaceEntry, WorkspaceEntryKind, WriteTextFileRequest,
};
use crate::{Error, Result};

pub(crate) fn workspace_path(raw: String) -> Result<PathBuf> {
  if raw.trim().is_empty() {
    return Err(Error::invalid_path(raw));
  }
  Ok(PathBuf::from(raw))
}

fn modified_at_millis(metadata: &fs::Metadata) -> Option<u64> {
  metadata
    .modified()
    .ok()?
    .duration_since(UNIX_EPOCH)
    .ok()
    .map(|duration| duration.as_millis() as u64)
}

fn document_version(content: &[u8]) -> String {
  blake3::hash(content).to_hex().to_string()
}

fn entry_kind(metadata: &fs::Metadata) -> WorkspaceEntryKind {
  if metadata.is_dir() {
    WorkspaceEntryKind::Directory
  } else if metadata.is_file() {
    WorkspaceEntryKind::File
  } else {
    WorkspaceEntryKind::Other
  }
}

fn entry_from_path(path: &Path) -> Result<WorkspaceEntry> {
  let display_path = path.to_string_lossy().into_owned();
  let symlink_metadata =
    fs::symlink_metadata(path).map_err(|error| Error::from_io(&display_path, error))?;
  let is_symlink = symlink_metadata.file_type().is_symlink();
  let target_metadata = if is_symlink {
    fs::metadata(path).ok()
  } else {
    None
  };
  let metadata = target_metadata.as_ref().unwrap_or(&symlink_metadata);
  let name = path
    .file_name()
    .map(|value| value.to_string_lossy().into_owned())
    .unwrap_or_else(|| display_path.clone());

  Ok(WorkspaceEntry {
    path: display_path,
    name,
    kind: entry_kind(metadata),
    is_symlink,
    readonly: metadata.permissions().readonly(),
    size: metadata.len(),
    modified_at: modified_at_millis(metadata),
  })
}

pub fn list_directory(request: ListDirectoryRequest) -> Result<Vec<WorkspaceEntry>> {
  let path = workspace_path(request.path)?;
  let display_path = path.to_string_lossy().into_owned();
  let metadata =
    fs::metadata(&path).map_err(|error| Error::from_io(&display_path, error))?;
  if !metadata.is_dir() {
    return Err(Error::not_directory(display_path));
  }

  let directory =
    fs::read_dir(&path).map_err(|error| Error::from_io(&display_path, error))?;
  let mut entries = directory
    .map(|entry| {
      let entry = entry.map_err(|error| Error::from_io(&display_path, error))?;
      entry_from_path(&entry.path())
    })
    .collect::<Result<Vec<_>>>()?;

  entries.sort_by(|left, right| {
    let left_directory = left.kind == WorkspaceEntryKind::Directory;
    let right_directory = right.kind == WorkspaceEntryKind::Directory;
    right_directory
      .cmp(&left_directory)
      .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
  });

  Ok(entries)
}

pub fn stat_entry(request: StatEntryRequest) -> Result<WorkspaceEntry> {
  let path = workspace_path(request.path)?;
  entry_from_path(&path)
}

fn detect_line_ending(content: &str) -> LineEnding {
  let bytes = content.as_bytes();
  let mut saw_crlf = false;
  let mut saw_lf = false;

  for (index, byte) in bytes.iter().enumerate() {
    if *byte != b'\n' {
      continue;
    }
    if index > 0 && bytes[index - 1] == b'\r' {
      saw_crlf = true;
    } else {
      saw_lf = true;
    }
  }

  if saw_crlf && saw_lf {
    LineEnding::Mixed
  } else if saw_crlf {
    LineEnding::CrLf
  } else {
    LineEnding::Lf
  }
}

pub fn read_text_file(request: ReadTextFileRequest) -> Result<TextDocumentSnapshot> {
  let path = workspace_path(request.path)?;
  let display_path = path.to_string_lossy().into_owned();
  let metadata =
    fs::metadata(&path).map_err(|error| Error::from_io(&display_path, error))?;
  if metadata.is_dir() {
    return Err(Error::is_directory(display_path));
  }

  let bytes = fs::read(&path).map_err(|error| Error::from_io(&display_path, error))?;
  let version = document_version(&bytes);
  let content =
    String::from_utf8(bytes).map_err(|_| Error::unsupported_encoding(&display_path))?;

  Ok(TextDocumentSnapshot {
    path: display_path,
    line_ending: detect_line_ending(&content),
    content,
    version,
    encoding: "utf-8".to_string(),
    readonly: metadata.permissions().readonly(),
    size: metadata.len(),
  })
}

fn writable_file_metadata(path: &Path, display_path: &str) -> Result<fs::Metadata> {
  let symlink_metadata =
    fs::symlink_metadata(path).map_err(|error| Error::from_io(display_path, error))?;
  if symlink_metadata.file_type().is_symlink() {
    return Err(Error::symlink_not_writable(display_path));
  }

  let metadata = fs::metadata(path).map_err(|error| Error::from_io(display_path, error))?;
  if metadata.is_dir() {
    return Err(Error::is_directory(display_path));
  }
  if !metadata.is_file() {
    return Err(Error::not_file(display_path));
  }
  if metadata.permissions().readonly() {
    return Err(Error::read_only(display_path));
  }

  Ok(metadata)
}

fn ensure_version(path: &Path, display_path: &str, expected_version: &str) -> Result<fs::Metadata> {
  let metadata = writable_file_metadata(path, display_path)?;
  let content = fs::read(path).map_err(|error| Error::from_io(display_path, error))?;
  if document_version(&content) != expected_version {
    return Err(Error::version_conflict(display_path));
  }
  Ok(metadata)
}

#[cfg(unix)]
fn sync_parent_directory(parent: &Path, display_path: &str) -> Result<()> {
  fs::File::open(parent)
    .and_then(|directory| directory.sync_all())
    .map_err(|error| Error::from_io(display_path, error))
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path, _display_path: &str) -> Result<()> {
  Ok(())
}

pub fn write_text_file(request: WriteTextFileRequest) -> Result<TextDocumentSnapshot> {
  let path = workspace_path(request.path)?;
  let display_path = path.to_string_lossy().into_owned();
  let metadata = ensure_version(&path, &display_path, &request.expected_version)?;
  let parent = path
    .parent()
    .filter(|value| !value.as_os_str().is_empty())
    .unwrap_or_else(|| Path::new("."));

  let mut temporary =
    tempfile::NamedTempFile::new_in(parent).map_err(|error| Error::from_io(&display_path, error))?;
  temporary
    .as_file()
    .set_permissions(metadata.permissions())
    .map_err(|error| Error::from_io(&display_path, error))?;
  temporary
    .write_all(request.content.as_bytes())
    .map_err(|error| Error::from_io(&display_path, error))?;
  temporary
    .as_file_mut()
    .sync_all()
    .map_err(|error| Error::from_io(&display_path, error))?;

  let latest_metadata = ensure_version(&path, &display_path, &request.expected_version)?;
  temporary
    .as_file()
    .set_permissions(latest_metadata.permissions())
    .map_err(|error| Error::from_io(&display_path, error))?;
  temporary
    .persist(&path)
    .map_err(|error| Error::from_io(&display_path, error.error))?;
  sync_parent_directory(parent, &display_path)?;

  read_text_file(ReadTextFileRequest { path: display_path })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ErrorCode;

  #[test]
  fn atomically_writes_when_the_version_matches() {
    let directory = tempfile::tempdir().expect("creates a temporary directory");
    let path = directory.path().join("document.txt");
    fs::write(&path, "before").expect("writes the fixture");

    let snapshot = read_text_file(ReadTextFileRequest {
      path: path.to_string_lossy().into_owned(),
    })
    .expect("reads the fixture");
    let saved = write_text_file(WriteTextFileRequest {
      path: path.to_string_lossy().into_owned(),
      content: "after".to_string(),
      expected_version: snapshot.version.clone(),
    })
    .expect("saves the matching version");

    assert_eq!(saved.content, "after");
    assert_ne!(saved.version, snapshot.version);
    assert_eq!(
      fs::read_to_string(path).expect("reads the saved file"),
      "after",
    );
  }

  #[test]
  fn rejects_an_external_change_without_overwriting_it() {
    let directory = tempfile::tempdir().expect("creates a temporary directory");
    let path = directory.path().join("document.txt");
    fs::write(&path, "before").expect("writes the fixture");

    let snapshot = read_text_file(ReadTextFileRequest {
      path: path.to_string_lossy().into_owned(),
    })
    .expect("reads the fixture");
    fs::write(&path, "external").expect("simulates an external edit");

    let error = write_text_file(WriteTextFileRequest {
      path: path.to_string_lossy().into_owned(),
      content: "editor draft".to_string(),
      expected_version: snapshot.version,
    })
    .expect_err("rejects a stale version");

    assert_eq!(error.code, ErrorCode::VersionConflict);
    assert_eq!(
      fs::read_to_string(path).expect("reads the external edit"),
      "external",
    );
  }
}
