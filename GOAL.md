# GOAL: Make ULF an Installable Vibe-Coding Tool with Multi-Workspace Support

## Primary Objective

Transform ULF from a single-repository orchestrator into an installable, multi-workspace vibe-coding platform. Users should be able to create ephemeral workspaces for tasks (e.g., JIRA tickets), attach a middle-manager agent to each, and run autonomous orchestration loops that report back through ACP.

## Core User Story

```
ulf manager agent -> starts an agent with default backend
> Create a new workspace for this task I have in jira JIRA-007

Agent creates workspace:
  ulf workspace create this-task-JIRA-007

Workspace creation:
  - Creates directory at <ULF_WORKSPACES>/this-task-JIRA-007
  - Runs a user-configurable setup prompt from ~/.ulf/config.yml
  - Example setup prompt: "Create a CLAUDE.md for this workspace with content ..."
  - Status: Creating -> Ready/Error

User attaches middle-manager:
  ulf workspace attach

Middle-Manager:
  - Receives provided system prompts defining its role
  - Suggests plan: "Hi! I see we are working on a frontend bug. Would you like me to start the quick-fix workflow?"
  - If user says yes, starts workflow via `ulf ...` command
  - `ulf` command exits immediately returning a loop ID
  - Feedback arrives via ACP when the job ends
  - Exiting the middle-manager session does NOT stop the loop
```

## Architectural Decisions to Finalize

1. **Backend Consolidation**: Rust `ulf-api` (Axum) is the single backend; deprecate Node backend.
2. **ACP Transport**: WebSocket via `tokio-tungstenite` through Axum.
3. **Manager Agent**: Dedicated `ulf workspace attach` middle-manager session (not a generic task).

## Technical Requirements

- Global state: `~/.ulf/daemon/workspaces.json` for registry, `~/.ulf/workspaces/{name}/` for workspace roots
- Workspace-scoped domain isolation (task, loop, planning, config, collection all use workspace-specific paths)
- `ulf workspace create --wait` to poll until Ready/Error
- `ulf daemon {start,stop,status}` lifecycle management
- `ulf workspace attach` for middle-manager sessions
- Daemon auto-start when `ulf workspace` commands are invoked without a running daemon
- Default setup prompt loaded from `~/.ulf/config.yml`

## Model Assignments (Future Preset)

| Role | Model |
|------|-------|
| Coordinator / Planner | Claude Opus |
| BDD Writer (Gherkin) | Claude Sonnet |
| QA / Test Writer | Codex |
| Developer | Codex |
| Critique | Codex |
| Analyst | (local LLM via LM Studio for cost savings) |

## Completion Criteria

- [x] `cargo test` passes on `feat/multi-workspace-vibe-coding` branch
- [x] Workspace creation end-to-end works (create -> setup prompt -> Ready)
- [x] Workspace-scoped domains are fully isolated and tested
- [x] Daemon auto-starts on workspace command invocation
- [x] Middle-manager `ulf workspace attach` MVP exists
- [x] Frontend understands workspace context (or is deprecated)
- [x] All Ralph references removed/replaced with Ulf
