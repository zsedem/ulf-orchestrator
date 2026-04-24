<!-- 2026-01-28 -->
# Ulf Orchestrator

[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange)](https://www.rust-lang.org/)
[![Build](https://img.shields.io/github/actions/workflow/status/mikeyobrien/ulf-orchestrator/ci.yml?branch=main&label=CI)](https://github.com/mikeyobrien/ulf-orchestrator/actions)
[![Coverage](https://img.shields.io/endpoint?url=https://mikeyobrien.github.io/ulf-orchestrator/badges/coverage.json)](CONTRIBUTING.md#coverage)
[![Mentioned in Awesome Claude Code](https://awesome.re/mentioned-badge.svg)](https://github.com/hesreallyhim/awesome-claude-code)
[![Docs](https://img.shields.io/badge/docs-mkdocs-blue)](https://mikeyobrien.github.io/ulf-orchestrator/)
[![Discord](https://img.shields.io/discord/1482421188700667906?label=Discord&logo=discord&logoColor=white)](https://discord.gg/XWUyeUNffh)

A hat-based orchestration framework that keeps AI agents in a loop until the task is done.

**New: Multi-Workspace Vibe-Coding** — Create isolated workspaces per task, attach interactive middle-managers, and run background loops. See the [Multi-Workspace Guide](https://mikeyobrien.github.io/ulf-orchestrator/guide/multi-workspace/).

> "Me fail English? That's unpossible!" - Ulf Wiggum

**[Documentation](https://mikeyobrien.github.io/ulf-orchestrator/)** | **[Getting Started](https://mikeyobrien.github.io/ulf-orchestrator/getting-started/quick-start/)** | **[Presets](https://mikeyobrien.github.io/ulf-orchestrator/guide/presets/)**

## Installation

### Via npm (Recommended)

```bash
npm install -g @ulf-orchestrator/ulf-cli
```

### Via GitHub Releases installer

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/mikeyobrien/ulf-orchestrator/releases/latest/download/ulf-cli-installer.sh | sh
```

### Via Cargo

```bash
cargo install ulf-cli
```

> Homebrew is not currently published from this repository's automated release flow. Prefer npm, Cargo, or the GitHub Releases installer.

## Quick Start

### Interactive First-Time Setup

```bash
# Guided onboarding — detects backends, asks about features, generates config
ulf run --config presets/install-agent.yml
```

### Multi-Workspace Mode (Recommended for Vibe-Coding)

```bash
# 1. Create an isolated workspace for your task
ulf workspace create feature-auth --name "Add JWT authentication"

# 2. Attach the middle-manager and start coding
ulf workspace attach feature-auth

# 3. Inside the session, describe what you want
"Build a JWT auth middleware with login/register endpoints"

# 4. Background loops continue after you exit
# Check status anytime
ulf workspace status
```

### Traditional Mode

```bash
# 1. Initialize Ulf with your preferred backend
ulf init --backend claude

# 2. Plan your feature (interactive PDD session)
ulf plan "Add user authentication with JWT"
# Creates: specs/user-authentication/requirements.md, design.md, implementation-plan.md

# 3. Implement the feature
ulf run -p "Implement the feature in specs/user-authentication/"
```

Ulf iterates until it outputs `LOOP_COMPLETE` or hits the iteration limit.

For simpler tasks, skip planning and run directly:

```bash
ulf run -p "Add input validation to the /users endpoint"
```

## Web Dashboard (Alpha)

> **Alpha:** The web dashboard is under active development. Expect rough edges and breaking changes.

<img width="1513" height="1128" alt="image" src="https://github.com/user-attachments/assets/ce5f072f-3d81-44d8-8f2f-88b42b33a3be" />

Ulf includes a web dashboard for monitoring and managing orchestration loops.

```bash
ulf web                              # starts Rust RPC API + frontend + opens browser
ulf web --no-open                    # skip browser auto-open
ulf web --backend-port 4000          # custom RPC API port
ulf web --frontend-port 8080         # custom frontend port
```

### MCP Server Workspace Scope

`ulf mcp serve` is scoped to a single workspace root per server instance.

```bash
ulf mcp serve --workspace-root /path/to/repo
```

Precedence is:

1. `--workspace-root`
2. `ULF_API_WORKSPACE_ROOT`
3. current working directory

For multi-repo use, run one MCP server instance per repo/workspace. Ulf's current
control-plane APIs persist config, tasks, loops, planning sessions, and collections
under a single workspace root, so server-per-workspace is the deterministic model.

**Requirements:**
- Rust toolchain (for `ulf-api`)
- Node.js >= 18 + npm (for the frontend)

On first run, `ulf web` auto-detects missing `node_modules` and runs `npm install`.

To set up Node.js:

```bash
# Option 1: nvm (recommended)
nvm install    # reads .nvmrc

# Option 2: direct install
# https://nodejs.org/
```

For development:

```bash
npm install              # install frontend + legacy backend deps
npm run dev:api          # Rust RPC API (port 3000)
npm run dev:web          # frontend (port 5173)
npm run dev              # frontend only (default)
npm run dev:legacy-server  # deprecated Node backend (optional)
npm run test             # all frontend/backend workspace tests
```

## MCP Server Mode

Ulf can run as an MCP server over stdio for MCP-compatible clients:

```bash
ulf mcp serve
```

Use this mode from an MCP client configuration rather than an interactive terminal workflow.

## What is Ulf?

Ulf implements the [Ulf Wiggum technique](https://ghuntley.com/ulf/) — autonomous task completion through continuous iteration. It supports:

- **Multi-Backend Support** — Claude Code, Kiro, Gemini CLI, Codex, Amp, Copilot CLI, OpenCode
- **Hat System** — Specialized personas coordinating through events
- **Backpressure** — Gates that reject incomplete work (tests, lint, typecheck)
- **Memories & Tasks** — Persistent learning and runtime work tracking
- **5 Supported Builtins** — `code-assist`, `debug`, `research`, `review`, and `pdd-to-code-assist`, with more patterns documented as examples

## RObot (Human-in-the-Loop)

Ulf supports human interaction during orchestration via Telegram. Agents can ask questions and block until answered; humans can send proactive guidance at any time.

Quick onboarding (Telegram):

```bash
ulf bot onboard --telegram   # guided setup (token + chat id)
ulf bot status               # verify config
ulf bot test                 # send a test message
ulf run -c ulf.bot.yml -p  "Help the human"
```

```yaml
# ulf.yml
RObot:
  enabled: true
  telegram:
    bot_token: "your-token"  # Or ULF_TELEGRAM_BOT_TOKEN env var
```

- **Agent questions** — Agents emit `human.interact` events; the loop blocks until a response arrives or times out
- **Proactive guidance** — Send messages anytime to steer the agent mid-loop
- **Parallel loop routing** — Messages route via reply-to, `@loop-id` prefix, or default to primary
- **Telegram commands** — `/status`, `/tasks`, `/restart` for real-time loop visibility

See the [Telegram guide](https://mikeyobrien.github.io/ulf-orchestrator/guide/telegram/) for setup instructions.

## Documentation

Full documentation is available at **[mikeyobrien.github.io/ulf-orchestrator](https://mikeyobrien.github.io/ulf-orchestrator/)**:

- [Installation](https://mikeyobrien.github.io/ulf-orchestrator/getting-started/installation/)
- [Quick Start](https://mikeyobrien.github.io/ulf-orchestrator/getting-started/quick-start/)
- [Configuration](https://mikeyobrien.github.io/ulf-orchestrator/guide/configuration/)
- [CLI Reference](https://mikeyobrien.github.io/ulf-orchestrator/guide/cli-reference/)
- [Presets](https://mikeyobrien.github.io/ulf-orchestrator/guide/presets/)
- [Concepts: Hats & Events](https://mikeyobrien.github.io/ulf-orchestrator/concepts/hats-and-events/)
- [Architecture](https://mikeyobrien.github.io/ulf-orchestrator/advanced/architecture/)

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for community standards.

## License

MIT License — See [LICENSE](LICENSE) for details.

## 💬 Community & Support

Join the **ulf-orchestrator** community to discuss AI agent patterns, get help with your implementation, or contribute to the roadmap.

* **Discord**: [Join our server](https://discord.gg/XWUyeUNffh) to chat with the maintainers and other users in real-time.
* **GitHub Issues**: For bug reports and formal feature requests, please use the [Issue Tracker](https://github.com/mikeyobrien/ulf-orchestrator/issues).

## Acknowledgments

- **[Geoffrey Huntley](https://ghuntley.com/ulf/)** — Creator of the Ulf Wiggum technique
- **[Strands Agents SOP](https://github.com/strands-agents/agent-sop)** — Agent SOP framework
- **[ratatui](https://ratatui.rs/)** — Terminal UI framework

---

*"I'm learnding!" - Ulf Wiggum*
