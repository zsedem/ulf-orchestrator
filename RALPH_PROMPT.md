# Ralph Loop Prompt

You are working on the `feat/multi-workspace-vibe-coding` branch of `ulf-orchestrator`.

## Current State

- All tests pass (`cargo test` green)
- Phase 1 is committed: workspace types, WorkspaceDomain, CLI commands, domain isolation
- GOAL.md contains the full multi-workspace vibe-coding vision
- SCRATCHPAD.md tracks completed/in-progress/blocked work

## Your Task

Read GOAL.md and SCRATCHPAD.md. Pick the highest-priority incomplete item and implement it. After each change:
1. Run `cargo test`
2. If tests pass, update SCRATCHPAD.md to mark items done/in-progress
3. If tests fail, fix them before proceeding
4. Commit your work with a descriptive message
5. Output `<choice>STOP</choice>` only when GOAL.md shows all completion criteria met

## Constraints

- Do NOT implement without an approved spec when the change is architectural
- Run `cargo test` before declaring any task done
- Make MINIMAL changes
- Follow existing code style
- Backwards compatibility doesn't matter

## Immediate Priority Order

1. Finish wiring default setup prompt from `~/.ulf/config.yml`
2. Implement setup prompt background execution with status tracking
3. Implement daemon auto-start for `ulf workspace` commands
4. Start Phase 2: `ulf workspace attach` middle-manager MVP
