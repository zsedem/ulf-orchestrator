# User Guide

Practical guides for using Ulf Orchestrator effectively.

## In This Section

| Guide | Description |
|-------|-------------|
| [Configuration](configuration.md) | Full core config reference |
| [Multi-Workspace](multi-workspace.md) | Vibe-coding with isolated workspaces |
| [Presets](presets.md) | Built-in hat collections |
| [CLI Reference](cli-reference.md) | Command-line interface |
| [Backends](backends.md) | Supported AI backends |
| [Writing Prompts](prompts.md) | Prompt engineering tips |
| [Cost Management](cost-management.md) | Controlling API costs |
| [Telegram Integration](telegram.md) | Human-in-the-loop via Telegram |

## Quick Links

### Getting Started

- Initialize core config: `ulf init --backend claude`
- List built-in hat collections: `ulf init --list-presets`
- Run with hats: `ulf run -c ulf.yml -H builtin:code-assist`

### Running Ulf

- Basic run (core only): `ulf run -c ulf.yml`
- With hats: `ulf run -c ulf.yml -H builtin:debug`
- With inline prompt: `ulf run -c ulf.yml -H builtin:code-assist -p "Implement feature X"`
- Headless mode: `ulf run --no-tui`
- Resume session: `ulf run --continue`

### Monitoring

- View event history: `ulf events`
- Check memories: `ulf tools memory list`
- Check tasks: `ulf tools task list`

## Choosing a Workflow

| Your Situation | Recommended Approach |
|----------------|---------------------|
| Simple task | Core only (no hats) |
| Implementation work | `-H builtin:code-assist` |
| Bug investigation | `-H builtin:debug` |
| Code review | `-H builtin:review` |
| Exploration and architecture mapping | `-H builtin:research` |

## Common Tasks

### Start a New Feature (Multi-Workspace)

```bash
ulf workspace create feat-oauth --name "Add OAuth login"
ulf workspace attach feat-oauth
# Inside: "Build OAuth2 login with Google and GitHub providers"
```

### Start a New Feature (Traditional)

```bash
ulf init --backend claude
ulf run -c ulf.yml -H builtin:code-assist -p "Add OAuth login"
```

### Debug an Issue

```bash
ulf workspace create bug-auth --name "Fix auth failure"
ulf workspace attach bug-auth
# Inside: "Investigate why user authentication fails on mobile"
```

### Review Code

```bash
ulf run -c ulf.yml -H builtin:review -p "Review the changes in src/api/"
```

## Next Steps

Start with [Configuration](configuration.md) to understand all options.
