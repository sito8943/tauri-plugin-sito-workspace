use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use regex::RegexBuilder;

use crate::models::{
  ListWorkspaceFilesRequest, SearchTextRequest, TextSearchMatch, TextSearchResult,
  WorkspaceFileList,
};
use crate::workspace::workspace_path;
use crate::{Error, Result};

/// Upper bound on returned files when the consumer does not pass a limit.
pub const DEFAULT_FILE_LIMIT: usize = 50_000;
/// Upper bound on matching lines when the consumer does not pass a limit.
pub const DEFAULT_MAX_RESULTS: usize = 2_000;
/// Files larger than this are skipped by text search.
const MAX_SEARCH_FILE_BYTES: u64 = 2 * 1024 * 1024;
/// Bytes inspected for a NUL byte to decide a file is binary.
const BINARY_SNIFF_BYTES: usize = 8 * 1024;
/// Characters kept in a match preview, and how many of them precede the first match.
const PREVIEW_CHARS: usize = 240;
const PREVIEW_LEAD_CHARS: usize = 40;

/// Joins path components with `/` so consumers get one separator on every platform.
fn relative_path(path: &Path) -> String {
  path
    .components()
    .map(|component| component.as_os_str().to_string_lossy())
    .collect::<Vec<_>>()
    .join("/")
}

/// Validates that `raw` is an existing directory.
fn workspace_root(raw: String) -> Result<(PathBuf, String)> {
  let root = workspace_path(raw)?;
  let display_root = root.to_string_lossy().into_owned();
  let metadata = fs::metadata(&root).map_err(|error| Error::from_io(&display_root, error))?;
  if !metadata.is_dir() {
    return Err(Error::not_directory(display_root));
  }
  Ok((root, display_root))
}

/// Visits every file under `root` as `(absolute, relative)`, honoring `.gitignore`, `.ignore`,
/// global git excludes and `.git/info/exclude` (even outside a Git repository). Hidden files are
/// included; `.git` is not. Unreadable entries are skipped. Return `false` to stop walking.
fn walk_files(root: &Path, mut visit: impl FnMut(&Path, String) -> bool) {
  let walker = WalkBuilder::new(root)
    .hidden(false)
    .require_git(false)
    .follow_links(false)
    .filter_entry(|entry| entry.file_name() != ".git")
    .build();

  for result in walker {
    let Ok(entry) = result else { continue };
    let is_file = match entry.file_type() {
      Some(file_type) if file_type.is_file() => true,
      Some(file_type) if file_type.is_symlink() => entry.path().is_file(),
      _ => false,
    };
    if !is_file {
      continue;
    }
    let Ok(relative) = entry.path().strip_prefix(root) else {
      continue;
    };
    if !visit(entry.path(), relative_path(relative)) {
      return;
    }
  }
}

/// Lists every file under `root` (see [`walk_files`] for the filtering rules).
pub fn list_workspace_files(request: ListWorkspaceFilesRequest) -> Result<WorkspaceFileList> {
  let (root, display_root) = workspace_root(request.root)?;
  let limit = request.limit.unwrap_or(DEFAULT_FILE_LIMIT).max(1);

  let mut files = Vec::new();
  let mut truncated = false;
  walk_files(&root, |_, relative| {
    if files.len() >= limit {
      truncated = true;
      return false;
    }
    files.push(relative);
    true
  });

  files.sort_unstable();
  Ok(WorkspaceFileList {
    root: display_root,
    files,
    truncated,
  })
}

fn utf16_len(text: &str) -> u32 {
  text.encode_utf16().count() as u32
}

/// Builds one result line. Offsets are UTF-16 code units so JavaScript can slice with them.
fn build_match(path: &str, line_index: usize, line: &str, ranges: &[(usize, usize)]) -> TextSearchMatch {
  let first_byte = ranges[0].0;

  // Window the preview around the first match and drop leading indentation.
  let mut start = if line.chars().count() <= PREVIEW_CHARS {
    0
  } else {
    line[..first_byte]
      .char_indices()
      .rev()
      .nth(PREVIEW_LEAD_CHARS - 1)
      .map(|(index, _)| index)
      .unwrap_or(0)
  };
  let indentation = line.len() - line.trim_start().len();
  start = start.max(indentation.min(first_byte));
  let end = line[start..]
    .char_indices()
    .nth(PREVIEW_CHARS)
    .map(|(index, _)| start + index)
    .unwrap_or(line.len());

  let preview_ranges = ranges
    .iter()
    .filter(|(match_start, match_end)| *match_end > start && *match_start < end)
    .map(|(match_start, match_end)| {
      [
        utf16_len(&line[start..(*match_start).max(start)]),
        utf16_len(&line[start..(*match_end).min(end)]),
      ]
    })
    .collect();

  TextSearchMatch {
    path: path.to_string(),
    line: line_index as u32 + 1,
    column: utf16_len(&line[..first_byte]),
    preview: line[start..end].to_string(),
    ranges: preview_ranges,
  }
}

/// Searches the text of every file under `root` (see [`walk_files`]), one result per matching
/// line. Binary, non-UTF-8 and files over 2 MiB are skipped.
pub fn search_text(request: SearchTextRequest) -> Result<TextSearchResult> {
  let (root, display_root) = workspace_root(request.root)?;
  let mut result = TextSearchResult {
    root: display_root,
    matches: Vec::new(),
    truncated: false,
    files_searched: 0,
  };
  if request.query.is_empty() {
    return Ok(result);
  }

  let pattern = if request.is_regex.unwrap_or(false) {
    request.query.clone()
  } else {
    regex::escape(&request.query)
  };
  let matcher = RegexBuilder::new(&pattern)
    .case_insensitive(!request.case_sensitive.unwrap_or(false))
    .build()
    .map_err(|error| Error::invalid_query(error.to_string()))?;
  let max_results = request.max_results.unwrap_or(DEFAULT_MAX_RESULTS).max(1);

  walk_files(&root, |path, relative| {
    let too_large = fs::metadata(path)
      .map(|metadata| metadata.len() > MAX_SEARCH_FILE_BYTES)
      .unwrap_or(true);
    if too_large {
      return true;
    }
    let Ok(bytes) = fs::read(path) else {
      return true;
    };
    if bytes.iter().take(BINARY_SNIFF_BYTES).any(|byte| *byte == 0) {
      return true;
    }
    let Ok(text) = std::str::from_utf8(&bytes) else {
      return true;
    };
    result.files_searched += 1;

    for (line_index, line) in text.lines().enumerate() {
      let ranges: Vec<(usize, usize)> = matcher
        .find_iter(line)
        .filter(|found| found.start() != found.end())
        .map(|found| (found.start(), found.end()))
        .collect();
      if ranges.is_empty() {
        continue;
      }
      if result.matches.len() >= max_results {
        result.truncated = true;
        return false;
      }
      result
        .matches
        .push(build_match(&relative, line_index, line, &ranges));
    }
    true
  });

  result
    .matches
    .sort_by(|left, right| left.path.cmp(&right.path).then(left.line.cmp(&right.line)));
  Ok(result)
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
    fs::write(
      root.join("src/main.rs"),
      "fn main() {\n    let needle = 1;\n    println!(\"Needle {}\", needle);\n}\n",
    )
    .expect("writes file");
    fs::write(root.join("src/nested/lib.rs"), "// 🦀 needle\n").expect("writes file");
    fs::write(root.join("debug.log"), "needle\n").expect("writes ignored file");
    fs::write(root.join("node_modules/pkg/index.js"), "needle\n").expect("writes ignored file");
    fs::write(root.join(".git/HEAD"), "needle\n").expect("writes git internals");
    directory
  }

  fn search(directory: &tempfile::TempDir, query: &str) -> SearchTextRequest {
    SearchTextRequest {
      root: directory.path().to_string_lossy().into_owned(),
      query: query.to_string(),
      is_regex: None,
      case_sensitive: None,
      max_results: None,
    }
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

  #[test]
  fn finds_literal_text_case_insensitively_outside_ignored_files() {
    let directory = fixture();
    let result = search_text(search(&directory, "needle")).expect("searches");

    let found: Vec<(&str, u32)> = result
      .matches
      .iter()
      .map(|found| (found.path.as_str(), found.line))
      .collect();
    assert_eq!(
      found,
      vec![("src/main.rs", 2), ("src/main.rs", 3), ("src/nested/lib.rs", 1)],
    );

    let println = &result.matches[1];
    assert_eq!(println.preview, "println!(\"Needle {}\", needle);");
    assert_eq!(println.ranges, vec![[10, 16], [22, 28]]);
    assert_eq!(println.column, 14);
  }

  #[test]
  fn reports_utf16_offsets() {
    let directory = fixture();
    let result = search_text(search(&directory, "needle")).expect("searches");
    let crab = result
      .matches
      .iter()
      .find(|found| found.path == "src/nested/lib.rs")
      .expect("finds the line with an emoji");

    // "// " is 3 units and the crab is 2 UTF-16 units, then a space.
    assert_eq!(crab.column, 6);
    assert_eq!(crab.ranges, vec![[6, 12]]);
  }

  #[test]
  fn supports_case_sensitive_regex() {
    let directory = fixture();
    let mut request = search(&directory, r"N\w+");
    request.is_regex = Some(true);
    request.case_sensitive = Some(true);
    let result = search_text(request).expect("searches");

    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].line, 3);
  }

  #[test]
  fn rejects_an_invalid_regex() {
    let directory = fixture();
    let mut request = search(&directory, "(unclosed");
    request.is_regex = Some(true);
    let error = search_text(request).expect_err("rejects the pattern");

    assert_eq!(error.code, crate::ErrorCode::InvalidQuery);
  }

  #[test]
  fn truncates_at_max_results() {
    let directory = fixture();
    let mut request = search(&directory, "needle");
    request.max_results = Some(1);
    let result = search_text(request).expect("searches");

    assert_eq!(result.matches.len(), 1);
    assert!(result.truncated);
  }
}
