# Ulf Hats Commands

## Create or Edit a Hats File

User-authored hat collections belong under `.ulf/hats/`:

```bash
mkdir -p .ulf/hats
$EDITOR .ulf/hats/my-workflow.yml
```

## Validate the Collection

```bash
ulf hats validate -c ulf.yml -H .ulf/hats/my-workflow.yml
```

This catches:

- reserved triggers
- ambiguous routing
- missing starting-event subscribers
- orphan published events

## Inspect the Topology

```bash
ulf hats graph -c ulf.yml -H .ulf/hats/my-workflow.yml --format ascii
ulf hats graph -c ulf.yml -H .ulf/hats/my-workflow.yml --format mermaid
ulf hats show -c ulf.yml -H .ulf/hats/my-workflow.yml planner
```

Use `graph` when the user wants a workflow explanation. Use `show` when one hat
needs closer inspection.

## Exercise the Workflow

```bash
ulf run -c ulf.yml -H .ulf/hats/my-workflow.yml -p "Add OAuth login"
```

For a quick inspection before running:

```bash
ulf run -c ulf.yml -H .ulf/hats/my-workflow.yml -p "Add OAuth login" --dry-run
```

## Improvement Loop

When refactoring an existing hats file:

1. read the current YAML
2. explain the current topology
3. propose the smallest structural improvement that fixes the problem
4. re-run `ulf hats validate`
5. if useful, re-render with `ulf hats graph`
