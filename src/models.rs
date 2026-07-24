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
