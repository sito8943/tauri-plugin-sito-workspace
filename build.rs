const COMMANDS: &[&str] = &[
  "create_directory",
  "create_file",
  "list_directory",
  "list_workspace_files",
  "read_text_file",
  "rename_entry",
  "search_text",
  "stat_entry",
  "trash_entry",
  "unwatch_workspace",
  "watch_workspace",
  "write_text_file",
];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .build();
}
