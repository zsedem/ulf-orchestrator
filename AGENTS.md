# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> The orchestrator is a thin coordination layer, not a platform. Agents are smart; let them do the work.

## Build & Test

```bash
cargo build
cargo test
cargo test -p ulf-core test_name           # Run single test
cargo test -p ulf-core smoke_runner        # Smoke tests (replay-based)
cargo run -p ulf-e2e -- --mock             # E2E tests (CI-safe)
./scripts/setup-hooks.sh                     # Install pre-commit hooks (once)
```

**IMPORTANT**: Run `cargo test` before declaring any task done. Smoke test after code changes.

### Web Dashboard

```bash
ulf web                                    # Launch both servers (backend:3000, frontend:5173)
npm install                                  # Install all dependencies
npm run dev                                  # Dev mode (both)
npm run dev:server                           # Backend only
npm run dev:web                              # Frontend only
npm run test:server                          # Backend tests
```

## Architecture

```
ulf-cli      → CLI entry point, commands (run, plan, task, loops, web)
ulf-core     → Orchestration logic, event loop, hats, memories, tasks
ulf-adapters → Backend integrations (Claude, Kiro, Gemini, Codex, Roo, etc.)
ulf-telegram → Telegram bot for human-in-the-loop communication
ulf-tui      → Terminal UI (ratatui-based)
ulf-e2e      → End-to-end test framework
ulf-proto    → Protocol definitions
ulf-bench    → Benchmarking

backend/       → Web server (@ulf-web/server) - Fastify + tRPC + SQLite
frontend/      → Web dashboard (@ulf-web/dashboard) - React + Vite + TailwindCSS
```

### Key Files

| File | Purpose |
|------|---------|
| `.ulf/agent/memories.md` | Persistent learning across sessions |
| `.ulf/agent/tasks.jsonl` | Runtime work tracking |
| `.ulf/loop.lock` | Contains PID + prompt of primary loop |
| `.ulf/loops.json` | Registry of all tracked loops |
| `.ulf/merge-queue.jsonl` | Event-sourced merge queue |
| `.ulf/telegram-state.json` | Telegram bot state (chat ID, pending questions) |

### Code Locations

- **Event loop**: `crates/ulf-core/src/event_loop/mod.rs`
- **Hat system**: `crates/ulf-core/src/hatless_ulf.rs`
- **Memory system**: `crates/ulf-core/src/memory.rs`, `memory_store.rs`
- **Task system**: `crates/ulf-core/src/task.rs`, `task_store.rs`
- **Lock coordination**: `crates/ulf-core/src/worktree.rs`
- **Loop registry**: `crates/ulf-core/src/loop_registry.rs`
- **Merge queue**: `crates/ulf-core/src/merge_queue.rs`
- **Completion gates**: `crates/ulf-core/src/completion_gates.rs`
- **Checkpoint gates**: `crates/ulf-core/src/completion_gates.rs` (`CheckpointGateRunner`)
- **CLI commands**: `crates/ulf-cli/src/loops.rs`, `task_cli.rs`
- **Telegram integration**: `crates/ulf-telegram/src/` (bot, service, state, handler)
- **RObot config**: `crates/ulf-core/src/config.rs` (`RobotConfig`, `TelegramBotConfig`)
- **Wave system**: `crates/ulf-core/src/wave_tracker.rs`, `wave_detection.rs`, `wave_prompt.rs`
- **Wave CLI**: `crates/ulf-cli/src/wave.rs`
- **Web server**: `backend/ulf-web-server/src/` (tRPC routes in `api/`, runners in `runner/`)
- **Web dashboard**: `frontend/ulf-web/src/` (React components in `components/`)

## The Ulf Tenets

1. **Fresh Context Is Reliability** — Each iteration clears context. Re-read specs, plan, code every cycle. Optimize for the "smart zone" (40-60% of ~176K usable tokens).

2. **Backpressure Over Prescription** — Don't prescribe how; create gates that reject bad work. Tests, typechecks, builds, lints. For subjective criteria, use LLM-as-judge with binary pass/fail.

3. **The Plan Is Disposable** — Regeneration costs one planning loop. Cheap. Never fight to save a plan.

4. **Disk Is State, Git Is Memory** — Memories and Tasks are the handoff mechanisms. No sophisticated coordination needed.

5. **Steer With Signals, Not Scripts** — The codebase is the instruction manual. When Ulf fails a specific way, add a sign for next time.

6. **Let Ulf Ulf** — Sit *on* the loop, not *in* it. Tune like a guitar, don't conduct like an orchestra.

## Anti-Patterns

- ❌ Building features into the orchestrator that agents can handle
- ❌ Complex retry logic (fresh context handles recovery)
- ❌ Detailed step-by-step instructions (use backpressure instead)
- ❌ Scoping work at task selection time (scope at plan creation instead)

### Completion Gates & Checkpoint Gates

**Completion gates** run when the agent emits `LOOP_COMPLETE`:

```yaml
event_loop:
  completion_gates:
    - name: tests-pass
      command: ["cargo", "test"]
```

**Checkpoint gates** run at iteration boundaries (mid-session):

```yaml
event_loop:
  checkpoint_gates:
    - name: lint-check
      trigger: every_n_iterations
      every_n: 5
      command: ["cargo", "clippy"]
    - name: tests-after-build
      trigger: after_event
      after_event: dev.done
      command: ["cargo", "test"]
```

Both gate types inject `task.resume` backpressure on failure. See `docs/concepts/backpressure.md` for details.
- ❌ Assuming functionality is missing without code verification

## Specs & Tasks

- Create specs in `.ulf/specs/` — do NOT implement without an approved spec first
- Create code tasks in `.ulf/tasks/` using `.code-task.md` extension
- Work step-by-step: spec → dogfood spec → implement → dogfood implementation → done

### Memories and Tasks (Default Mode)

Memories and tasks are enabled by default. Both must be enabled/disabled together:

When enabled (default):
- Scratchpad is disabled
- Tasks replace scratchpad for completion verification
- Loop terminates when no open tasks + consecutive LOOP_COMPLETE

To disable (legacy scratchpad mode):
```yaml
memories:
  enabled: false
tasks:
  enabled: false
```

## Parallel Loops

Ulf supports multiple orchestration loops in parallel using git worktrees.

```
Primary Loop (holds .ulf/loop.lock)
├── Runs in main workspace
├── Processes merge queue on completion
└── Spawns merge-ulf for queued loops

Worktree Loops (.worktrees/<loop-id>/)
├── Isolated filesystem via git worktree
├── Symlinked memories, specs, tasks → main repo
├── Queue for merge on completion
└── Exit cleanly (no spawn)
```

### Testing Parallel Loops

```bash
cd $(mktemp -d) && git init && echo "<p>Hello</p>" > index.html && git add . && git commit -m "init"

# Terminal 1: Primary loop
ulf run -p "Add header before <p>" --max-iterations 5

# Terminal 2: Worktree loop
ulf run -p "Add footer after </p>" --max-iterations 5

# Monitor
ulf loops
```

## Agent Waves (Intra-Loop Parallelism)

Waves enable a single hat to process multiple work items in parallel within one iteration.

### Hat Config Fields

```yaml
hats:
  reviewer:
    name: "Reviewer"
    triggers: ["review.file"]
    publishes: ["review.done"]
    concurrency: 4              # Max parallel workers (default: 1)
    instructions: "..."

  synthesizer:
    triggers: ["review.done"]
    publishes: ["review.complete"]
    aggregate:                   # Buffer results until all arrive
      mode: wait_for_all
      timeout: 300               # Seconds to wait
```

- `concurrency > 1` enables wave execution for a hat
- `aggregate` makes a hat wait for all wave results before activating
- A hat cannot have both `concurrency > 1` and `aggregate`

### Wave Dispatch

Agents dispatch waves via CLI:
```bash
ulf wave emit review.file --payloads "src/main.rs" "src/lib.rs" "src/config.rs"
```

### How It Works

1. Agent emits wave events (tagged with shared `wave_id`)
2. Loop runner detects wave events, resolves target hat
3. Spawns N parallel backend instances (up to `concurrency` limit)
4. Each worker gets: focused prompt, per-worker events file, wave env vars
5. Results merged back to main events file
6. Aggregator hat picks up results on next iteration

### Key Code Locations

- **Wave CLI**: `crates/ulf-cli/src/wave.rs`
- **Wave detection**: `crates/ulf-core/src/wave_detection.rs`
- **Worker prompt**: `crates/ulf-core/src/wave_prompt.rs`
- **Wave tracker**: `crates/ulf-core/src/wave_tracker.rs`
- **Loop integration**: `crates/ulf-cli/src/loop_runner.rs` (`execute_wave`)

### Presets

- `presets/wave-review.yml` — Scatter-gather code review

## Smoke Tests (Replay-Based)

Smoke tests use recorded JSONL fixtures instead of live API calls:

```bash
cargo test -p ulf-core smoke_runner        # All smoke tests
cargo test -p ulf-core kiro                # Kiro-specific
```

**Fixtures location:** `crates/ulf-core/tests/fixtures/`

### Recording New Fixtures

```bash
cargo run --bin ulf -- run -c ulf.claude.yml --record-session session.jsonl -p "your prompt"
```

## E2E Testing

```bash
cargo run -p ulf-e2e -- claude             # Live API tests
cargo run -p ulf-e2e -- --mock             # CI-safe mock mode
cargo run -p ulf-e2e -- --mock --filter connect  # Filter scenarios
cargo run -p ulf-e2e -- --list             # List scenarios
```

Reports generated in `.e2e-tests/`.

## RObot (Human-in-the-Loop)

Ulf supports human interaction during orchestration via Telegram. Agents can ask questions and humans can send proactive guidance.

### Configuration

```yaml
# ulf.yml
RObot:
  enabled: true
  timeout_seconds: 300    # How long to block waiting for a response
  telegram:
    bot_token: "your-token"  # Or set ULF_TELEGRAM_BOT_TOKEN env var
```

### Event Types

| Event / Command | Direction | Purpose |
|-------|-----------|---------|
| `human.interact` | Agent to Human | Agent asks a question; loop blocks until response or timeout |
| `human.response` | Human to Agent | Reply to a `human.interact` question |
| `human.guidance` | Human to Agent | Proactive guidance injected as `## ROBOT GUIDANCE` in prompt |
| `ulf tools interact progress` | Agent to Human | Non-blocking progress notification via Telegram (no event, direct send) |

### How It Works

- The Telegram bot starts only on the **primary loop** (the one holding `.ulf/loop.lock`)
- When an agent emits `human.interact`, the event loop sends the question via Telegram and **blocks**
- Responses are published as `human.response` events on the bus
- Proactive messages become `human.guidance` events, squashed into a numbered list in the prompt
- Send failures retry with exponential backoff (3 attempts); if all fail, treated as timeout
- Parallel loops route messages via reply-to, `@loop-id` prefix, or default to primary

See `crates/ulf-telegram/README.md` for setup instructions.

## Diagnostics

TUI mode always logs to `.ulf/diagnostics/logs/ulf-{timestamp}.log` (last 5 kept automatically).

```bash
ULF_DIAGNOSTICS=1 ulf run -p "your prompt"
```

Output in `.ulf/diagnostics/<timestamp>/`:
- `agent-output.jsonl` — Agent text, tool calls, results
- `orchestration.jsonl` — Hat selection, events, backpressure
- `errors.jsonl` — Parse errors, validation failures

```bash
jq 'select(.type == "tool_call")' .ulf/diagnostics/*/agent-output.jsonl
ulf clean --diagnostics
```

## IMPORTANT

- Run `cargo test` before declaring any task done
- Backwards compatibility doesn't matter — it adds clutter for no reason
- Prefer replay-based smoke tests over live API calls for CI
- BDD/Cucumber tests MUST exercise real runtime code paths via integration tests (not placeholder/source-only assertions)
- Run python tests using a .venv
- You MUST not commit ephemeral files
- When I ask you to view something that means to use playwright/chrome tools to go view it.
- When adding or changing `ulf tools` subcommands, update the appropriate file in `crates/ulf-core/data/`: `ulf-tools.md` (shared commands), `ulf-tools-tasks.md` (task commands), or `ulf-tools-memories.md` (memory commands). `.claude/skills/ulf-tools/SKILL.md` is a symlink to the base `ulf-tools.md`
- Design docs and specs go in `.ulf/specs` and one-off code tasks and bug fixes go in `.ulf/tasks`
