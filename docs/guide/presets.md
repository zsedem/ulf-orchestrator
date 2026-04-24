# Hat Collections

Built-in hat collections are now intentionally small. Ulf ships a core working set of defaults and documents broader workflow ideas as examples instead of treating every pattern as a supported builtin.

## Quick Start

```bash
ulf init --backend claude
ulf init --list-presets

ulf run -c ulf.yml -H builtin:code-assist -p "Add user authentication"
```

## Supported Builtins

| Collection | Hats | Best for | Notes |
|---|---|---|---|
| `code-assist` | `planner`, `builder`, `critic`, `finalizer` | Default implementation work | Recommended default; adds fresh-eyes review and a final completion gate |
| `debug` | `investigator`, `tester`, `fixer`, `verifier` | Root-cause debugging | Strong on repro and fix verification |
| `research` | `researcher`, `synthesizer` | Read-only analysis | No code changes |
| `review` | `reviewer`, `analyzer` | Adversarial code review | No code changes |
| `pdd-to-code-assist` | multi-stage design + build pipeline | Idea to code | Advanced and fun, but slower and less predictable |

## Internal Presets

Ulf also keeps a few internal/testing presets available without advertising them in the normal list:

- `merge-loop`
- `hatless-baseline`

## Recommended Workflow

- Use `code-assist` for most implementation tasks.
- Use `debug`, `research`, or `review` when you need a specialized mode.
- Use `pdd-to-code-assist` when you specifically want an end-to-end exploratory workflow and are comfortable paying for extra iterations.

| Collection | Canonical source | Hats | Start event | Completion | Best for |
|---|---|---|---|---|---|
| `bugfix` | `presets/bugfix.yml` | `reproducer`, `fixer`, `verifier`, `committer` | `repro.start` | `LOOP_COMPLETE` (default) | Reproduce/fix/verify/commit bug workflow |
| `code-assist` | `presets/code-assist.yml` | `planner`, `builder`, `validator`, `committer` | `build.start` | `LOOP_COMPLETE` | TDD implementation from specs/tasks/descriptions |
| `debug` | `presets/debug.yml` | `investigator`, `tester`, `fixer`, `verifier` | `debug.start` | `DEBUG_COMPLETE` | Root-cause debugging and hypothesis testing |
| `deploy` | `presets/deploy.yml` | `builder`, `deployer`, `verifier` | `task.start` (default) | `LOOP_COMPLETE` | Deployment and release workflows |
| `docs` | `presets/docs.yml` | `writer`, `reviewer` | `task.start` (default) | `DOCS_COMPLETE` | Documentation writing and review |
| `feature` | `presets/feature.yml` | `builder`, `reviewer` | `task.start` (default) | `LOOP_COMPLETE` | Feature development with integrated review |
| `fresh-eyes` | `presets/fresh-eyes.yml` | `builder`, `fresh_eyes_auditor`, `fresh_eyes_gatekeeper` | `fresh_eyes.start` | `LOOP_COMPLETE` | Enforced repeated skeptical self-review passes |
| `gap-analysis` | `presets/gap-analysis.yml` | `analyzer`, `verifier`, `reporter` | `gap.start` | `GAP_ANALYSIS_COMPLETE` | Spec-vs-implementation auditing |
| `hatless-baseline` | `presets/hatless-baseline.yml` | _(none)_ | `task.start` | `LOOP_COMPLETE` | Baseline no-hat behavior for comparison |
| `merge-loop` | `crates/ulf-cli/presets/merge-loop.yml` | `merger`, `resolver`, `tester`, `cleaner`, `failure_handler` | `merge.start` | `MERGE_COMPLETE` | Internal merge/worktree automation |
| `pdd-to-code-assist` | `presets/pdd-to-code-assist.yml` | `inquisitor`, `architect`, `design_critic`, `explorer`, `planner`, `task_writer`, `builder`, `validator`, `committer` | `design.start` | `LOOP_COMPLETE` | Full idea → plan → implementation pipeline |
| `pr-review` | `presets/pr-review.yml` | `correctness_reviewer`, `security_reviewer`, `architecture_reviewer`, `synthesizer` | `task.start` (default) | `LOOP_COMPLETE` | Multi-perspective PR review |
| `refactor` | `presets/refactor.yml` | `refactorer`, `verifier` | `task.start` (default) | `REFACTOR_COMPLETE` | Incremental, verified refactoring |
| `research` | `presets/research.yml` | `researcher`, `synthesizer` | `research.start` | `RESEARCH_COMPLETE` | Exploration and analysis without code changes |
| `review` | `presets/review.yml` | `reviewer`, `analyzer` | `review.start` | `REVIEW_COMPLETE` | Review-only workflow |
| `spec-driven` | `presets/spec-driven.yml` | `spec_writer`, `spec_reviewer`, `implementer`, `verifier` | `spec.start` | `LOOP_COMPLETE` (default) | Specification-driven implementation |
| `wave-review` | `presets/wave-review.yml` | `coordinator`, `reviewer` (x3), `synthesizer` | `review.start` | `LOOP_COMPLETE` | Specialized parallel code review (wave-enabled) |

## Why The Builtin Set Is Small

Every builtin preset becomes product surface area:

- It must be documented.
- It must be tested and kept working.
- It must appear coherent in API and CLI listings.

Ulf now prefers a small supported set plus documentation examples for more experimental or niche orchestration patterns.

## Examples Instead Of Builtins

Historical workflow ideas such as spec-driven development, red-team review, mob programming, and fresh-eyes loops are now examples rather than shipped builtins. See:

- [Examples Index](../examples/index.md)
- [Spec-Driven Development Example](../examples/spec-driven.md)
- [Multi-Hat Workflow](../examples/multi-hat.md)

## Usage Examples

```bash
# Default implementation workflow
ulf run -c ulf.yml -H builtin:code-assist -p "Add OAuth login"

# Debugging
ulf run -c ulf.yml -H builtin:debug -p "Investigate why login fails on mobile"

# Research
ulf run -c ulf.yml -H builtin:research -p "Map the authentication architecture"

# Review
ulf run -c ulf.yml -H builtin:review -p "Review the changes in src/api/"

# Advanced/fun workflow
ulf run -c ulf.yml -H builtin:pdd-to-code-assist -p "Build a rate limiter"
```

## Common Workflow Patterns

Ulf built-ins usually follow one of these shapes:

### 1) Linear Pipeline
A fixed sequence of specialist hats.

Examples: `feature`, `bugfix`, `deploy`, `docs`

### 2) Critic / Actor Loop
One hat proposes, another critiques/validates, then iterates.

Examples: `spec-driven`, `review`, `fresh-eyes`

### 3) Multi-Reviewer + Synthesis
Parallel perspectives merged into one result.

Example: `pr-review`

### 4) Scatter-Gather (Waves)
One hat dispatches, parallel workers execute, an aggregator synthesizes.

Example: `wave-review`

See [Agent Waves](../advanced/agent-waves.md) for details.

### 5) Extended End-to-End Orchestration
Large multi-stage pipelines from idea through implementation.

Example: `pdd-to-code-assist`

## Split Config vs Single-File Config

Recommended:
- Keep core/runtime config in `ulf.yml`
- Select workflow via `-H builtin:<name>`

Backward-compatible single-file mode (still supported):

```bash
# Uses one combined preset file as the main config
ulf run -c presets/feature.yml -p "Add OAuth login"
```

## Creating Your Own Hat Collection

Create a hats file with hats-related sections:

```yaml
event_loop:
  starting_event: "build.start"
  completion_promise: "LOOP_COMPLETE"

hats:
  builder:
    name: "Builder"
    triggers: ["build.start"]
    publishes: ["build.done"]
    instructions: |
      Implement the requested change and verify it.

  reviewer:
    name: "Reviewer"
    triggers: ["build.done"]
    publishes: ["LOOP_COMPLETE"]
    instructions: |
      Review the change, request fixes if needed, and close when done.
```

Run it:

```bash
ulf run -c ulf.yml -H .ulf/hats/my-workflow.yml
```

## Example Configurations by Use Case

### Minimal Workspace Setup

```yaml
# ~/.ulf/config.yml — For vibe-coding with workspaces
cli:
  backend: "claude"

workspace:
  default_setup_prompt: |
    Create a CLAUDE.md with build/test commands and code style.

  middle_manager:
    prompt_extensions:
      - "Always run tests before suggesting done."
```

### Team Shared Config

```yaml
# ~/.ulf/config.yml — Shared team defaults
cli:
  backend: "claude"

event_loop:
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 50

core:
  guardrails:
    - "Follow existing code patterns"
    - "Never modify production database"
    - "Always add tests for new functionality"

workspace:
  backend_presets:
    claude-team:
      backend: "claude"
      args: ["--allowedTools", "Bash", "Edit", "Read"]

  middle_manager:
    backend_preset: "claude-team"
    prompt_extensions:
      - "Use the team's coding conventions from CLAUDE.md"
      - "Prefer small, focused commits"
```

### Per-Project with Hats

```yaml
# ./ulf.yml — Project-specific core config
cli:
  backend: "claude"

event_loop:
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 100

features:
  preflight:
    enabled: true
    strict: true
```

```bash
# Run with built-in hat collection
ulf run -c ulf.yml -H builtin:code-assist -p "Add user authentication"
```

### Full-Stack Development Setup

```yaml
# ~/.ulf/config.yml
cli:
  backend: "claude"

workspace:
  backend_presets:
    frontend:
      backend: "claude"
      args: ["--verbose"]
    backend-api:
      backend: "kiro"
      idle_timeout_secs: 300

  middle_manager:
    backend_preset: "frontend"
    prompt_extensions:
      - "Run the full test suite before declaring done."
      - "Follow the project's existing architecture patterns."

hooks:
  enabled: true
  events:
    post.loop.complete:
      - name: notify-done
        command: ["echo", "Task complete!"]
        on_error: warn
```

## Source of Truth and Sync

- Canonical preset files: `presets/*.yml`
- Embedded CLI mirror: `crates/ulf-cli/presets/*.yml`
- Sync script: `./scripts/sync-embedded-files.sh`
