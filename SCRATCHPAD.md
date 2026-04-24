# SCRATCHPAD: Multi-Workspace Vibe-Coding

## Session: c3aa568e

## ✅ Completed (Brief)

- Phase 0: `ralph` → `ulf` rename, completion gates, publish backpressure, Enterprise Dev Pipeline preset
- Phase 1: Workspace types, `WorkspaceDomain`, CRUD + status RPC, domain isolation, `workspace_isolation.rs` tests
- Phase 1.5: Setup prompt wired end-to-end (`~/.ulf/config.yml` → daemon background spawn → status Ready/Error). Fixed `--autonomous`/`--no-tui` conflict.
- Phase 2 MVP: `ulf workspace attach` with embedded middle-manager prompt (hardcoded, no presets yet)
- Phase 3: Frontend deprecated (banner in AppShell)
- Daemon auto-start on `ulf workspace` commands
- All tests green (`cargo test --workspace`)

## 🚨 Devil's Advocate Audit Backlog

### BLOCKERS (Fix First)

| # | Issue | File | Fix |
|---|-------|------|-----|
| B1 | **Path traversal in workspace ID** — `"id": "../../../tmp/pwn"` resolves outside sandbox; `delete --remove-files` nukes arbitrary paths | `workspace_domain.rs:295-302` | Sanitize ID to `[a-zA-Z0-9_-]+`; validate path is under workspace root |
| B2 | **Unrestricted `from` path copies any directory** — `"from": "/home/user/.ssh"` recursively copied | `workspace_domain.rs:101-118` | Restrict `from` to approved source dirs; reject outside paths |
| B3 | **Non-atomic registry write** — crash during `fs::write` = total data loss | `workspace_domain.rs:265-293` | Write to temp file + `rename()` |
| B4 | **Zombie workspaces on daemon crash** — save happens before `tokio::spawn`; crash between = permanent `Creating` | `dispatch.rs:346-362` | Spawn first, then save; or add a watchdog that transitions stale `Creating` workspaces to `Error` after timeout |
| B5 | **Workspace stuck `Creating` forever if no setup prompt** — no auto-transition to `Ready` | `dispatch.rs:354-360` | If `setup_prompt` is None, mark `Ready` immediately after directory creation |
| B6 | **`workspace.update_status` exposed with zero auth** — any client can mark any workspace `Ready` | `dispatch.rs:383-409` | Add authorization check or restrict to daemon-internal use |
| B7 | **Setup prompt = unsandboxed RCE** — arbitrary text from user config passed to `ulf run --autonomous` | `dispatch.rs:549-555` | Add approval gate, timeout, sandbox (e.g. denylist of dangerous commands) |
| B8 | **Prompt injection via workspace ID/path** — `format!()` injects raw strings into LLM prompt | `workspace_cli.rs:309-329` | Escape or validate inputs before `format!()` |
| B9 | **`kill_on_drop(true)` kills setup on daemon restart** — daemon OOM/restart = SIGKILL mid-setup | `dispatch.rs:557` | Remove `kill_on_drop`; track tasks in a `JoinSet` |
| B10 | **`std::sync::Mutex` held across `git clone`** — blocks all concurrent workspace reads | `workspace_domain.rs:74`, `dispatch.rs:344` | Drop mutex before slow I/O; use `tokio::sync::Mutex` if async needed |
| B11 | **Daemon output nulled — crashes invisible** — stdout/stderr to `/dev/null` | `daemon.rs:61-63` | Redirect to `~/.ulf/daemon/daemon.log` instead |

### HIGH

| # | Issue | File | Fix |
|---|-------|------|-----|
| H1 | No `--json` on workspace/daemon commands | `workspace_cli.rs`, `daemon.rs` | Add `--format json/table` |
| H2 | Silent auto-start on read-only queries (`list`, `get`, `status`) | `workspace_cli.rs:129-133` | Only auto-start for mutating commands; add `--no-start` |
| H3 | Fixed 500ms health check race | `daemon.rs:88-96` | Poll with backoff up to 5s |
| H4 | Port 3000 collision — anything on 3000 = "daemon running" | `daemon_client.rs:19-36` | Add PID verification or use unique probe |
| H5 | No RPC timeout — deadlocked daemon = hung CLI forever | `daemon_client.rs:45` | `reqwest::Client::builder().timeout(30s)` |
| H6 | Broken `nohup` template in middle-manager prompt (typo: `"<workflow prompt">`) | `workspace_cli.rs:317` | Fix quote; use `-P prompt_file` instead of `-p` |
| H7 | No ACP feedback from spawned loops to middle-manager | N/A | Design WebSocket/polling feed from worktree loops back to manager session |
| H8 | `workspaces.json` default permissions `0o644` — may contain API keys in setup prompts | `workspace_domain.rs` | Write with `0o600` |

### MEDIUM

| # | Issue | File | Fix |
|---|-------|------|-----|
| M1 | No backend preset system — `attach` hardcodes `ulf` command | `config.rs`, `workspace_cli.rs` | Add `BackendPresetConfig`, `MiddleManagerConfig`, `resolve_backend_preset()` |
| M2 | Middle-manager prompt hardcoded in binary | `workspace_cli.rs:309-329` | Load from config `workspace.middle_manager.prompt_extensions` or prompt file |
| M3 | No `workspace.setup_status` / `workspace.cancel_setup` RPCs | `dispatch.rs` | Add observability and cancel for background setup tasks |
| M4 | Daemon stop never escalates to SIGKILL | `daemon.rs:137-147` | Send SIGKILL after grace period |
| M5 | `--wait` hardcodes 5-min timeout | `workspace_cli.rs:180` | Add `--timeout <seconds>` |
| M6 | Config parse failures silent | `workspace_cli.rs:149-155` | Warn to stderr on parse error |
| M7 | `workspace.status` leaks `.ulf-health-check.tmp` | `workspace_domain.rs:162-166` | Use `tempfile::NamedTempFile` |
| M8 | `copy_dir_all` loses symlinks/permissions | `workspace_domain.rs:306-318` | Use `cp -a` or `fs_extra` with symlink handling |
| M9 | `workspace.update_status` does manual JSON parsing | `dispatch.rs:384-409` | Define `WorkspaceUpdateStatusParams` struct |
| M10 | Blocking `std::fs::write` in async `run_workspace_setup` | `dispatch.rs:541-547` | Use `tokio::fs::write` |
| M11 | `rpc_call` / `rpc_mutate` are 50-line duplicates | `daemon_client.rs` | Extract shared `do_rpc` helper |
| M12 | No `ulf daemon restart` | `daemon.rs:19-29` | Add `DaemonCommands::Restart` |

### LOW

- `attach` hardcodes preset paths that may not exist
- `--wait` dots corrupt terminal on Ctrl-C
- `is_daemon_running` uses POST for health check
- `workspace get` is prose-only, unparseable
- No `ulf daemon logs` command
- `stop_daemon` on Windows is a hard error
- `git clone` progress lost (daemon has no TTY)

## 🔥 Immediate Next Steps

1. **Security blockers**: B1, B2, B6, B7, B8
2. **Reliability blockers**: B3, B4, B5, B9, B10, B11
3. **Configurability**: M1, M2 — add `BackendPresetConfig`, `MiddleManagerConfig`, thread through `attach_workspace`
4. **ACP feedback**: H7 — design loop-to-manager event stream
