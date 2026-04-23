# Architecture

## System Overview

Ulf Orchestrator is a multi-agent orchestration framework that coordinates AI coding assistants (Claude, Kiro, Gemini, Codex, Amp, Pi) through an event-driven loop. The system uses a "hat" metaphor where different agent personas (hats) handle different phases of work, coordinated via pub/sub messaging.

```mermaid
graph TB
    subgraph "User Interfaces"
        CLI["ulf-cli<br/>CLI Entry Point"]
        TUI["ulf-tui<br/>Terminal UI"]
        WEB["ulf-web<br/>React Dashboard"]
        TEL["ulf-telegram<br/>Telegram Bot"]
    end

    subgraph "API Layer"
        API["ulf-api<br/>REST/WebSocket Server"]
    end

    subgraph "Core Engine"
        CORE["ulf-core<br/>Orchestration Logic"]
        PROTO["ulf-proto<br/>Protocol Types"]
    end

    subgraph "Agent Backends"
        ADAPT["ulf-adapters<br/>Backend Integrations"]
        CLAUDE["Claude CLI"]
        KIRO["Kiro CLI"]
        GEMINI["Gemini CLI"]
        CODEX["Codex CLI"]
        AMP["Amp CLI"]
        PI["Pi CLI"]
    end

    CLI --> CORE
    CLI --> ADAPT
    TUI --> CORE
    TUI --> API
    WEB --> API
    TEL --> CORE
    API --> CORE
    CORE --> PROTO
    CORE --> ADAPT
    ADAPT --> CLAUDE
    ADAPT --> KIRO
    ADAPT --> GEMINI
    ADAPT --> CODEX
    ADAPT --> AMP
    ADAPT --> PI
```

## Crate Dependency Graph

```mermaid
graph LR
    CLI["ulf-cli"] --> CORE["ulf-core"]
    CLI --> ADAPT["ulf-adapters"]
    CLI --> TUI["ulf-tui"]
    CLI --> TELE["ulf-telegram"]
    CLI --> API["ulf-api"]

    CORE --> PROTO["ulf-proto"]
    ADAPT --> PROTO
    TUI --> PROTO
    TUI --> CORE
    TELE --> PROTO
    TELE --> CORE
    API --> CORE
    API --> ADAPT
    API --> PROTO

    E2E["ulf-e2e"] --> CORE
    E2E --> ADAPT
    E2E --> PROTO
    BENCH["ulf-bench"] --> CORE
```

## Core Architectural Patterns

### 1. Event-Driven Orchestration

The central architectural pattern is a pub/sub event bus. Each iteration of the loop:
1. An agent (wearing a "hat") executes a task
2. The agent writes events to a JSONL file (`.ulf/events.jsonl`)
3. The event loop parses events and publishes them on the `EventBus`
4. The `EventBus` routes events to subscribed hats based on topic patterns
5. The next hat with pending events is activated

```mermaid
sequenceDiagram
    participant EL as Event Loop
    participant EB as EventBus
    participant P as Planner Hat
    participant B as Builder Hat
    participant Agent as AI Backend

    EL->>EB: publish(task.start)
    EB->>P: route to Planner (subscribed)
    EL->>Agent: Execute Planner iteration
    Agent-->>EL: Writes build.task event
    EL->>EB: publish(build.task)
    EB->>B: route to Builder (subscribed)
    EL->>Agent: Execute Builder iteration
    Agent-->>EL: Writes build.done event
    EL->>EB: publish(build.done)
    EB->>P: route to Planner (subscribed)
```

### 2. Hat System (Agent Personas)

Hats define how an AI agent behaves for a given iteration. Each hat has:
- **Subscriptions**: Topic patterns it responds to (e.g., `build.task`)
- **Publishes**: Topics it emits (e.g., `build.done`, `build.blocked`)
- **Instructions**: Prompt content injected for the hat's iterations
- **Backend**: Which AI CLI to use (can differ per hat)

The default topology is Planner → Builder:
- **Planner**: Subscribes to `task.start`, `task.resume`, `build.done`, `build.blocked`; publishes `build.task`
- **Builder**: Subscribes to `build.task`; publishes `build.done`, `build.blocked`

Custom hats are defined in YAML configuration and can create arbitrary workflows.

### 3. Hatless Ulf (Coordinator)

Ulf is the constant coordinator — always present, cannot be configured away. Ulf:
- Handles `task.start` and `task.resume` (reserved events)
- Performs gap analysis and planning
- Delegates to custom hats via events
- Acts as fallback handler for unrouted events (global wildcard `*` subscriber)
- Builds the prompt including guardrails, memories, skills, and hat-specific instructions

### 4. Topic-Based Routing

Topics are dot-separated strings with glob pattern matching:
- Exact match: `build.done` matches `build.done`
- Wildcard suffix: `impl.*` matches `impl.done`, `impl.started`
- Wildcard prefix: `*.done` matches `build.done`, `review.done`
- Global wildcard: `*` matches everything (fallback priority)

Routing priority: specific subscriptions > fallback wildcards.

### 5. Parallel Loops via Git Worktrees

When the primary loop lock is held, Ulf spawns parallel loops in git worktrees:
- Primary loop holds `.ulf/loop.lock`
- Worktree loops run in `.worktrees/<loop-id>/`
- Shared state: memories, specs, and code tasks are symlinked from main repo
- Merge queue: completed worktree loops queue for merge via `.ulf/merge-queue.jsonl`

### 6. Disk-as-State, Git-as-Memory

Per the Ulf Tenets, persistent state lives on disk:
- `.ulf/agent/memories.md` — persistent learning across sessions
- `.ulf/agent/tasks.jsonl` — runtime work tracking
- `.ulf/events.jsonl` — event history for the current loop
- `.ulf/loop.lock` — PID + prompt of the primary loop
- `.ulf/loops.json` — registry of all tracked loops

### 7. Communication Protocols

The system supports multiple communication modes:

| Protocol | Transport | Use Case |
|----------|-----------|----------|
| **CLI** | PTY subprocess | Default agent execution |
| **RPC** | JSON-lines over stdin/stdout | IDE integrations, subprocess TUI |
| **HTTP/WS** | Axum REST + WebSocket | Web dashboard, remote TUI |
| **Telegram** | teloxide bot framework | Human-in-the-loop interaction |

### 8. Backpressure Over Prescription

Rather than prescribing how agents should work, Ulf creates gates that reject bad work:
- Tests, typechecks, builds, lints (via lifecycle hooks)
- LLM-as-judge with binary pass/fail for subjective criteria
- Hooks can `warn`, `block`, or `suspend` the loop on failure

### 9. Fresh Context Is Reliability

Each iteration clears agent context. The orchestrator:
- Re-reads specs, plans, and memories every cycle
- Optimizes for the "smart zone" (40-60% of ~176K usable tokens)
- Never fights to save a plan — regeneration costs one planning loop

## Configuration Architecture

Ulf uses a split configuration model:
1. **Core config** (`ulf.yml`): Backend, event loop settings, features
2. **Hat collections** (`-H builtin:feature` or YAML files): Hat definitions, event metadata
3. **Presets** (`presets/`): Pre-built hat collection YAML files

Configuration supports both v1 (flat) and v2 (nested) formats with automatic normalization.
