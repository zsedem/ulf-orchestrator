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

## Disk Management Rules (Prevent 18GB target/ bloat)

4. If `target/` exceeds 6GB, run `cargo clean` before continuing. Check with: `du -sm target/ | cut -f1`
5. Prefer `cargo test --no-run` to compile tests, then run only the relevant test binary. Only run full `cargo test --workspace` when you are ready to verify everything. This avoids rebuilding all 9 workspace crates on every iteration.

## Critical Gate: Devil's Advocate Audit

Before declaring the GOAL reached, you MUST spawn at least 5 parallel devil's advocate subagents to critically review the implementation. Each agent must adopt a distinct persona (e.g. Security Engineer, SRE/Reliability Engineer, CLI UX Purist, AI Systems Researcher, Distributed Systems Engineer, Configurability Architect).

The GOAL is NOT reached until:
- All completion criteria in GOAL.md are checked off, AND
- The devil's advocate audit reports zero Critical or High severity issues

If major issues are found, add them to SCRATCHPAD.md as blockers and continue fixing them.
