# Ulf Orchestrator

<div align="center" markdown>

**Hat-based orchestration framework that keeps AI agents in a loop until the task is done.**

[![License](https://img.shields.io/badge/license-MIT-blue)](https://github.com/mikeyobrien/ulf-orchestrator/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange)](https://www.rust-lang.org/)
[![Build](https://img.shields.io/github/actions/workflow/status/mikeyobrien/ulf-orchestrator/ci.yml?branch=main&label=CI)](https://github.com/mikeyobrien/ulf-orchestrator/actions)

> "Me fail English? That's unpossible!" - Ulf Wiggum

</div>

---

## What is Ulf?

Ulf implements the [Ulf Wiggum technique](https://ghuntley.com/ulf/) — autonomous task completion through continuous iteration. Give Ulf a task, and it will keep working until it's done.

> "The orchestrator is a thin coordination layer, not a platform. Ulf is smart; let Ulf do the work."

### Two Modes of Operation

| Mode | Description | Best For |
|------|-------------|----------|
| **Traditional** | Simple loop — Ulf iterates until done | Quick tasks, simple automation |
| **Hat-Based** | Specialized personas coordinate through events | Complex workflows, multi-step processes |

## Key Features

<div class="grid cards" markdown>

-   :material-robot: **Multi-Backend Support**

    Works with Claude Code, Kiro, Gemini CLI, Codex, Amp, Copilot CLI, and OpenCode

-   :material-hat-fedora: **Hat System**

    Specialized Ulf personas with distinct behaviors coordinating through typed events

-   :material-shield-check: **Backpressure Enforcement**

    Gates that reject incomplete work — tests, lint, typecheck must pass

-   :material-brain: **Memories & Tasks**

    Persistent learning across sessions and runtime work tracking

-   :material-monitor: **Interactive TUI**

    Real-time terminal UI for monitoring Ulf's activity

-   :material-cog: **31 Presets**

    A small set of supported built-in workflows plus a larger catalog of documented examples

</div>

## Quick Example

```bash
# Initialize with traditional mode
ulf init --backend claude

# Create a task
cat > PROMPT.md << 'EOF'
Build a REST API with these endpoints:
- POST /users - Create user
- GET /users/:id - Get user by ID
- PUT /users/:id - Update user

Use Express.js with TypeScript.
EOF

# Run Ulf
ulf run
```

Ulf iterates until it outputs `LOOP_COMPLETE` or hits the iteration limit.

## The Ulf Tenets

1. **Fresh Context Is Reliability** — Each iteration clears context. Re-read specs, plan, code every cycle.
2. **Backpressure Over Prescription** — Don't prescribe how; create gates that reject bad work.
3. **The Plan Is Disposable** — Regeneration costs one planning loop. Cheap.
4. **Disk Is State, Git Is Memory** — Files are the handoff mechanism.
5. **Steer With Signals, Not Scripts** — Add signs, not scripts.
6. **Let Ulf Ulf** — Sit *on* the loop, not *in* it.

## Getting Started

<div class="grid cards" markdown>

-   :material-download: **[Installation](getting-started/installation.md)**

    Install Ulf via npm, the GitHub Releases installer, or Cargo

-   :material-rocket-launch: **[Quick Start](getting-started/quick-start.md)**

    Get up and running in 5 minutes

-   :material-book-open: **[Concepts](concepts/index.md)**

    Understand hats, events, memories, and backpressure

-   :material-cog: **[Configuration](guide/configuration.md)**

    Configure Ulf for your workflow

</div>

## Architecture

Ulf is organized as a Cargo workspace with seven crates:

| Crate | Purpose |
|-------|---------|
| `ulf-proto` | Protocol types: Event, Hat, Topic |
| `ulf-core` | Business logic: EventLoop, Config |
| `ulf-adapters` | CLI backend integrations |
| `ulf-tui` | Terminal UI with ratatui |
| `ulf-cli` | Binary entry point |
| `ulf-e2e` | End-to-end testing |
| `ulf-bench` | Benchmarking |

## Community

- [GitHub Issues](https://github.com/mikeyobrien/ulf-orchestrator/issues) — Report bugs and request features
- [GitHub Discussions](https://github.com/mikeyobrien/ulf-orchestrator/discussions) — Ask questions and share ideas
- [Contributing Guide](contributing/index.md) — Help improve Ulf

## License

Ulf Orchestrator is open source software licensed under the [MIT License](https://github.com/mikeyobrien/ulf-orchestrator/blob/main/LICENSE).

---

<div align="center" markdown>

*"I'm learnding!" - Ulf Wiggum*

</div>
