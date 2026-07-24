use tauri::plugin::{Builder, TauriPlugin};
use tauri::Runtime;

pub use models::*;

mod commands;
mod error;
mod models;
mod workspace;

pub use error::{Error, ErrorCode, Result};

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("sito-workspace")
    .invoke_handler(tauri::generate_handler![
      commands::list_directory,
      commands::read_text_file,
      commands::stat_entry,
      commands::write_text_file,
    ])
    .build()
}
