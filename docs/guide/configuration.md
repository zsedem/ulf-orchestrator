# Configuration

Complete reference for Ulf's YAML configuration.

## Configuration File

Ulf composes configuration from up to three layers:

1. `~/.ulf/config.yml` when present — user-level defaults loaded automatically
2. `ulf.yml` in the current workspace (or `$ULF_CONFIG` / `-c <file>`) — project-level overrides
3. `-c core.field=value` overrides — applied last

Project config overlays on top of the user config via deep merge. Mappings are merged recursively and scalar values or arrays from the project config replace the user-level value.

```bash
# Use the workspace config (and automatically merge ~/.ulf/config.yml if present)
ulf run

# Override the project config path
ULF_CONFIG=/path/to/config.yml ulf run ...
ulf run -c custom-config.yml
```

### User-level config (`~/.ulf/config.yml`)

Use `~/.ulf/config.yml` for defaults you want everywhere, such as shared backend settings, global lifecycle hooks, or organization-wide guardrails.

A common pattern is keeping notification hooks global while leaving project-specific automation in the repo-local `ulf.yml`:

```yaml
# ~/.ulf/config.yml
hooks:
  enabled: true
  events:
    post.loop.complete:
      - name: notify-success
        command: ["./scripts/notify.sh", "complete"]
        on_error: warn
    post.loop.error:
      - name: notify-failure
        command: ["./scripts/notify.sh", "error"]
        on_error: warn
```

```yaml
# ./ulf.yml
hooks:
  events:
    pre.loop.start:
      - name: env-guard
        command: ["./scripts/hooks/env-guard.sh"]
        on_error: block
```

With those two files, Ulf loads both and deep-merges them before validation and execution.

## MCP Workspace Resolution

`ulf mcp serve` resolves its workspace root in this order:

1. `--workspace-root <path>`
2. `ULF_API_WORKSPACE_ROOT`
3. current working directory

Use one MCP server instance per workspace/repo. Ulf's current control-plane APIs are
workspace-scoped: `config.*`, `task.*`, `loop.*`, `planning.*`, and `collection.*` all
read or persist state under a single root.

## CLI Config Overrides

You can override specific core fields from the command line without creating a separate config file. This is useful for:

- Running parallel Ulf instances with isolated scratchpads
- Testing with different specs directories
- CI/CD pipelines with dynamic paths

**Syntax:** `-c core.field=value`

**Supported fields:**

| Field | Description |
|-------|-------------|
| `core.scratchpad` | Path to scratchpad file (string shorthand for `scratchpad.path`) |
| `core.specs_dir` | Path to specs directory |

**Examples:**

```bash
# Override scratchpad (loads ulf.yml + applies override)
ulf run -c core.scratchpad=.ulf/agent/feature-auth/scratchpad.md

# Explicit config + override
ulf run -c ulf.yml -c core.scratchpad=.ulf/agent/feature-auth/scratchpad.md

# Multiple overrides
ulf run -c core.scratchpad=.runs/task-1/scratchpad.md -c core.specs_dir=./custom-specs/
```

Overrides are applied after `ulf.yml` is loaded, so they take precedence. The scratchpad directory is auto-created if it doesn't exist.

## Combined Config Compatibility (`-c` + `-H`)

Ulf supports both styles:
- **Single-file combined config**: `-c ulf.yml` with core + hats in one file
- **Split config**: `-c <core>` plus `-H <hats source>`

If both are used (`-c` contains hats and `-H` is provided), `-H` wins for workflow sections:
- `hats` and `events` from `-H` replace `hats`/`events` from `-c`
- `event_loop` values from `-H` override matching `event_loop` keys from `-c`
- `-c core.*=...` overrides still apply last

## Full Configuration Reference

```yaml
# Event loop settings
event_loop:
  completion_promise: "LOOP_COMPLETE"  # Output that signals completion
  max_iterations: 100                   # Maximum orchestration loops
  max_runtime_seconds: 14400            # 4 hours max runtime
  idle_timeout_secs: 1800               # 30 min idle timeout
  starting_event: "task.start"          # First event published (hat mode)
  prompt_file: "PROMPT.md"              # Default prompt file

  # Completion gates run when LOOP_COMPLETE is emitted
  completion_gates:
    - name: tests-pass
      command: ["cargo", "test"]
      timeout_seconds: 120

  # Checkpoint gates run mid-session at iteration boundaries
  checkpoint_gates:
    - name: lint-check
      trigger: every_n_iterations
      every_n: 5
      command: ["cargo", "clippy"]
    - name: tests-after-build
      trigger: after_event
      after_event: dev.done
      command: ["cargo", "test"]

# CLI backend settings
cli:
  backend: "claude"                     # Backend name
  prompt_mode: "arg"                    # arg or stdin

# Core behaviors
core:
  scratchpad:                            # Scratchpad configuration
    enabled: true                        # Enable scratchpad (default: true)
    path: .ulf/agent/scratchpad.md     # Scratchpad file path
  specs_dir: "./specs/"                  # Specifications directory
  guardrails:                            # Rules injected into every prompt
    - "Fresh context each iteration"
    - "Never modify production database"

# Memories — persistent learning
memories:
  enabled: true                         # Enable memory system
  inject: auto                          # auto, manual, none
  budget: 2000                          # Max tokens to inject
  filter:
    types: []                           # Filter by memory type
    tags: []                            # Filter by memory tags
    recent: 0                           # Days limit (0 = no limit)

# Tasks — runtime work tracking
tasks:
  enabled: true                         # Enable task system

# Optional features
features:
  parallel: true                        # Allow worktree loops when primary lock is held
  auto_merge: false                     # Auto-merge worktree loops on completion
  preflight:
    enabled: false                      # Run preflight automatically on `ulf run`
    strict: false                       # Treat warnings as failures
    skip: []                            # Skip checks by name (for example: ["hooks"])

# Lifecycle hooks (v1)
hooks:
  enabled: false
  defaults:
    timeout_seconds: 30
    max_output_bytes: 8192
    suspend_mode: wait_for_resume
  events:
    pre.loop.start:
      - name: env-guard
        command: ["./scripts/hooks/env-guard.sh"]
        on_error: block
        mutate:
          enabled: false

# Hats — specialized personas
hats:
  my_hat:
    name: "My Hat"                      # Display name
    description: "Purpose"              # Optional description
    triggers: ["event.*"]               # Subscription patterns
    publishes: ["event.done"]           # Allowed event types
    default_publishes: "event.done"     # Default when no explicit
    max_activations: 10                 # Activation limit
    backend: "claude"                   # Backend override
    scratchpad:                         # Per-hat scratchpad override
      enabled: true                     #   Enable scratchpad (default: true)
      path: .ulf/agent/my-hat.md      #   Scratchpad file path. Inherits from core if omitted.
    instructions: |
      Hat-specific instructions...
```

## Section Details

### event_loop

Controls the orchestration loop behavior.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `completion_promise` | string | `"LOOP_COMPLETE"` | Output text that ends the loop |
| `max_iterations` | integer | `100` | Maximum iterations before stopping |
| `max_runtime_seconds` | integer | `14400` | Maximum runtime (4 hours) |
| `idle_timeout_secs` | integer | `1800` | Idle timeout (30 minutes) |
| `starting_event` | string | `null` | First event (enables hat mode) |
| `prompt_file` | string | `"PROMPT.md"` | Default prompt file |
| `completion_gates` | array | `[]` | Scripts run at LOOP_COMPLETE |
| `checkpoint_gates` | array | `[]` | Scripts run at iteration boundaries |

### cli

Backend configuration.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `backend` | string | auto-detect | Backend name |
| `prompt_mode` | string | `"arg"` | How prompt is passed |

**Backend values:**
- `claude` — Claude Code
- `kiro` — Kiro
- `gemini` — Gemini CLI
- `codex` — Codex
- `amp` — Amp
- `copilot` — Copilot CLI
- `opencode` — OpenCode
- `pi` — Pi
- `custom` — Custom adapter/backend

**Prompt mode values:**
- `arg` — Pass as CLI argument: `cli -p "prompt"`
- `stdin` — Pass via stdin: `echo "prompt" | cli`

### core

Core behaviors, scratchpad, and guardrails.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `scratchpad` | string or object | `{ enabled: true, path: ".ulf/agent/scratchpad.md" }` | Scratchpad configuration (see below) |
| `scratchpad.enabled` | boolean | `true` | Enable the scratchpad |
| `scratchpad.path` | string | `".ulf/agent/scratchpad.md"` | Scratchpad file path |
| `specs_dir` | string | `"./specs/"` | Specifications directory |
| `guardrails` | list | `[]` | Rules injected into every prompt |

The `scratchpad` field accepts a plain string (shorthand for setting `path` with `enabled: true`) or a structured object with `enabled` and `path`:

```yaml
# String shorthand — sets path, enabled defaults to true
core:
  scratchpad: ".workspace/plan.md"

# Structured object — full control
core:
  scratchpad:
    enabled: true
    path: .ulf/agent/scratchpad.md
```

> **Solo mode safety:** If scratchpad is disabled (`enabled: false`) but no hats are defined, Ulf force-enables it with a warning. Scratchpad is the only continuity mechanism in solo mode.

### memories

Persistent learning across sessions.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable memory system |
| `inject` | string | `"auto"` | Injection mode |
| `budget` | integer | `2000` | Max tokens to inject |
| `filter.types` | list | `[]` | Filter by memory type |
| `filter.tags` | list | `[]` | Filter by tags |
| `filter.recent` | integer | `0` | Days limit |

**Injection modes:**
- `auto` — Automatically inject at iteration start
- `manual` — Agent must call `ulf tools memory prime`
- `none` — No injection

### tasks

Runtime work tracking.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable task system |

### features

Optional runtime capabilities.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `parallel` | boolean | `true` | Spawn worktree loops when another loop holds the primary lock |
| `auto_merge` | boolean | `false` | Auto-merge completed worktree loops |
| `preflight.enabled` | boolean | `false` | Run `ulf preflight` checks automatically before `ulf run` |
| `preflight.strict` | boolean | `false` | Treat preflight warnings as failures |
| `preflight.skip` | list | `[]` | Skip checks by name (for example `hooks`, `git`) |

When `features.preflight.enabled: true`, `ulf run` uses the default preflight suite:
`config`, `hooks`, `backend`, `telegram`, `git`, `paths`, `tools`, and `specs`.

### hooks

Lifecycle hooks for orchestrator phase-events (v1).

Hooks can be defined in either the user-level `~/.ulf/config.yml` or the workspace `ulf.yml`. Ulf loads the user config first, then overlays the project config on top. That means hooks in the user config apply globally unless the project config replaces the same event mapping.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable hook dispatch for lifecycle events |
| `defaults.timeout_seconds` | integer | `30` | Default per-hook timeout in seconds |
| `defaults.max_output_bytes` | integer | `8192` | Default stdout/stderr cap per stream |
| `defaults.suspend_mode` | enum | `wait_for_resume` | Default suspend mode for `on_error: suspend` |
| `events` | map | `{}` | Mapping from lifecycle phase-event key to list of hook specs |

Supported v1 lifecycle phase-event keys under `hooks.events`:

- `pre.loop.start`, `post.loop.start`
- `pre.iteration.start`, `post.iteration.start`
- `pre.plan.created`, `post.plan.created`
- `pre.human.interact`, `post.human.interact`
- `pre.loop.complete`, `post.loop.complete`
- `pre.loop.error`, `post.loop.error`

Hook spec (`HookSpec`) fields:

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Stable identifier used in telemetry/diagnostics |
| `command` | Yes | Command argv array (`command[0]` must resolve to an executable) |
| `cwd` | No | Working directory override (absolute or workspace-relative) |
| `env` | No | Environment variable overrides for the hook process |
| `timeout_seconds` | No | Per-hook timeout override (must be > 0) |
| `max_output_bytes` | No | Per-hook output cap override per stream (must be > 0) |
| `on_error` | Yes | Failure disposition: `warn`, `block`, or `suspend` |
| `suspend_mode` | No | Suspend strategy override (`wait_for_resume`, `retry_backoff`, `wait_then_retry`) |
| `mutate.enabled` | No | Opt-in hook stdout mutation parsing (default `false`) |
| `mutate.format` | No | Optional format guardrail; only `json` is allowed in v1 |

Mutation scope in v1 is intentionally narrow:

- Mutation parsing only happens when `mutate.enabled: true`.
- Hook stdout must be JSON using the v1 contract: `{"metadata": { ... }}`.
- Only metadata namespace updates are allowed (`metadata.accumulated.hook_metadata.<hook_name>`).
- Prompt/event/config mutation is out of scope for v1.

Minimal runnable example:

- Config: [`examples/hooks/minimal/ulf.hooks.yml`](https://github.com/mikeyobrien/ulf-orchestrator/blob/main/examples/hooks/minimal/ulf.hooks.yml)
- Scripts: [`examples/hooks/scripts/env-guard.sh`](https://github.com/mikeyobrien/ulf-orchestrator/blob/main/examples/hooks/scripts/env-guard.sh), [`examples/hooks/scripts/notify.sh`](https://github.com/mikeyobrien/ulf-orchestrator/blob/main/examples/hooks/scripts/notify.sh)
- Validate: `ulf hooks validate -c examples/hooks/minimal/ulf.hooks.yml`

### hats

Specialized personas for hat-based mode.

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `name` | string | Yes | Display name |
| `description` | string | No | Purpose description |
| `triggers` | list | Yes | Event subscription patterns |
| `publishes` | list | Yes | Allowed event types |
| `default_publishes` | string | No | Default event if none explicit |
| `max_activations` | integer | No | Limit activations |
| `backend` | string | No | Backend override |
| `scratchpad` | string or object | No | Per-hat scratchpad override (inherits `core.scratchpad` if omitted) |
| `instructions` | string | Yes | Hat-specific prompt |

Each hat can override the global scratchpad with its own `scratchpad` field. Like the core-level setting, it accepts a plain string or a structured object:

```yaml
hats:
  planner:
    scratchpad: .ulf/agent/planner.md       # String shorthand
    # ...
  builder:
    scratchpad:
      path: .ulf/agent/builder.md           # Structured with custom path
    # ...
  validator:
    scratchpad:
      enabled: false                          # Disable scratchpad entirely
    # ...
  reviewer:                                   # No scratchpad key = inherits global
    # ...
```

**Resolution order:** hat override → `core.scratchpad` → defaults.

## Example Configurations

### Traditional Mode (Minimal)

```yaml
cli:
  backend: "claude"

event_loop:
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 100
```

### Hat-Based Mode

```yaml
cli:
  backend: "claude"

event_loop:
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 100
  starting_event: "task.start"

hats:
  planner:
    name: "Planner"
    triggers: ["task.start"]
    publishes: ["plan.ready"]
    instructions: |
      Create an implementation plan.

  builder:
    name: "Builder"
    triggers: ["plan.ready"]
    publishes: ["build.done"]
    instructions: |
      Implement the plan.
      Evidence required: tests pass.
```

### With Memories Disabled

```yaml
cli:
  backend: "claude"

event_loop:
  completion_promise: "LOOP_COMPLETE"

memories:
  enabled: false

tasks:
  enabled: false
```

### With Per-Hat Scratchpads

```yaml
cli:
  backend: "claude"

event_loop:
  completion_promise: "LOOP_COMPLETE"
  starting_event: "task.start"

core:
  scratchpad:
    enabled: true
    path: .ulf/agent/scratchpad.md

hats:
  planner:
    name: "Planner"
    scratchpad:
      path: .ulf/agent/planner.md
    triggers: ["task.start"]
    publishes: ["plan.ready"]
    instructions: |
      Create an implementation plan.

  builder:
    name: "Builder"
    triggers: ["plan.ready"]
    publishes: ["build.done"]
    instructions: |
      Implement the plan.

  reviewer:
    name: "Reviewer"
    scratchpad:
      enabled: false
    triggers: ["build.done"]
    publishes: ["review.done"]
    instructions: |
      Review the implementation. No scratchpad needed.
```

### With Custom Guardrails

```yaml
cli:
  backend: "claude"

event_loop:
  completion_promise: "LOOP_COMPLETE"

core:
  guardrails:
    - "Always run tests before declaring done"
    - "Never modify production database"
    - "Follow existing code patterns"
```

## Workspace Configuration

When using multi-workspace mode, the `workspace` section configures default behaviors for `ulf workspace create` and `ulf workspace attach`.

### workspace

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `default_setup_prompt` | string | `null` | Prompt executed automatically when a workspace is created |
| `backend_presets` | map | `{}` | Named backend configurations (see below) |
| `middle_manager` | object | `{}` | Configuration for `ulf workspace attach` sessions |

### workspace.backend_presets

Named backend configurations referenced by `middle_manager.backend_preset` or `--backend-preset`.

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `backend` | string | Yes | Backend name (`claude`, `kiro`, `codex`, etc.) |
| `args` | list | No | Extra CLI arguments for the backend |
| `prompt_mode` | string | No | `arg` or `stdin` |
| `prompt_flag` | string | No | Custom prompt flag (e.g., `-p`, `--prompt`) |
| `idle_timeout_secs` | integer | No | Session idle timeout |

Example:

```yaml
workspace:
  backend_presets:
    claude-thorough:
      backend: "claude"
      args: ["--verbose", "--allowedTools", "Bash", "Edit"]
    kiro-minimal:
      backend: "kiro"
      args: ["--no-confirm"]
      idle_timeout_secs: 300
```

### workspace.middle_manager

Configuration for the interactive middle-manager session.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `backend_preset` | string | `null` | Name of a `backend_presets` entry to use |
| `prompt_extensions` | list | `[]` | Extra instructions appended to every prompt |
| `workflow_preset` | string | `null` | Hat collection preset to load (e.g., `code-assist`) |

Example:

```yaml
workspace:
  middle_manager:
    backend_preset: "claude-thorough"
    prompt_extensions:
      - "Always run tests before suggesting the task is complete."
      - "Prefer small, focused commits with descriptive messages."
      - "Follow the existing code style in this repo."
    workflow_preset: "code-assist"
```

### Full Workspace Config Example

```yaml
# ~/.ulf/config.yml — User-level workspace defaults
workspace:
  default_setup_prompt: |
    Create a CLAUDE.md for this workspace with:
    - Project overview and tech stack
    - Build, test, and lint commands
    - Code style and architecture conventions

  backend_presets:
    claude-verbose:
      backend: "claude"
      args: ["--verbose"]
    codex-4o:
      backend: "codex"
      args: ["--model", "gpt-4o"]
    kiro-fast:
      backend: "kiro"
      idle_timeout_secs: 60

  middle_manager:
    backend_preset: "claude-verbose"
    prompt_extensions:
      - "Always run tests before suggesting done."
      - "Prefer descriptive commit messages."
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ULF_CONFIG` | Default config file path |
| `ULF_DIAGNOSTICS` | Enable diagnostics (`1`) |
| `NO_COLOR` | Disable color output |

## Next Steps

- Read the [Multi-Workspace Guide](multi-workspace.md)
- Explore [Presets](presets.md) for pre-configured workflows
- Learn about [CLI Reference](cli-reference.md)
- Understand [Backends](backends.md)
