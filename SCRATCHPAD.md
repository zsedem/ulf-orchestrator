# SCRATCHPAD: Multi-Workspace Vibe-Coding Implementation

## Session: c3aa568e — "Supervisor pattern"

## ✅ Completed

### Phase 0: Foundation
- Renamed all `ralph` references to `ulf` across the entire codebase (committed)
- Added completion gates for end-of-session backpressure (bash scripts must exit 0)
- Added publish backpressure when hats omit event emission
- Created Enterprise Dev Pipeline preset with backend assignments and graph

### Phase 1: Multi-Workspace Data Model & Daemon Core
- Added `Workspace`, `WorkspaceStatus`, `WorkspaceKind`, `WorkspaceRegistry` types to `ulf-core/src/workspaces.rs`
- Built `WorkspaceDomain` in `ulf-api` with CRUD + list + status RPC methods
- Persisted workspace registry to JSON at `{daemon_state_dir}/workspaces.json`
- Added `workspace.*` methods to `KNOWN_METHODS`, `MUTATING_METHODS`, and `rpc-v1-schema.json`
- Added `WorkspaceNotFound` to `RpcErrorCode`
- Implemented CLI workspace commands: `ulf workspace {create,list,get,delete,status}`
- Implemented CLI daemon commands: `ulf daemon {start,stop,status}`
- Added workspace-scoped domain resolution for task/loop/planning/config/collection routes
- Verified domain isolation: each workspace stores state under `{workspace_root}/.ulf/api/...`
- Added `workspace.status` RPC with detailed health info
- Added `--wait` flag to `ulf workspace create` to poll until Ready/Error
- Added workspace health check (dir exists, readable, writable)
- Wrote `workspace_isolation.rs` test file (uncommitted)

## ✅ Committed (Phase 1 Complete)

All Phase 1 changes committed as `297ba81`:
- `crates/ulf-api/data/rpc-v1-schema.json`
- `crates/ulf-api/src/protocol.rs`
- `crates/ulf-api/src/runtime.rs`
- `crates/ulf-api/src/runtime/dispatch.rs`
- `crates/ulf-api/src/workspace_domain.rs`
- `crates/ulf-cli/src/daemon.rs`
- `crates/ulf-cli/src/daemon_client.rs`
- `crates/ulf-cli/src/workspace_cli.rs`
- `crates/ulf-core/src/config.rs`
- `crates/ulf-api/tests/workspace_isolation.rs`
- `GOAL.md`
- `SCRATCHPAD.md`

## ❌ Blocked / Not Started

### Setup Prompt Execution
- Default setup prompt from `~/.ulf/config.yml` — partially implemented but not wired end-to-end
- Background task spawning for setup execution (tokio::spawn from sync dispatch)
- Status transition: Creating -> Ready/Error after setup completes

### Cargo Test Verification
- ✅ `cargo test` passes fully (all crates green)
- target/ rebuilt successfully (~2.1GB, not 18GB)

### Architectural Decisions (Background Tasks Timed Out)
- Backend consolidation pro/con analysis — agent timed out after 180s
- ACP transport pro/con analysis — agent timed out after 180s
- Decisions assumed but not formally ratified in code/docs

### Phase 2: Manager Agent & Attach
- `ulf workspace attach` middle-manager command not implemented
- Manager agent prompts not designed
- ACP feedback loop from running loops back to manager not wired

### Phase 3: Frontend & Polish
- Frontend still shows "RO" in top-left corner (needs "ULF")
- Frontend npm build issues (`npm ci` fails with ENOENT on wrong path)
- Frontend only capable of creating loops inside current repository
- Frontend needs workspace context awareness or deprecation plan

### Daemon Auto-Start
- `ulf workspace` commands assume daemon is running
- Auto-start on first workspace command not yet implemented

## 🔥 Immediate Next Steps

1. Run `cargo test` to verify current state after target/ rebuild
2. Finish wiring default setup prompt from `~/.ulf/config.yml`
3. Implement setup prompt background execution with status tracking
4. Commit the uncommitted changes on `feat/multi-workspace-vibe-coding`
5. Decide and document backend consolidation (remove Node backend?)
6. Implement daemon auto-start for `ulf workspace` commands
7. Start Phase 2: `ulf workspace attach` MVP
