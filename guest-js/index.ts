import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

const COMMAND_PREFIX = "plugin:sito-workspace";

export const WORKSPACE_ENTRY_KIND = {
  DIRECTORY: "directory",
  FILE: "file",
  OTHER: "other",
} as const;

export type WorkspaceEntryKind =
  (typeof WORKSPACE_ENTRY_KIND)[keyof typeof WORKSPACE_ENTRY_KIND];

export const LINE_ENDING = {
  CRLF: "cr_lf",
  LF: "lf",
  MIXED: "mixed",
} as const;

export type LineEnding = (typeof LINE_ENDING)[keyof typeof LINE_ENDING];

export type WorkspaceEntry = {
  path: string;
  name: string;
  kind: WorkspaceEntryKind;
  isSymlink: boolean;
  readonly: boolean;
  size: number;
  modifiedAt: number | null;
};

export type TextDocumentSnapshot = {
  path: string;
  content: string;
  version: string;
  encoding: "utf-8";
  lineEnding: LineEnding;
  readonly: boolean;
  size: number;
};

export type WorkspaceErrorCode =
  | "already_exists"
  | "invalid_path"
  | "invalid_query"
  | "io"
  | "is_directory"
  | "not_file"
  | "not_directory"
  | "not_found"
  | "permission_denied"
  | "read_only"
  | "symlink_not_writable"
  | "task_failed"
  | "trash_failed"
  | "unsupported_encoding"
  | "version_conflict"
  | "watch_failed";

export type WorkspaceError = {
  code: WorkspaceErrorCode;
  message: string;
  path?: string;
};

export const listDirectory = async (
  path: string,
): Promise<WorkspaceEntry[]> =>
  await invoke(`${COMMAND_PREFIX}|list_directory`, {
    payload: { path },
  });

export const statEntry = async (path: string): Promise<WorkspaceEntry> =>
  await invoke(`${COMMAND_PREFIX}|stat_entry`, {
    payload: { path },
  });

export const readTextFile = async (
  path: string,
): Promise<TextDocumentSnapshot> =>
  await invoke(`${COMMAND_PREFIX}|read_text_file`, {
    payload: { path },
  });

export const writeTextFile = async (
  path: string,
  content: string,
  expectedVersion: string,
): Promise<TextDocumentSnapshot> =>
  await invoke(`${COMMAND_PREFIX}|write_text_file`, {
    payload: { path, content, expectedVersion },
  });

export type WorkspaceFileList = {
  /** The root exactly as requested; `files` are relative to it. */
  root: string;
  /** Relative paths using `/` separators, sorted. */
  files: string[];
  /** True when the listing stopped at `limit`. */
  truncated: boolean;
};

/**
 * Lists every file under `root` recursively, honoring `.gitignore`/`.ignore`.
 * Hidden files are included; `.git` is not.
 */
export const listWorkspaceFiles = async (
  root: string,
  limit?: number,
): Promise<WorkspaceFileList> =>
  await invoke(`${COMMAND_PREFIX}|list_workspace_files`, {
    payload: { root, limit: limit ?? null },
  });

/** One matching line. Offsets are UTF-16 code units, ready for `String#slice`. */
export type TextSearchMatch = {
  /** Relative to the searched root, `/`-separated. */
  path: string;
  /** 1-based line number. */
  line: number;
  /** 0-based column of the first match in the full line. */
  column: number;
  /** The line, windowed around the first match and without leading indentation. */
  preview: string;
  /** `[start, end)` ranges of every match inside `preview`. */
  ranges: [number, number][];
};

export type TextSearchResult = {
  root: string;
  /** Sorted by path, then line. */
  matches: TextSearchMatch[];
  /** True when the search stopped at `maxResults`. */
  truncated: boolean;
  filesSearched: number;
};

export type SearchTextOptions = {
  isRegex?: boolean;
  caseSensitive?: boolean;
  /** Defaults to 2,000 matching lines. */
  maxResults?: number;
};

/**
 * Searches the text of every file under `root`, honoring `.gitignore`/`.ignore`.
 * Binary, non-UTF-8 and files over 2 MiB are skipped. An invalid regex rejects
 * with code `invalid_query`.
 */
export const searchText = async (
  root: string,
  query: string,
  options: SearchTextOptions = {},
): Promise<TextSearchResult> =>
  await invoke(`${COMMAND_PREFIX}|search_text`, {
    payload: {
      root,
      query,
      isRegex: options.isRegex ?? null,
      caseSensitive: options.caseSensitive ?? null,
      maxResults: options.maxResults ?? null,
    },
  });

/** Creates an empty file; never overwrites (`already_exists`). */
export const createFile = async (path: string): Promise<WorkspaceEntry> =>
  await invoke(`${COMMAND_PREFIX}|create_file`, { payload: { path } });

/** Creates one directory (parent must exist); fails with `already_exists`. */
export const createDirectory = async (path: string): Promise<WorkspaceEntry> =>
  await invoke(`${COMMAND_PREFIX}|create_directory`, { payload: { path } });

/** Renames or moves an entry; never overwrites (`already_exists`). */
export const renameEntry = async (
  from: string,
  to: string,
): Promise<WorkspaceEntry> =>
  await invoke(`${COMMAND_PREFIX}|rename_entry`, { payload: { from, to } });

/** Moves an entry to the system trash (recoverable). */
export const trashEntry = async (path: string): Promise<void> =>
  await invoke(`${COMMAND_PREFIX}|trash_entry`, { payload: { path } });

export const WORKSPACE_CHANGED_EVENT = "sito-workspace://changed";

export type WorkspaceChangeEvent = {
  /** The watched root, as passed to `watchWorkspace`. */
  root: string;
  /** Absolute paths created, modified, removed or renamed, sorted. */
  paths: string[];
};

/**
 * Watches `root` recursively, replacing the previous watcher (one workspace per
 * app; watching the same root again is a no-op). Batched changes arrive
 * through `onWorkspaceChange`; `.git` internals are ignored.
 */
export const watchWorkspace = async (root: string): Promise<void> =>
  await invoke(`${COMMAND_PREFIX}|watch_workspace`, { payload: { root } });

/** Stops the watcher if it is watching `root`. */
export const unwatchWorkspace = async (root: string): Promise<void> =>
  await invoke(`${COMMAND_PREFIX}|unwatch_workspace`, { payload: { root } });

export const onWorkspaceChange = async (
  handler: (event: WorkspaceChangeEvent) => void,
): Promise<UnlistenFn> =>
  await listen<WorkspaceChangeEvent>(WORKSPACE_CHANGED_EVENT, (event) =>
    handler(event.payload),
  );
