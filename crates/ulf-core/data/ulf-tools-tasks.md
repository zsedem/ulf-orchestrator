---
name: ulf-tools-tasks
description: Use when managing runtime tasks during Ulf orchestration runs
metadata:
  internal: true
---

# Ulf Tools — Tasks

## Two Task Systems

| System | Command | Purpose | Storage |
|--------|---------|---------|---------|
| **Runtime tasks** | `ulf tools task` | Track work items during runs | `.ulf/agent/tasks.jsonl` |
| **Code tasks** | `ulf task` | Implementation planning | `tasks/*.code-task.md` |

This skill covers **runtime tasks**. For code tasks, see `/code-task-generator`.

## Task Commands

```bash
ulf tools task add "Title" -p 2 -d "description" --blocked-by id1,id2
ulf tools task ensure "Title" --key spec:task-01 -p 2 -d "description" --blocked-by id1,id2
ulf tools task list [--status open|in_progress|closed] [--format table|json|quiet]
ulf tools task ready                    # Show unblocked tasks
ulf tools task start <task-id>
ulf tools task close <task-id>
ulf tools task reopen <task-id>
ulf tools task fail <task-id>
ulf tools task show <task-id>
```

**Task ID format:** `task-{timestamp}-{4hex}` (e.g., `task-1737372000-a1b2`)

**Task key:** optional stable key for idempotent orchestrator-managed tasks (for example `spec:task-01`)

**Priority:** 1-5 (1 = highest, default 3)

### Task Rules
- One task = one testable unit of work (completable in 1-2 iterations)
- Break large features into smaller tasks BEFORE starting implementation
- On your first iteration, check `ulf tools task ready` — prior iterations may have created tasks
- Use `task ensure --key ...` when a task has a stable identity and may be recreated across fresh-context iterations
- Use `task start` when you begin active work on a task
- ONLY close tasks after verification (tests pass, build succeeds)
- Use `task reopen` when more work remains after a failed review/finalization pass
- Use `task fail` when the task is blocked and cannot be completed in the current iteration

### First thing every iteration
```bash
ulf tools task ready    # What's open? Pick one. Don't create duplicates.
```

### Failure Capture — Task Half

If any command fails (non-zero exit), or you hit a missing dependency/skill, or you are blocked:
- **Open or reopen a task** if it won't be resolved in the same iteration.

```bash
ulf tools task ensure "Fix: <short description>" --key fix:<short-key> -p 2
```

## Common Workflows

### Track dependent work
```bash
ulf tools task ensure "Setup auth" --key auth:setup -p 1
# Returns: task-1737372000-a1b2

ulf tools task ensure "Add user routes" --key auth:routes --blocked-by task-1737372000-a1b2
ulf tools task ready  # Only shows unblocked tasks
```
