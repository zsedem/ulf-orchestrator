# ulf-cli

Binary entry point and CLI parsing.

## Overview

`ulf-cli` is the main binary that:

- Parses command-line arguments
- Routes to command handlers
- Configures runtime logging/output behavior

## Top-Level Commands

The `Commands` enum in `crates/ulf-cli/src/main.rs` currently includes:

- `run`
- `preflight`
- `hooks`
- `doctor`
- `tutorial`
- `events`
- `init`
- `clean`
- `emit`
- `plan`
- `code-task` (plus hidden legacy `task` alias)
- `tools`
- `loops`
- `hats`
- `tui`
- `web`
- `mcp`
- `bot`
- `completions`

For user-facing flags and examples, see the canonical CLI guide: `docs/guide/cli-reference.md`.

## MCP Server Mode (`ulf mcp`)

`ulf mcp serve` runs Ulf as a Model Context Protocol server over `stdio`.

Notes:

- Intended for MCP client configuration (non-interactive)
- Uses stdout for protocol messages and stderr for logs
- Exposes control-plane tools, including stream polling tools like `stream_next`

## Runtime Directories

Ulf runtime artifacts are stored in `.ulf/` (for example `.ulf/agent`, `.ulf/tasks`, `.ulf/specs`), not `.agent/`.

## Command Dispatch

Dispatch is handled in `run()` via a `match` on `cli.command`, delegating to each submodule (for example `web::execute(args).await`, `mcp::execute(args).await`, `bot::execute(...)`).

## Global Options

Global CLI options include:

- `--config <PATH>`
- `--verbose`
- `--color <auto|always|never>`

## Shell Completions

`ulf completions <shell>` outputs completion scripts.

Example:

```bash
ulf completions bash > ~/.local/share/bash-completion/completions/ulf
```

## Exit Codes

Command handlers return process errors via `anyhow::Result`, surfaced by the binary entry point.
