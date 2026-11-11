use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceEntryKind {
  Directory,
  File,
  Other,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEntry {
  pub path: String,
  pub name: String,
  pub kind: WorkspaceEntryKind,
  pub is_symlink: bool,
  pub readonly: bool,
  pub size: u64,
  pub modified_at: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDirectoryRequest {
  pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatEntryRequest {
  pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileRequest {
  pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteTextFileRequest {
  pub path: String,
  pub content: String,
  pub expected_version: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LineEnding {
  CrLf,
  Lf,
  Mixed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentSnapshot {
  pub path: String,
  pub content: String,
  pub version: String,
  pub encoding: String,
  pub line_ending: LineEnding,
  pub readonly: bool,
  pub size: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkspaceFilesRequest {
  pub root: String,
  pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFileList {
  /// The root exactly as requested; `files` are relative to it.
  pub root: String,
  /// Relative paths using `/` separators, sorted.
  pub files: Vec<String>,
  /// True when the listing stopped at the limit.
  pub truncated: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchTextRequest {
  pub root: String,
  pub query: String,
  pub is_regex: Option<bool>,
  pub case_sensitive: Option<bool>,
  pub max_results: Option<usize>,
}

/// One matching line. Offsets are UTF-16 code units.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextSearchMatch {
  /// Relative to the searched root, `/`-separated.
  pub path: String,
  /// 1-based line number.
  pub line: u32,
  /// 0-based column of the first match in the full line.
  pub column: u32,
  /// The line, windowed around the first match and without leading indentation.
  pub preview: String,
  /// `[start, end)` ranges of every match inside `preview`.
  pub ranges: Vec<[u32; 2]>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextSearchResult {
  pub root: String,
  /// Sorted by path, then line.
  pub matches: Vec<TextSearchMatch>,
  /// True when the search stopped at `max_results`.
  pub truncated: bool,
  pub files_searched: u32,
}
