# Multi-Workspace Vibe-Coding

Ulf's multi-workspace mode lets you create isolated environments per task, run a central daemon, and attach a middle-manager to each workspace. This is ideal for vibe-coding: spinning up focused AI sessions for individual features or bugs without polluting your main project.

## Overview

```
┌─────────────────┐
│   ulf daemon    │  ← Central coordinator (port 3000)
│  workspaces.json│     Tracks all workspaces
└────────┬────────┘
         │
    ┌────┴────┬────────┬────────┐
    ▼         ▼        ▼        ▼
┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐
│ ws-1  │ │ ws-2  │ │ ws-3  │ │ ws-4  │
│ Ready │ │Active │ │Ready  │ │Error  │
└───┬───┘ └───┬───┘ └───┬───┘ └───┬───┘
    │         │         │         │
    ▼         ▼         ▼         ▼
  ~./ulf/workspaces/{id}/  (isolated filesystem)
```

- **Daemon** — HTTP API that tracks workspace state, persists to `~/.ulf/daemon/workspaces.json`
- **Workspace** — Isolated directory under `~/.ulf/workspaces/{id}/` with its own source tree
- **Middle-Manager** — Interactive Ulf session attached to a single workspace
- **Background Loops** — Long-running orchestrations that continue after you detach

## Quick Workflow

### 1. Create a Workspace

```bash
# Simple workspace
ulf workspace create jira-007 --name "Fix login bug"

# From a git repo
ulf workspace create feature-auth --from https://github.com/you/template.git

# With a setup prompt (runs automatically)
ulf workspace create new-api \
  --name "Build REST API" \
  --setup-prompt "Create a Fastify project with TypeScript and tests"

# Wait for setup to complete before returning
ulf workspace create jira-007 --wait
```

Workspaces are created under `~/.ulf/workspaces/` by default. The daemon tracks them in `~/.ulf/daemon/workspaces.json`.

### 2. Attach the Middle-Manager

```bash
ulf workspace attach jira-007
```

This spawns an interactive Ulf session inside the workspace. The middle-manager:

- Explores the workspace to understand the codebase
- Suggests a concrete plan with workflow commands
- Runs background workflows that return immediately (loop ID reported)
- Exiting the session does **not** stop background loops

### 3. Manage Workspaces

```bash
# List all workspaces
ulf workspace list

# Show one workspace
ulf workspace get jira-007

# Check status across all workspaces
ulf workspace status

# Delete a workspace
ulf workspace delete jira-007

# Delete and remove files
ulf workspace delete jira-007 --remove-files
```

### 4. Start Background Workflows

From inside an attached session, or via the API:

```bash
# Background workflow (returns immediately with loop ID)
ulf run -p "Refactor the auth module" --background
```

## Workspace Lifecycle

```
Creating → Ready → Active → (background loops) → Completed/Error
    │
    └── Setup prompt executing (if provided)
```

| Status | Meaning |
|--------|---------|
| `Creating` | Workspace directory being initialized, setup prompt running |
| `Ready` | Workspace initialized, ready for attachment |
| `Active` | Middle-manager currently attached |
| `Completed` | All work finished successfully |
| `Error` | Setup failed or unrecoverable error occurred |

## Configuration

### User-Level Defaults (`~/.ulf/config.yml`)

Set defaults that apply to every workspace:

```yaml
# ~/.ulf/config.yml
workspace:
  # Default setup prompt for new workspaces
  default_setup_prompt: |
    Create a CLAUDE.md for this workspace with:
    - Project overview and tech stack
    - Build, test, and lint commands
    - Code style and architecture conventions

  # Backend presets for per-workspace or per-session overrides
  backend_presets:
    claude-verbose:
      backend: "claude"
      args: ["--verbose"]
    kiro-fast:
      backend: "kiro"
      idle_timeout_secs: 60

  # Middle-manager prompt extensions
  middle_manager:
    backend_preset: "claude-verbose"
    prompt_extensions:
      - "Always run tests before suggesting the task is complete."
      - "Prefer small, focused commits with descriptive messages."
```

### Per-Workspace Config

Each workspace can have its own `ulf.yml` inside the workspace directory. It overlays on top of `~/.ulf/config.yml` via deep merge.

### Backend Presets

Backend presets let you define reusable backend configurations:

```yaml
workspace:
  backend_presets:
    claude-thorough:
      backend: "claude"
      args: ["--verbose", "--allowedTools", "Bash", "Edit"]
    codex-4o:
      backend: "codex"
      args: ["--model", "gpt-4o"]
```

Reference a preset when attaching:

```bash
# Uses the preset from ~/.ulf/config.yml
ulf workspace attach jira-007 --backend-preset claude-thorough
```

### Middle-Manager Configuration

The middle-manager is the interactive session that runs when you `ulf workspace attach`. Configure it with:

```yaml
workspace:
  middle_manager:
    # Which backend preset to use (optional)
    backend_preset: "claude-verbose"

    # Additional instructions appended to every prompt
    prompt_extensions:
      - "Always write tests for new functionality."
      - "Follow the existing code style in this repo."
      - "Use descriptive variable names."

    # Which workflow preset to load (optional)
    workflow_preset: "code-assist"
```

## Daemon Management

```bash
# Start the daemon (auto-starts on workspace commands)
ulf daemon start

# Check daemon status
ulf daemon status

# Stop the daemon
ulf daemon stop

# Restart the daemon
ulf daemon restart

# View daemon logs
ulf daemon logs
```

The daemon logs to `~/.ulf/daemon/daemon.log`.

## Workspace Filesystem Layout

```
~/.ulf/
├── config.yml                    # User-level defaults
├── daemon/
│   ├── workspaces.json           # Registry (atomic writes, 0o600)
│   └── daemon.log                # Daemon logs
└── workspaces/
    ├── jira-007/
    │   ├── src/                  # Source code
    │   ├── ulf.yml               # Workspace-specific config
    │   └── .ulf/
    │       └── agent/
    │           ├── memories.md   # Persistent learning
    │           └── tasks.jsonl   # Task history
    └── feature-auth/
        └── ...
```

## Security Model

Multi-workspace mode is designed for **single-user local use**:

- Workspace IDs are validated: `^[a-zA-Z0-9_.-]{1,64}$`
- Path traversal is blocked (`..` rejected)
- All workspace paths are canonicalized under `~/.ulf/workspaces/`
- Registry writes are atomic (temp file + rename)
- `workspace.update_status` RPC is disabled for external callers
- Setup prompts are passed via file (`-P`) instead of inline (`-p`) to avoid shell injection

## Best Practices

### Naming Conventions

Use descriptive, namespaced workspace IDs:

```bash
# Good: task- or feature-scoped
ulf workspace create jira-007-fix-login
ulf workspace create feat-oauth2
ulf workspace create bug-null-deref

# Bad: vague or sequential
ulf workspace create ws1
ulf workspace create temp
```

### Setup Prompts

Always provide a setup prompt for new workspaces. It runs once on creation and sets the stage for all future work:

```yaml
workspace:
  default_setup_prompt: |
    1. Explore the codebase structure
    2. Create a CLAUDE.md with build/test commands
    3. Identify the tech stack and conventions
    4. Note any existing tests or CI setup
```

### Cleaning Up

Workspaces accumulate over time. Clean up regularly:

```bash
# List and filter
ulf workspace list | grep "Error"

# Delete old workspaces
ulf workspace delete jira-007 --remove-files
```

### Transitioning from Traditional Mode

If you've been using `ulf run` in your project directory, you can migrate to multi-workspace:

```bash
# Create a workspace from your existing project
cd /path/to/project
ulf workspace create my-project --from .

# Or just create fresh and copy what you need
ulf workspace create my-project --setup-prompt "Set up a Rust project with Axum"
```

## Troubleshooting

### Workspace Stuck in "Creating"

After 5 minutes, workspaces stuck in `Creating` are automatically transitioned to `Error`. To recover:

```bash
ulf workspace get <id>
# Check the error field

# Delete and recreate
ulf workspace delete <id> --remove-files
ulf workspace create <id> --wait
```

### Daemon Not Responding

```bash
# Check if daemon is running
ulf daemon status

# Restart it
ulf daemon restart

# Check logs for errors
tail -f ~/.ulf/daemon/daemon.log
```

### Attach Fails

```bash
# Verify workspace exists and is Ready
ulf workspace get <id>

# Check daemon health
ulf daemon status

# Try with explicit backend
ulf workspace attach <id> --backend claude
```

## Comparison: Traditional vs Multi-Workspace

| Aspect | Traditional (`ulf run`) | Multi-Workspace (`ulf workspace`) |
|--------|------------------------|-----------------------------------|
| Scope | Current directory | Isolated `~/.ulf/workspaces/` |
| State | `.ulf/` in project | `~/.ulf/workspaces/{id}/` |
| Daemon | Not required | Required |
| Attach | N/A | Interactive middle-manager session |
| Background | N/A | Background loops supported |
| Best For | Quick tasks, existing projects | Feature work, isolation, vibe-coding |

## Next Steps

- Read the [Configuration](configuration.md) guide for all config options
- Explore [Presets](presets.md) for pre-configured workflows
- Learn about [Hats & Events](../concepts/hats-and-events.md) for complex multi-agent coordination
