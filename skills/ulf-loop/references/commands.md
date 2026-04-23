# Ulf Loop Commands

## Start a Run

```bash
ulf run -c ulf.yml -H .ulf/hats/my-workflow.yml -p "Add OAuth login"
```

Use `--dry-run` for a quick preflight:

```bash
ulf run -c ulf.yml -H .ulf/hats/my-workflow.yml -p "Add OAuth login" --dry-run
```

## Inspect Loops

```bash
ulf loops list
ulf loops list --json
ulf loops logs <id>
ulf loops logs <id> -f
ulf loops history <id>
ulf loops history <id> --json
ulf loops diff <id>
ulf loops diff <id> --stat
ulf loops attach <id>
```

Use `list --json` and `history --json` when the caller wants structured output.

## Merge Queue Operations

```bash
ulf loops merge <id>
ulf loops process
ulf loops retry <id>
ulf loops discard <id> -y
ulf loops merge-button-state <id>
```

Recommended flow for queued or `needs-review` work:

1. `ulf loops diff <id> --stat`
2. `ulf loops history <id>`
3. `ulf loops merge <id>` or `ulf loops retry <id>`
4. `ulf loops discard <id> -y` if the work should be abandoned

## Stop or Resume

```bash
ulf loops stop <id>
ulf loops stop <id> --force
ulf loops resume <id>
ulf loops prune
```

Use `resume` only when the loop is actually suspended. The command is idempotent
and writes the operator signal Ulf consumes at the suspension boundary.
