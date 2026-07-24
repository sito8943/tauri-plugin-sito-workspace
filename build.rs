const COMMANDS: &[&str] = &[
  "list_directory",
  "read_text_file",
  "stat_entry",
];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .build();
}
