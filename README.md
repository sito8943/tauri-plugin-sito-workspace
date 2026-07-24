# `@sito/plugin-workspace`

Shared workspace primitives for Sito Tauri applications.

This repository publishes two coordinated artifacts:

- `tauri-plugin-sito-workspace`: the Rust/Tauri plugin;
- `@sito/plugin-workspace`: typed TypeScript guest bindings.

## Current scope

The first vertical slice is intentionally read-only:

- list a directory;
- stat one filesystem entry;
- read a UTF-8 text document with version and line-ending metadata.

Filesystem work runs on Tauri's blocking pool. Errors cross the IPC boundary as structured
`{ code, message, path? }` payloads instead of UI text.

Writing, file watching, workspace-root permission scopes and remote resources are not implemented
yet. Consumers must not infer those capabilities from this initial API.
