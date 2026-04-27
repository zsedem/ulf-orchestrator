# CLI Reference

Complete reference for Ulf's command-line interface.

## Global Options

These options are accepted by all commands.

| Option | Description |
|--------|-------------|
| `-c, --config <SOURCE>` | Primary config source (can be specified multiple times). Defaults to `ulf.yml`, or `$ULF_CONFIG` when set. |
| `-H, --hats <SOURCE>` | Hat collection source (`file`, `builtin:<name>`, or URL). |
| `-v, --verbose` | Verbose output |
| `--color <MODE>` | Color output: `auto`, `always`, `never` |
| `-h, --help` | Show help |
| `-V, --version` | Show version |

### Core Config Sources (`-c`)

The `-c` flag specifies where to load **core** configuration from. If not provided, `ulf` falls back to:

1. `$ULF_CONFIG` when present
2. `ulf.yml`

**Core source types:**

| Format | Description |
|--------|-------------|
| `ulf.yml` | Local file path |
| `https://example.com/ulf.core.yml` | Remote URL |
| `core.field=value` | Core config override |

> `-c builtin:<name>` is no longer supported. Use `-H builtin:<name>` for hat collections.

The first non-override core source is used as the base config. Later core overrides replace earlier values.

Backward compatibility: a `-c` config file may still contain `hats`/`events` (single-file combined config).

If `-H/--hats` is provided, it takes precedence over hats in `-c`:
- `hats` and `events` from `-H` replace `hats`/`events` from `-c`
- `event_loop` values from `-H` override matching `event_loop` keys from `-c`
- `-c core.*=...` overrides are still applied last

**Supported override fields:**

| Field | Description |
|-------|-------------|
| `core.scratchpad` | Path to scratchpad file (string shorthand for `scratchpad.path`) |
| `core.specs_dir` | Path to specs directory |

### Hat Collection Sources (`-H`)

The `-H` flag specifies where to load hat collections from.

| Format | Description |
|--------|-------------|
| `hats/feature.yml` | Local hats file |
| `builtin:code-assist` | Built-in hat collection |
| `https://example.com/hats.yml` | Remote hats file |

**Examples:**

```bash
# Core only (hatless)
ulf run -c ulf.yml

# Core + built-in hat collection
ulf run -c ulf.yml -H builtin:code-assist

# Core + file hat collection
ulf run -c ulf.yml -H hats/review.yml

# Core override + hats
ulf run -c ulf.yml -c core.specs_dir=./my-specs -H builtin:debug
```

## Commands

### ulf run

Run the orchestration loop.

```bash
ulf run [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `-p, --prompt <TEXT>` | Inline prompt text |
| `-P, --prompt-file <FILE>` | Prompt file path |
| `--max-iterations <N>` | Override max iterations |
| `--completion-promise <TEXT>` | Override completion trigger |
| `--dry-run` | Show what would execute |
| `--no-tui` | Disable TUI mode |
| `-a, --autonomous` | Force headless mode |
| `--idle-timeout <SECS>` | TUI idle timeout |
| `--exclusive` | Wait for primary loop slot |
| `--no-auto-merge` | Skip automatic merge after worktree loops complete |
| `--skip-preflight` | Skip auto preflight checks (even when `features.preflight.enabled: true`) |
| `--record-session <FILE>` | Record session JSONL |
| `-q, --quiet` | Suppress streaming output |
| `--continue` | Resume from existing state |

### ulf init

Initialize `ulf.yml`.

```bash
ulf init [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--backend <NAME>` | Backend: `claude`, `kiro`, `gemini`, `codex`, `amp`, `copilot`, `opencode`, `pi`, `custom` |
| `--preset <NAME>` | Removed (monolithic presets no longer supported) |
| `--list-presets` | List available built-in hat collections |
| `--force` | Overwrite existing config |

### ulf preflight

Run the preflight check suite.

```bash
ulf preflight [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--format <human|json>` | Output format |
| `--strict` | Treat warnings as failures |
| `--check <NAME>` | Run one or more checks by name |

Default check names:

- `config`
- `hooks`
- `backend`
- `telegram`
- `git`
- `paths`
- `tools`
- `specs`

Notes:

- `--check` can be repeated (for example: `--check hooks --check config`).
- `--strict` fails when there are warnings (not just failures).
- During `ulf run`, auto-preflight uses `features.preflight.skip` to skip checks by these names.

### ulf hooks

Validate hooks configuration and command wiring without starting loop execution.

```bash
ulf hooks <COMMAND>
```

**Subcommands:**

- `validate [--format human|json]`

`ulf hooks validate` behavior:

- Exit code `0`: validation passed.
- Exit code `1`: one or more diagnostics (or config load/parse failure).
- `--format human` (default): readable report with diagnostics.
- `--format json`: structured report (`pass`, `source`, `hooks_enabled`, `checked_hooks`, `diagnostics`).

Try it against the minimal sample hooks config:

- `ulf hooks validate -c examples/hooks/minimal/ulf.hooks.yml`
- Config: [`examples/hooks/minimal/ulf.hooks.yml`](https://github.com/mikeyobrien/ulf-orchestrator/blob/main/examples/hooks/minimal/ulf.hooks.yml)
- Scripts: [`examples/hooks/scripts/env-guard.sh`](https://github.com/mikeyobrien/ulf-orchestrator/blob/main/examples/hooks/scripts/env-guard.sh), [`examples/hooks/scripts/notify.sh`](https://github.com/mikeyobrien/ulf-orchestrator/blob/main/examples/hooks/scripts/notify.sh)

### ulf doctor

Run environment and first-run diagnostic checks.

```bash
ulf doctor [OPTIONS]
```

### ulf tutorial

Run interactive intro walkthrough.

```bash
ulf tutorial [OPTIONS]
```

### ulf plan

Start an interactive PDD planning session.

```bash
ulf plan [OPTIONS] [IDEA]
```

**Options:**

| Option | Description |
|--------|-------------|
| `<IDEA>` | Optional rough idea |
| `-b, --backend <BACKEND>` | Backend override |
| `--teams` | Enable Claude Code agent teams mode |
| `-- <ARGUMENTS>` | Custom backend arguments |

### ulf code-task

Generate code task files from a description or PDD plan.

```bash
ulf code-task [OPTIONS] [INPUT]
```

### ulf task

Deprecated legacy alias for `ulf code-task`.

```bash
ulf task [OPTIONS] [INPUT]
```

### ulf events

View event history for the current or selected run.

```bash
ulf events [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--file <PATH>` | Use a specific events file |
| `--clear` | Clear event history |

### ulf emit

Emit an event to the current run's events file.

```bash
ulf emit <TOPIC> [PAYLOAD] [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `<TOPIC>` | Event topic (e.g., `build.done`) |
| `[PAYLOAD]` | Optional payload (string or JSON when `--json` is set) |
| `-j, --json` | Parse payload as JSON object |
| `--ts <TIMESTAMP>` | Override event timestamp |
| `--file <PATH>` | Events file path (`.ulf/events.jsonl`) |

### ulf clean

Clean `.ulf/agent` scratchpad and memory state.

```bash
ulf clean [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--diagnostics` | Clean diagnostics directory |
| `--dry-run` | Preview deletions |

### ulf loops

Manage parallel loops and worktree loop lifecycle.

```bash
ulf loops [OPTIONS] [COMMAND]
```

**Subcommands:**

- `list [--json] [--all]`
- `history <loop-id> [--json]`
- `retry <loop-id>`
- `discard <loop-id> [--yes]`
- `stop [loop-id] [--force]`
- `resume <loop-id>`
- `prune`
- `attach <loop-id>`
- `diff <loop-id> [--stat]`
- `merge <loop-id> [--force]`
- `process`
- `merge-button-state <loop-id>`

`ulf loops resume <loop-id>` writes a resume signal for suspended loops. It is idempotent:
re-running the command reports that resume was already requested (or that the loop is not suspended).

### ulf hats

Manage and inspect configured hats.

```bash
ulf hats [OPTIONS] [COMMAND]
```

**Subcommands:**

- `list [--format table|json]`
- `show <name>`
- `validate`
- `graph [--format unicode|ascii|compact|mermaid] [--backend <backend>]`

### ulf web

Run the web dashboard.

```bash
ulf web [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--backend-port <BACKEND_PORT>` | RPC API port (default: 3000) |
| `--frontend-port <FRONTEND_PORT>` | Frontend port (default: 5173) |
| `--workspace <WORKSPACE>` | Workspace root |
| `--legacy-node-api` | Run deprecated Node tRPC backend instead of Rust RPC API |
| `--no-open` | Do not open browser |

### ulf mcp

Run Ulf as a Model Context Protocol server over `stdio`.

```bash
ulf mcp serve
```

Notes:

- v1 is tools-only and `stdio`-only.
- Launch it from an MCP client configuration, not an interactive terminal workflow.
- The server exposes Ulf control-plane methods as MCP tools, including polling stream tools such as `stream_next`.

### ulf bot

Manage Telegram bot setup and testing.

```bash
ulf bot [OPTIONS] <COMMAND>
```

**Subcommands:**

- `onboard [--token <TOKEN>] [--chat-id <CHAT_ID>] [--timeout <SECONDS>]`
- `status`
- `test [MESSAGE]`
- `token set <TOKEN> [--config <path>]`
- `daemon`

### ulf workspace

Manage isolated workspaces for multi-workspace vibe-coding.

```bash
ulf workspace <COMMAND>
```

**Subcommands:**

| Command | Description |
|---------|-------------|
| `create <ID>` | Create a new workspace |
| `list` | List all workspaces |
| `get <ID>` | Show workspace details |
| `status` | Check status across all workspaces |
| `attach <ID>` | Attach middle-manager to a workspace |
| `delete <ID>` | Delete a workspace |

**`ulf workspace create` options:**

| Option | Description |
|--------|-------------|
| `<ID>` | Workspace identifier (`^[a-zA-Z0-9_.-]{1,64}$`) |
| `--name <NAME>` | Human-readable name |
| `--from <PATH or URL>` | Clone from git repo or copy from directory |
| `--setup-prompt <TEXT>` | Prompt to run automatically on creation |
| `--wait` | Block until setup completes |

**Examples:**

```bash
# Create a simple workspace
ulf workspace create jira-007 --name "Fix login bug"

# Create from a git repo
ulf workspace create feature-auth --from https://github.com/you/template.git

# Create with auto-setup
ulf workspace create new-api \
  --name "Build REST API" \
  --setup-prompt "Create a Fastify project with TypeScript"

# Attach and work
ulf workspace attach jira-007

# List and manage
ulf workspace list
ulf workspace status
ulf workspace delete jira-007 --remove-files
```

### ulf daemon

Manage the workspace daemon.

```bash
ulf daemon <COMMAND>
```

**Subcommands:**

| Command | Description |
|---------|-------------|
| `start` | Start the daemon |
| `stop` | Stop the daemon |
| `status` | Check daemon status |
| `logs` | *(not yet implemented)* View logs with `tail -f ~/.ulf/daemon/daemon.log` |

### ulf wave

Dispatch wave events for parallel hat execution.

```bash
ulf wave emit <TOPIC> --payloads <ITEM>...
```

**Options:**

| Option | Description |
|--------|-------------|
| `<TOPIC>` | Event topic targeting a wave-capable hat |
| `--payloads <ITEM>...` | One or more payloads, each becomes a separate event |

Each payload becomes an event tagged with a shared `wave_id`. The loop runner spawns parallel backend instances bounded by the target hat's `concurrency` setting.

Blocked when `ULF_WAVE_WORKER=1` (prevents nested waves).

See [Agent Waves](../advanced/agent-waves.md) for full details.

### ulf tools

Runtime tools for memories, tasks, and skills.

#### ulf tools memory

```bash
ulf tools memory <SUBCOMMAND>
```

**Subcommands:**

| Command | Description |
|---------|-------------|
| `init` | Initialize memory file |
| `add <CONTENT>` | Store a new memory |
| `search <QUERY>` | Search memories |
| `list` | List memories |
| `show <ID>` | Show a memory |
| `delete <ID>` | Delete a memory |
| `prime` | Prime context memory output |

#### ulf tools task

```bash
ulf tools task <SUBCOMMAND>
```

**Subcommands:**

| Command | Description |
|---------|-------------|
| `add <TITLE>` | Create a task |
| `list` | List all tasks |
| `ready` | List unblocked tasks |
| `close <ID>` | Mark task complete |
| `fail <ID>` | Mark task failed |
| `show <ID>` | Show task details |

#### ulf tools skill

```bash
ulf tools skill <SUBCOMMAND>
```

#### ulf tools interact

Interact with human via Telegram progress/proactiveness hooks.

### ulf completions

Generate shell completions.

```bash
ulf completions <SHELL>
```

Supported shells: `bash`, `elvish`, `fish`, `powershell`, `zsh`.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Completion promise reached (`LOOP_COMPLETE`) |
| 1 | Failure or stop condition (failure/cancelled/throttled state) |
| 2 | Runtime limits reached (`max-iterations`, `max-runtime`, or `max-cost`) |
| 3 | Loop requested restart |
| 130 | Interrupted by signal (Ctrl-C / SIGINT) |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ULF_DIAGNOSTICS` | Set to `1` to enable diagnostics |
| `ULF_CONFIG` | Default config file path |
| `NO_COLOR` | Disable color output |
| `ULF_WAVE_WORKER` | Set to `1` inside wave workers (blocks nested waves) |
| `ULF_WAVE_ID` | Wave correlation ID (set on wave workers) |
| `ULF_WAVE_INDEX` | 0-based worker index within the wave |
| `ULF_EVENTS_FILE` | Per-worker events file path (set on wave workers) |

## Shell Completion

Generate shell completions:

```bash
# Bash
ulf completions bash > ~/.local/share/bash-completion/completions/ulf

# Zsh
ulf completions zsh > ~/.zfunc/_ulf

# Fish
ulf completions fish > ~/.config/fish/completions/ulf.fish
```
