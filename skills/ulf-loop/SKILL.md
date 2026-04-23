---
name: ulf-loop
description: Run, monitor, resume, merge, and debug Ulf loops. Use this skill whenever the user asks to operate `ulf run` or `ulf loops`, inspect loop state, recover suspended loops, analyze diagnostics, or unblock merge queue issues.
---

# Ulf Loop

Use this skill to operate Ulf loops from the outside.

## Use This Skill For

- Starting or continuing a Ulf run with the right `-c` and `-H` inputs
- Inspecting loop state, worktrees, logs, history, and diffs
- Resuming a hook-suspended loop
- Merging or discarding completed worktree loops
- Debugging unexpected loop behavior with current diagnostics files

## Workflow

1. Start with `ulf loops list` or `ulf loops list --json` to establish the
   current state.
2. If the user wants execution, run `ulf run ...` with the right core config
   and hats source.
3. If the loop is stuck or suspicious, inspect `logs`, `history`, and `diff`
   before changing state.
4. If the loop is suspended, read `.ulf/suspend-state.json` and use
   `ulf loops resume <id>`.
5. If a loop is queued or in `needs-review`, inspect the diff first, then use
   `merge`, `process`, `retry`, or `discard` as appropriate.
6. Use diagnostics when you need detailed evidence about hats, events, tool
   calls, parse errors, or performance.

## Guardrails

- Prefer the CLI over direct edits to `.ulf` state files.
- Treat tasks and memories as the canonical runtime systems; do not center
  scratchpad as the primary state model.
- Inspect diffs before merging.
- Only remove lock or queue artifacts when the underlying process is confirmed
  dead.
- Manual edits under `.ulf/` are last-resort recovery steps and should be
  called out explicitly when used.

## Read These References When Needed

- For command recipes and operator flows: `references/commands.md`
- For diagnostics files and suspend-state details: `references/diagnostics.md`
