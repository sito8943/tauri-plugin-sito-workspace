use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::models::{
  LineEnding, ListDirectoryRequest, ReadTextFileRequest, StatEntryRequest,
  TextDocumentSnapshot, WorkspaceEntry, WorkspaceEntryKind,
};
use crate::{Error, Result};

fn workspace_path(raw: String) -> Result<PathBuf> {
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
  let content =
    String::from_utf8(bytes).map_err(|_| Error::unsupported_encoding(&display_path))?;
  let modified_at = modified_at_millis(&metadata)
    .map(|value| value.to_string())
    .unwrap_or_else(|| "unknown".to_string());

  Ok(TextDocumentSnapshot {
    path: display_path,
    line_ending: detect_line_ending(&content),
    content,
    version: format!("{modified_at}:{}", metadata.len()),
    encoding: "utf-8".to_string(),
    readonly: metadata.permissions().readonly(),
    size: metadata.len(),
  })
}
