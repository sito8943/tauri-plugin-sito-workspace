use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};

pub use models::*;

mod commands;
mod error;
mod models;
mod operations;
mod search;
mod watch;
mod workspace;

pub use error::{Error, ErrorCode, Result};
pub use watch::WORKSPACE_CHANGED_EVENT;

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("sito-workspace")
    .setup(|app, _api| {
      app.manage(watch::WatcherRegistry::default());
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::create_directory,
      commands::create_file,
      commands::list_directory,
      commands::list_workspace_files,
      commands::read_text_file,
      commands::rename_entry,
      commands::search_text,
      commands::stat_entry,
      commands::trash_entry,
      commands::unwatch_workspace,
      commands::watch_workspace,
      commands::write_text_file,
    ])
    .build()
}
