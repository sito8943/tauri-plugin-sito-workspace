const COMMANDS: &[&str] = &[
  "list_directory",
  "list_workspace_files",
  "read_text_file",
  "search_text",
  "stat_entry",
  "write_text_file",
];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .build();
}
