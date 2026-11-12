# Changelog

All notable changes to this project are documented in this file.

## [0.3.0]

### Added

- `create_file` / `createFile`, `create_directory` / `createDirectory` and `rename_entry` / `renameEntry` (also moves): never overwrite; an existing target fails with the new `already_exists` code. Not in the default permission set (`2e6e9cb`)
- `trash_entry` / `trashEntry`: moves an entry to the system trash, never deleting permanently (new `trash_failed` code). Not in the default permission set (`2e6e9cb`)
- `watch_workspace` / `watchWorkspace` and `unwatch_workspace` / `unwatchWorkspace`, with `onWorkspaceChange`: one recursive watcher per app sends batched `sito-workspace://changed` events with the changed paths, ignoring `.git` (new `watch_failed` code). In the default, read-only set (`2e6e9cb`)

## [0.2.0]

### Added

- `list_workspace_files` / `listWorkspaceFiles(root, limit?)`: lists every file under a folder, leaving out what `.gitignore`, `.ignore` and Git excludes ignore (also outside a Git repository) and the `.git` folder. Returns sorted relative paths with `/` separators and `truncated` when it stops at the limit (default 50,000). Included in the default, read-only permission set (`1d624d4`)
- `search_text` / `searchText(root, query, { isRegex, caseSensitive, maxResults })`: searches the text of the same files, literal or regex, optionally case-sensitive. One result per matching line with its 1-based line, UTF-16 column, a trimmed preview and the match ranges inside it. Skips binary, non-UTF-8 and files over 2 MiB; stops at 2,000 lines by default. An invalid regex fails with the new `invalid_query` code. Included in the default, read-only permission set (`fe79b8a`)

## [0.1.0]

### Added

- List a directory, stat an entry, read UTF-8 text documents with version and line-ending metadata, and atomically replace a document when its version still matches
