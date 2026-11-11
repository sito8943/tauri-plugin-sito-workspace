# `@sito/plugin-workspace`

Shared workspace primitives for Sito Tauri applications.

This repository publishes two coordinated artifacts:

- `tauri-plugin-sito-workspace`: the Rust/Tauri plugin;
- `@sito/plugin-workspace`: typed TypeScript guest bindings.

## Current scope

The workspace contract supports:

- list a directory;
- list every file under a root recursively, honoring `.gitignore`/`.ignore` (for quick open);
- search the text of every file under a root (literal or regex, optional case sensitivity), one result per matching line with UTF-16 match ranges;
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
