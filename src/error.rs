use std::io::ErrorKind;

use serde::Serialize;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
  InvalidPath,
  IsDirectory,
  NotFile,
  NotDirectory,
  NotFound,
  PermissionDenied,
  ReadOnly,
  SymlinkNotWritable,
  UnsupportedEncoding,
  VersionConflict,
  Io,
  TaskFailed,
}

#[derive(Debug, Serialize, thiserror::Error)]
#[error("{message}")]
#[serde(rename_all = "camelCase")]
pub struct Error {
  pub code: ErrorCode,
  pub message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub path: Option<String>,
}

impl Error {
  pub fn invalid_path(path: impl Into<String>) -> Self {
    let path = path.into();
    Self {
      code: ErrorCode::InvalidPath,
      message: "Workspace paths cannot be empty".to_string(),
      path: Some(path),
    }
  }

  pub fn is_directory(path: impl Into<String>) -> Self {
    let path = path.into();
    Self {
      code: ErrorCode::IsDirectory,
      message: "Expected a file but received a directory".to_string(),
      path: Some(path),
    }
  }

  pub fn not_directory(path: impl Into<String>) -> Self {
    let path = path.into();
    Self {
      code: ErrorCode::NotDirectory,
      message: "Expected a directory".to_string(),
      path: Some(path),
    }
  }

  pub fn not_file(path: impl Into<String>) -> Self {
    let path = path.into();
    Self {
      code: ErrorCode::NotFile,
      message: "Expected a regular file".to_string(),
      path: Some(path),
    }
  }

  pub fn read_only(path: impl Into<String>) -> Self {
    let path = path.into();
    Self {
      code: ErrorCode::ReadOnly,
      message: "The file is read-only".to_string(),
      path: Some(path),
    }
  }

  pub fn symlink_not_writable(path: impl Into<String>) -> Self {
    let path = path.into();
    Self {
      code: ErrorCode::SymlinkNotWritable,
      message: "Writing through symbolic links is not supported".to_string(),
      path: Some(path),
    }
  }

  pub fn unsupported_encoding(path: impl Into<String>) -> Self {
    let path = path.into();
    Self {
      code: ErrorCode::UnsupportedEncoding,
      message: "Only UTF-8 text files are supported".to_string(),
      path: Some(path),
    }
  }

  pub fn version_conflict(path: impl Into<String>) -> Self {
    let path = path.into();
    Self {
      code: ErrorCode::VersionConflict,
      message: "The file changed on disk after it was opened".to_string(),
      path: Some(path),
    }
  }

  pub fn from_io(path: impl Into<String>, error: std::io::Error) -> Self {
    let code = match error.kind() {
      ErrorKind::NotFound => ErrorCode::NotFound,
      ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
      _ => ErrorCode::Io,
    };
    Self {
      code,
      message: error.to_string(),
      path: Some(path.into()),
    }
  }

  pub fn from_task(error: impl ToString) -> Self {
    Self {
      code: ErrorCode::TaskFailed,
      message: error.to_string(),
      path: None,
    }
  }
}
