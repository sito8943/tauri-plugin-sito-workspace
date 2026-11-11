use std::fs;
use std::path::Path;

use ignore::WalkBuilder;

use crate::models::{ListWorkspaceFilesRequest, WorkspaceFileList};
use crate::workspace::workspace_path;
use crate::{Error, Result};

/// Upper bound on returned files when the consumer does not pass a limit.
pub const DEFAULT_FILE_LIMIT: usize = 50_000;

/// Joins path components with `/` so consumers get one separator on every platform.
fn relative_path(path: &Path) -> String {
  path
    .components()
    .map(|component| component.as_os_str().to_string_lossy())
    .collect::<Vec<_>>()
    .join("/")
}

/// Lists every file under `root`, honoring `.gitignore`, `.ignore`, global git excludes and
/// `.git/info/exclude` (even outside a Git repository). Hidden files are included; `.git` is not.
pub fn list_workspace_files(request: ListWorkspaceFilesRequest) -> Result<WorkspaceFileList> {
  let root = workspace_path(request.root)?;
  let display_root = root.to_string_lossy().into_owned();
  let metadata = fs::metadata(&root).map_err(|error| Error::from_io(&display_root, error))?;
  if !metadata.is_dir() {
    return Err(Error::not_directory(display_root));
  }

  let limit = request.limit.unwrap_or(DEFAULT_FILE_LIMIT).max(1);
  let walker = WalkBuilder::new(&root)
    .hidden(false)
    .require_git(false)
    .follow_links(false)
    .filter_entry(|entry| entry.file_name() != ".git")
    .build();

  let mut files = Vec::new();
  let mut truncated = false;
  for result in walker {
    // Unreadable entries are skipped instead of failing the whole listing.
    let Ok(entry) = result else { continue };
    let is_file = match entry.file_type() {
      Some(file_type) if file_type.is_file() => true,
      Some(file_type) if file_type.is_symlink() => entry.path().is_file(),
      _ => false,
    };
    if !is_file {
      continue;
    }
    let Ok(relative) = entry.path().strip_prefix(&root) else {
      continue;
    };
    if files.len() >= limit {
      truncated = true;
      break;
    }
    files.push(relative_path(relative));
  }

  files.sort_unstable();
  Ok(WorkspaceFileList {
    root: display_root,
    files,
    truncated,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn fixture() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("creates a temporary directory");
    let root = directory.path();
    fs::create_dir_all(root.join("src/nested")).expect("creates folders");
    fs::create_dir_all(root.join("node_modules/pkg")).expect("creates ignored folder");
    fs::create_dir_all(root.join(".git")).expect("creates .git");
    fs::write(root.join(".gitignore"), "node_modules/\n*.log\n").expect("writes .gitignore");
    fs::write(root.join(".editorconfig"), "").expect("writes hidden file");
    fs::write(root.join("src/main.rs"), "").expect("writes file");
    fs::write(root.join("src/nested/lib.rs"), "").expect("writes file");
    fs::write(root.join("debug.log"), "").expect("writes ignored file");
    fs::write(root.join("node_modules/pkg/index.js"), "").expect("writes ignored file");
    fs::write(root.join(".git/HEAD"), "").expect("writes git internals");
    directory
  }

  #[test]
  fn lists_files_respecting_gitignore_and_skipping_git_internals() {
    let directory = fixture();
    let list = list_workspace_files(ListWorkspaceFilesRequest {
      root: directory.path().to_string_lossy().into_owned(),
      limit: None,
    })
    .expect("lists the workspace");

    assert_eq!(
      list.files,
      vec![".editorconfig", ".gitignore", "src/main.rs", "src/nested/lib.rs"],
    );
    assert!(!list.truncated);
  }

  #[test]
  fn truncates_at_the_limit() {
    let directory = fixture();
    let list = list_workspace_files(ListWorkspaceFilesRequest {
      root: directory.path().to_string_lossy().into_owned(),
      limit: Some(2),
    })
    .expect("lists the workspace");

    assert_eq!(list.files.len(), 2);
    assert!(list.truncated);
  }

  #[test]
  fn rejects_a_file_as_root() {
    let directory = fixture();
    let error = list_workspace_files(ListWorkspaceFilesRequest {
      root: directory.path().join("src/main.rs").to_string_lossy().into_owned(),
      limit: None,
    })
    .expect_err("rejects a file root");

    assert_eq!(error.code, crate::ErrorCode::NotDirectory);
  }
}
