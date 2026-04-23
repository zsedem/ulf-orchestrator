# ulf-api

Rust-native bootstrap runtime for the RPC v1 control plane.

## What this crate provides (bootstrap scope)

- HTTP RPC endpoint: `POST /rpc/v1`
- WebSocket stream endpoint: `GET /rpc/v1/stream` (keepalive skeleton)
- Metadata endpoints:
  - `GET /health`
  - `GET /rpc/v1/capabilities`
- Protocol runtime for canonical RPC v1 envelopes
- Shared error envelope mapping (`INVALID_REQUEST`, `METHOD_NOT_FOUND`, etc.)
- Auth abstraction:
  - `trusted_local`
  - `token` mode hook
- Idempotency primitives for mutating methods with in-memory store
- Implemented methods:
  - `system.health`
  - `system.version`
  - `system.capabilities`
  - Full `task.*` family (`list/get/ready/create/update/close/archive/unarchive/delete/clear/run/run_all/retry/cancel/status`)
  - Full `loop.*` family (`list/status/process/prune/retry/discard/stop/merge/merge_button_state/trigger_merge_task`)
  - Full `planning.*` family (`list/get/start/respond/resume/delete/get_artifact`)
  - Full `config.*` family (`get/update`)
  - Full `preset.*` family (`list`)
  - Full `collection.*` family (`list/get/create/update/delete/import/export`)

Persistence notes:
- `task.*` data is persisted in `.ulf/api/tasks-v1.json`
- `loop.*` reads/writes `.ulf/loops.json` and `.ulf/merge-queue.jsonl` via `ulf-core`
- `planning.*` data is persisted under `.ulf/planning-sessions/<session-id>/`
- `collection.*` data is persisted in `.ulf/api/collections-v1.json`
- `config.*` reads/writes `ulf.yml` with YAML validation + atomic replace semantics
- `preset.list` reads builtins from `presets/`, local files from `.ulf/hats/`, and collection-backed presets

Intentional migration differences vs legacy Node backend:
- `task.cancel` currently allows cancelling `pending` tasks (legacy allowed only `running`).
- `planning.start` returns a full `session` object instead of just `{sessionId}`.

## Run locally

From repository root:

```bash
cargo run -p ulf-api
```

For the MCP server:

```bash
./target/debug/ulf mcp serve --workspace-root /path/to/repo
```

The MCP server is workspace-scoped. One server instance manages one workspace root for
`ulf.yml`, `.ulf/api/*`, loops, planning sessions, and collections.

Environment variables:

- `ULF_API_HOST` (default: `127.0.0.1`)
- `ULF_API_PORT` (default: `3000`)
- `ULF_API_SERVED_BY` (default: `ulf-api`)
- `ULF_API_AUTH_MODE` (`trusted_local` or `token`, default: `trusted_local`)
  - `trusted_local` is restricted to loopback hosts (`127.0.0.1`, `::1`, `localhost`)
- `ULF_API_TOKEN` (required for practical token auth use)
- `ULF_API_IDEMPOTENCY_TTL_SECS` (default: `3600`)
- `ULF_API_WORKSPACE_ROOT` (default: current working directory)
- `ULF_API_LOOP_PROCESS_INTERVAL_MS` (default: `30000`)
- `ULF_API_ULF_COMMAND` (default: `ulf`; command used for loop-side-effect parity flows like `loop.retry`)

## Smoke call examples

Health:

```bash
curl -s http://127.0.0.1:3000/health | jq .
```

RPC system health:

```bash
curl -s http://127.0.0.1:3000/rpc/v1 \
  -H 'content-type: application/json' \
  -d '{
    "apiVersion": "v1",
    "id": "req-health-1",
    "method": "system.health",
    "params": {}
  }' | jq .
```
