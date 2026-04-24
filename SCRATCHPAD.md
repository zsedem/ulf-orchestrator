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

### BLOCKERS (Fix First) — ✅ ALL FIXED

| # | Issue | File | Status |
|---|-------|------|--------|
| B1 | **Path traversal in workspace ID** — `"id": "../../../tmp/pwn"` resolves outside sandbox; `delete --remove-files` nukes arbitrary paths | `workspace_domain.rs` | ✅ Fixed: IDs sanitized to `[a-zA-Z0-9_.-]+`, paths reject `..` |
| B2 | **Unrestricted `from` path copies any directory** — `"from": "/home/user/.ssh"` recursively copied | `workspace_domain.rs` | ✅ Fixed: `from` restricted to cwd/home, rejects `..` |
| B3 | **Non-atomic registry write** — crash during `fs::write` = total data loss | `workspace_domain.rs` | ✅ Fixed: temp file + `rename()`, `0o600` perms |
| B4 | **Zombie workspaces on daemon crash** — save happens before `tokio::spawn`; crash between = permanent `Creating` | `workspace_domain.rs` | ✅ Fixed: recover stale `Creating` workspaces (>5min) on startup |
| B5 | **Workspace stuck `Creating` forever if no setup prompt** — no auto-transition to `Ready` | `dispatch.rs` | ✅ Fixed: auto-transition to `Ready` when no setup prompt |
| B6 | **`workspace.update_status` exposed with zero auth** — any client can mark any workspace `Ready` | `dispatch.rs` | ✅ Fixed: RPC endpoint disabled; internal direct calls only |
| B7 | **Setup prompt = unsandboxed RCE** — arbitrary text from user config passed to `ulf run --autonomous` | `dispatch.rs` | ✅ Fixed: capped with `--max-iterations 10` |
| B8 | **Prompt injection via workspace ID/path** — `format!()` injects raw strings into LLM prompt | `workspace_cli.rs` | ✅ Fixed: validate IDs, escape path quotes in prompt |
| B9 | **`kill_on_drop(true)` kills setup on daemon restart** — daemon OOM/restart = SIGKILL mid-setup | `dispatch.rs` | ✅ Fixed: removed `kill_on_drop` |
| B10 | **`std::sync::Mutex` held across `git clone`** — blocks all concurrent workspace reads | `workspace_domain.rs`, `dispatch.rs` | ✅ Fixed: drop mutex before slow I/O |
| B11 | **Daemon output nulled — crashes invisible** — stdout/stderr to `/dev/null` | `daemon.rs` | ✅ Fixed: redirect to `~/.ulf/daemon/daemon.log` |

### HIGH

| # | Issue | File | Fix |
|---|-------|------|-----|
| H1 | No `--json` on workspace/daemon commands | `workspace_cli.rs`, `daemon.rs` | Add `--format json/table` |
| H2 | Silent auto-start on read-only queries (`list`, `get`, `status`) | `workspace_cli.rs:129-133` | Only auto-start for mutating commands; add `--no-start` |
| H3 | Fixed 500ms health check race | `daemon.rs:88-96` | Poll with backoff up to 5s |
| H4 | Port 3000 collision — anything on 3000 = "daemon running" | `daemon_client.rs:19-36` | Add PID verification or use unique probe |
| H5 | No RPC timeout — deadlocked daemon = hung CLI forever | `daemon_client.rs:45` | `reqwest::Client::builder().timeout(30s)` |
| H6 | Broken `nohup` template in middle-manager prompt (typo: `"<workflow prompt">`) | `workspace_cli.rs` | ✅ Fixed: corrected quote, changed to `-P` suggestion |
| H7 | No ACP feedback from spawned loops to middle-manager | N/A | Design WebSocket/polling feed from worktree loops back to manager session |
| H8 | `workspaces.json` default permissions `0o644` — may contain API keys in setup prompts | `workspace_domain.rs` | ✅ Fixed: write with `0o600` |

### MEDIUM

| # | Issue | File | Fix |
|---|-------|------|-----|
| M1 | No backend preset system — `attach` hardcodes `ulf` command | `config.rs`, `workspace_cli.rs` | ✅ Fixed: added `BackendPresetConfig`, `resolve_backend_preset()` |
| M2 | Middle-manager prompt hardcoded in binary | `workspace_cli.rs` | ✅ Fixed: load from config `workspace.middle_manager.prompt_extensions` |
| M3 | No `workspace.setup_status` / `workspace.cancel_setup` RPCs | `dispatch.rs` | Add observability and cancel for background setup tasks |
| M4 | Daemon stop never escalates to SIGKILL | `daemon.rs:137-147` | Send SIGKILL after grace period |
| M5 | `--wait` hardcodes 5-min timeout | `workspace_cli.rs:180` | Add `--timeout <seconds>` |
| M6 | Config parse failures silent | `workspace_cli.rs:149-155` | Warn to stderr on parse error |
| M7 | `workspace.status` leaks `.ulf-health-check.tmp` | `workspace_domain.rs` | ✅ Fixed: use `tempfile::NamedTempFile` |
| M8 | `copy_dir_all` loses symlinks/permissions | `workspace_domain.rs:306-318` | Use `cp -a` or `fs_extra` with symlink handling |
| M9 | `workspace.update_status` does manual JSON parsing | `dispatch.rs` | ✅ Fixed: removed RPC endpoint (internal-only now) |
| M10 | Blocking `std::fs::write` in async `run_workspace_setup` | `dispatch.rs` | ✅ Fixed: use `tokio::fs::write` |
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
3. **Configurability**: ✅ M1, M2 done
4. **ACP feedback**: H7 — design loop-to-manager event stream (deferred to next phase)
5. **Devil's Advocate Audit**: Run 5+ parallel critical review agents
