# Getting Started

Welcome to Ulf Orchestrator! This section will help you get up and running quickly.

## What You'll Learn

1. **[Installation](installation.md)** — Install Ulf and its prerequisites
2. **[Quick Start](quick-start.md)** — Run your first Ulf orchestration
3. **[Your First Task](first-task.md)** — Create and configure a real task
4. **[Multi-Workspace Guide](../guide/multi-workspace.md)** — Vibe-coding with isolated workspaces

## Prerequisites

Before you begin, ensure you have:

- **Rust 1.75+** (if building from source)
- **At least one AI CLI tool** installed:
    - [Claude Code](https://github.com/anthropics/claude-code) (recommended)
    - [Kiro](https://kiro.dev/)
    - [Gemini CLI](https://github.com/google-gemini/gemini-cli)
    - [Codex](https://github.com/openai/codex)
    - [Amp](https://github.com/sourcegraph/amp)
    - [Copilot CLI](https://docs.github.com/copilot)
    - [OpenCode](https://opencode.ai/)

## Quick Installation

=== "npm (Recommended)"

    ```bash
    npm install -g @ulf-orchestrator/ulf-cli
    ```

=== "GitHub Releases installer"

    ```bash
    curl --proto '=https' --tlsv1.2 -LsSf \
      https://github.com/mikeyobrien/ulf-orchestrator/releases/latest/download/ulf-cli-installer.sh | sh
    ```

=== "Cargo"

    ```bash
    cargo install ulf-cli
    ```

## Verify Installation

```bash
ulf --version
ulf --help
```

## Next Steps

Once installed, head to the [Quick Start](quick-start.md) guide to run your first orchestration.
