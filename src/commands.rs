use tauri::command;

use crate::models::*;
use crate::Result;

#[command]
pub(crate) async fn list_directory(
  payload: ListDirectoryRequest,
) -> Result<Vec<WorkspaceEntry>> {
  tauri::async_runtime::spawn_blocking(move || crate::workspace::list_directory(payload))
    .await
    .map_err(crate::Error::from_task)?
}

#[command]
pub(crate) async fn stat_entry(payload: StatEntryRequest) -> Result<WorkspaceEntry> {
  tauri::async_runtime::spawn_blocking(move || crate::workspace::stat_entry(payload))
    .await
    .map_err(crate::Error::from_task)?
}

#[command]
pub(crate) async fn read_text_file(
  payload: ReadTextFileRequest,
) -> Result<TextDocumentSnapshot> {
  tauri::async_runtime::spawn_blocking(move || crate::workspace::read_text_file(payload))
    .await
    .map_err(crate::Error::from_task)?
}

#[command]
pub(crate) async fn write_text_file(
  payload: WriteTextFileRequest,
) -> Result<TextDocumentSnapshot> {
  tauri::async_runtime::spawn_blocking(move || crate::workspace::write_text_file(payload))
    .await
    .map_err(crate::Error::from_task)?
}

#[command]
pub(crate) async fn list_workspace_files(
  payload: ListWorkspaceFilesRequest,
) -> Result<WorkspaceFileList> {
  tauri::async_runtime::spawn_blocking(move || crate::search::list_workspace_files(payload))
    .await
    .map_err(crate::Error::from_task)?
}

#[command]
pub(crate) async fn search_text(payload: SearchTextRequest) -> Result<TextSearchResult> {
  tauri::async_runtime::spawn_blocking(move || crate::search::search_text(payload))
    .await
    .map_err(crate::Error::from_task)?
}

#[command]
pub(crate) async fn create_file(payload: CreateEntryRequest) -> Result<WorkspaceEntry> {
  tauri::async_runtime::spawn_blocking(move || crate::operations::create_file(payload))
    .await
    .map_err(crate::Error::from_task)?
}

#[command]
pub(crate) async fn create_directory(payload: CreateEntryRequest) -> Result<WorkspaceEntry> {
  tauri::async_runtime::spawn_blocking(move || crate::operations::create_directory(payload))
    .await
    .map_err(crate::Error::from_task)?
}

#[command]
pub(crate) async fn rename_entry(payload: RenameEntryRequest) -> Result<WorkspaceEntry> {
  tauri::async_runtime::spawn_blocking(move || crate::operations::rename_entry(payload))
    .await
    .map_err(crate::Error::from_task)?
}

#[command]
pub(crate) async fn trash_entry(payload: TrashEntryRequest) -> Result<()> {
  tauri::async_runtime::spawn_blocking(move || crate::operations::trash_entry(payload))
    .await
    .map_err(crate::Error::from_task)?
}

#[command]
pub(crate) async fn watch_workspace<R: tauri::Runtime>(
  app: tauri::AppHandle<R>,
  registry: tauri::State<'_, crate::watch::WatcherRegistry>,
  payload: WatchWorkspaceRequest,
) -> Result<()> {
  crate::watch::watch_workspace(app, &registry, payload)
}

#[command]
pub(crate) async fn unwatch_workspace(
  registry: tauri::State<'_, crate::watch::WatcherRegistry>,
  payload: WatchWorkspaceRequest,
) -> Result<()> {
  crate::watch::unwatch_workspace(&registry, payload)
}
