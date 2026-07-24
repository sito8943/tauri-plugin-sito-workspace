use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::SitoWorkspace;
#[cfg(mobile)]
use mobile::SitoWorkspace;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the sito-workspace APIs.
pub trait SitoWorkspaceExt<R: Runtime> {
  fn sito_workspace(&self) -> &SitoWorkspace<R>;
}

impl<R: Runtime, T: Manager<R>> crate::SitoWorkspaceExt<R> for T {
  fn sito_workspace(&self) -> &SitoWorkspace<R> {
    self.state::<SitoWorkspace<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("sito-workspace")
    .invoke_handler(tauri::generate_handler![commands::ping])
    .setup(|app, api| {
      #[cfg(mobile)]
      let sito_workspace = mobile::init(app, api)?;
      #[cfg(desktop)]
      let sito_workspace = desktop::init(app, api)?;
      app.manage(sito_workspace);
      Ok(())
    })
    .build()
}
