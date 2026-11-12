use std::collections::BTreeSet;
use std::path::{Component, Path};
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, Runtime};

use crate::models::{WatchWorkspaceRequest, WorkspaceChangeEvent};
use crate::workspace::workspace_path;
use crate::{Error, Result};

/// Event emitted with a [`WorkspaceChangeEvent`] after changes settle.
pub const WORKSPACE_CHANGED_EVENT: &str = "sito-workspace://changed";
/// Quiet time that closes a batch of changes (saves often touch several files).
const DEBOUNCE: Duration = Duration::from_millis(150);

/// The active watcher and its root: one workspace per app. Dropping a watcher stops it and
/// ends its batching thread.
#[derive(Default)]
pub struct WatcherRegistry(Mutex<Option<(String, RecommendedWatcher)>>);

fn inside_git_directory(path: &Path) -> bool {
  path
    .components()
    .any(|component| matches!(component, Component::Normal(name) if name == ".git"))
}

/// Watches `root` recursively, replacing any previous watcher. Changes are batched and emitted
/// as [`WORKSPACE_CHANGED_EVENT`]; `.git` internals are ignored. Watching the root that is
/// already watched is a no-op, so repeated calls (e.g. React StrictMode) are harmless.
pub fn watch_workspace<R: Runtime>(
  app: AppHandle<R>,
  registry: &WatcherRegistry,
  request: WatchWorkspaceRequest,
) -> Result<()> {
  let root = workspace_path(request.root)?;
  let display_root = root.to_string_lossy().into_owned();
  {
    let active = registry
      .0
      .lock()
      .map_err(|error| Error::from_task(error.to_string()))?;
    if matches!(active.as_ref(), Some((watched, _)) if *watched == display_root) {
      return Ok(());
    }
  }
  let metadata = std::fs::metadata(&root).map_err(|error| Error::from_io(&display_root, error))?;
  if !metadata.is_dir() {
    return Err(Error::not_directory(display_root));
  }

  let (sender, receiver) = mpsc::channel::<notify::Event>();
  let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
    if let Ok(event) = result {
      let _ = sender.send(event);
    }
  })
  .map_err(|error| Error::watch_failed(&display_root, error.to_string()))?;
  watcher
    .watch(&root, RecursiveMode::Recursive)
    .map_err(|error| Error::watch_failed(&display_root, error.to_string()))?;

  let event_root = display_root.clone();
  std::thread::spawn(move || {
    let mut pending = BTreeSet::new();
    loop {
      match receiver.recv_timeout(DEBOUNCE) {
        Ok(event) => {
          for path in event.paths {
            if !inside_git_directory(&path) {
              pending.insert(path.to_string_lossy().into_owned());
            }
          }
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
          if !pending.is_empty() {
            let _ = app.emit(
              WORKSPACE_CHANGED_EVENT,
              WorkspaceChangeEvent {
                root: event_root.clone(),
                paths: std::mem::take(&mut pending).into_iter().collect(),
              },
            );
          }
        }
        // The watcher (and its sender) was dropped: stop.
        Err(mpsc::RecvTimeoutError::Disconnected) => return,
      }
    }
  });

  let previous = registry
    .0
    .lock()
    .map_err(|error| Error::from_task(error.to_string()))?
    .replace((display_root, watcher));
  // Stop the old watcher outside the lock.
  drop(previous);
  Ok(())
}

/// Stops the watcher when it is watching `root`; another root's watcher is left alone.
pub fn unwatch_workspace(registry: &WatcherRegistry, request: WatchWorkspaceRequest) -> Result<()> {
  let root = workspace_path(request.root)?;
  let display_root = root.to_string_lossy().into_owned();
  let previous = {
    let mut active = registry
      .0
      .lock()
      .map_err(|error| Error::from_task(error.to_string()))?;
    if matches!(active.as_ref(), Some((watched, _)) if *watched == display_root) {
      active.take()
    } else {
      None
    }
  };
  drop(previous);
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ignores_git_internals_only() {
    assert!(inside_git_directory(Path::new("/repo/.git/index")));
    assert!(!inside_git_directory(Path::new("/repo/.github/workflows/ci.yml")));
    assert!(!inside_git_directory(Path::new("/repo/src/main.rs")));
  }
}
