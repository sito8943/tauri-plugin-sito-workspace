# `@sito/plugin-workspace`

Shared workspace primitives for Sito Tauri applications.

This repository publishes two coordinated artifacts:

- `tauri-plugin-sito-workspace`: the Rust/Tauri plugin;
- `@sito/plugin-workspace`: typed TypeScript guest bindings.

## Current scope

The workspace contract supports:

- list a directory;
- stat one filesystem entry;
- read a UTF-8 text document with version and line-ending metadata.
- atomically replace an existing UTF-8 text document when its content version still matches.

Filesystem work runs on Tauri's blocking pool. Errors cross the IPC boundary as structured
`{ code, message, path? }` payloads instead of UI text.

Writing is excluded from the default permission set. Consumers must explicitly grant
`sito-workspace:allow-write-text-file`. Writes reject version conflicts, symbolic links,
read-only files and non-regular files.

File watching, workspace-root permission scopes, file creation/deletion and remote resources are
not implemented yet.
