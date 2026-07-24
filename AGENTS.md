# Sito Workspace Plugin - Agent Instructions

These instructions apply to this repository.

## Required workflow

- Start with `git status --short`.
- Ask before acting on any pre-existing change.
- Do not execute package scripts, Cargo commands, builds, tests, formatters or installers unless the
  developer explicitly authorizes the exact command.
- Static inspection and `git diff --check` are allowed.
- Keep Rust commands, TypeScript bindings and generated command permissions aligned.

## Architecture

- Put reusable filesystem behavior in plain Rust modules.
- Keep Tauri commands as thin async adapters.
- Run blocking filesystem work through Tauri's blocking pool.
- Return structured errors; UI copy and notifications belong to consumers.
- Keep the default permission set non-destructive.
- Do not add terminal, agent, UI or application-specific behavior to this plugin.

## Completion

- Report all changed files.
- Distinguish static inspection from executed verification.
- Never claim runtime readiness until the developer has run and confirmed it.
