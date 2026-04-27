# Migration from v1

Guide for migrating from the Python-based Ulf v1 to the Rust-based v2.

## Overview

Ulf v2 is a complete rewrite in Rust with significant changes:

| Aspect | v1 (Python) | v2 (Rust) |
|--------|-------------|-----------|
| Language | Python | Rust |
| Installation | pip/pipx | npm/cargo |
| Config format | Python dict | YAML |
| Hat system | Not present | Core feature |
| Event system | Not present | Core feature |
| Memories | Not present | Built-in |
| Tasks | Not present | Built-in |
| TUI | Basic | Full ratatui |

## Uninstalling v1

Remove the old Python version first:

```bash
# If installed via pip
pip uninstall ulf-orchestrator

# If installed via pipx
pipx uninstall ulf-orchestrator

# If installed via uv
uv tool uninstall ulf-orchestrator

# Verify removal
which ulf  # Should return nothing
```

## Installing v2

```bash
# Via npm (recommended)
npm install -g @ulf-orchestrator/ulf-cli

# Via GitHub Releases installer
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/mikeyobrien/ulf-orchestrator/releases/latest/download/ulf-cli-installer.sh | sh

# Via Cargo
cargo install ulf-cli
```

## Configuration Changes

### v1 Configuration (Python)

```python
# ulf_config.py
config = {
    "max_iterations": 100,
    "agent": "claude",
    "cost_limit": 10.0,
}
```

### v2 Configuration (YAML)

```yaml
# ulf.yml
cli:
  backend: "claude"

event_loop:
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 100
  checkpoint_gates:
    - name: periodic-check
      trigger: every_n_iterations
      every_n: 10
      command: ["cargo", "test"]
```

**Note:** `checkpoint_interval` from v1 is replaced by the more flexible `checkpoint_gates` system in v2. Instead of a single interval, you define gates with explicit triggers and commands.

## Command Changes

| v1 Command | v2 Command |
|------------|------------|
| `python ulf_orchestrator.py --prompt PROMPT.md` | `ulf run` |
| `python ulf_orchestrator.py --agent claude` | `ulf run --backend claude` |
| `python ulf_orchestrator.py --max-iterations 50` | `ulf run --max-iterations 50` |
| `python ulf_orchestrator.py --dry-run` | `ulf run --dry-run` |

## New Features in v2

### Hat System

Specialized personas that didn't exist in v1:

```yaml
hats:
  planner:
    triggers: ["task.start"]
    publishes: ["plan.ready"]
    instructions: "Create a plan..."
```

### Events

Typed communication between hats:

```bash
ulf emit "build.done" "tests: pass, lint: pass, typecheck: pass, audit: pass, coverage: pass"
ulf events  # View history
```

### Memories

Persistent learning:

```bash
ulf tools memory add "Pattern discovered" -t pattern
ulf tools memory search "pattern"
```

### Tasks

Runtime tracking:

```bash
ulf tools task add "Implement feature"
ulf tools task list
ulf tools task close task-123
```

### Presets

Pre-configured workflows:

```bash
ulf init --preset tdd-red-green
```

### TUI

Rich terminal interface (enabled by default):

```bash
ulf run  # TUI mode
ulf run --no-tui  # Headless mode
```

## Removed Features

Some v1 features are handled differently in v2:

| v1 Feature | v2 Equivalent |
|------------|---------------|
| Cost tracking | Not built-in (use backend's tracking) |
| Loop detection | Simplified (max iterations) |
| ACP protocol | Not supported (direct CLI only) |
| Metrics export | Diagnostics system |

## PROMPT.md Compatibility

The prompt file format is mostly compatible:

```markdown
# Task: My Task

Description here.

## Requirements
- Requirement 1
- Requirement 2
```

**Changes:**

- `- [x] TASK_COMPLETE` marker is no longer used
- Use `LOOP_COMPLETE` in output instead
- Acceptance criteria still work the same

## State Directory

| v1 Location | v2 Location |
|-------------|-------------|
| `.agent/metrics/` | (removed) |
| `.agent/checkpoints/` | Git-based |
| `.agent/prompts/` | (removed) |
| `.agent/plans/` | (removed) |
| (none) | `.agent/memories.md` |
| (none) | `.agent/tasks.jsonl` |
| (none) | `.agent/event_history.jsonl` |

## Migration Steps

### 1. Uninstall v1

```bash
pip uninstall ulf-orchestrator
```

### 2. Install v2

```bash
npm install -g @ulf-orchestrator/ulf-cli
```

### 3. Convert Configuration

Create `ulf.yml` from your old config:

```yaml
cli:
  backend: "claude"  # was "agent"

event_loop:
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 100  # same as before
```

### 4. Update Prompts

Change completion markers:

```markdown
# Before (v1)
- [x] TASK_COMPLETE

# After (v2)
Output: LOOP_COMPLETE
```

### 5. Clean Old State

```bash
rm -rf .agent/metrics .agent/checkpoints .agent/prompts .agent/plans
```

### 6. Test

```bash
ulf run --dry-run
ulf run
```

## Getting Help

If you encounter migration issues:

- Check [Troubleshooting](troubleshooting.md)
- [Open an issue](https://github.com/mikeyobrien/ulf-orchestrator/issues)
- Reference v1 code at [v1.2.3](https://github.com/mikeyobrien/ulf-orchestrator/tree/v1.2.3)
