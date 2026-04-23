# Debugging Example

!!! note "Documentation In Progress"
    This page is under development. Check back soon for a complete debugging workflow example.

## Overview

This example demonstrates using Ulf to debug issues, with specialized hats for investigation, hypothesis testing, and fix verification.

## Enabling Diagnostics

```bash
ULF_DIAGNOSTICS=1 ulf run -p "fix the authentication bug"
```

## Reviewing Logs

```bash
# View all agent output
jq 'select(.type == "text")' .ulf/diagnostics/*/agent-output.jsonl

# View hat selection decisions
jq 'select(.event.type == "hat_selected")' .ulf/diagnostics/*/orchestration.jsonl

# View errors
jq '.' .ulf/diagnostics/*/errors.jsonl
```

## See Also

- [Diagnostics](../advanced/diagnostics.md) - Full diagnostics reference
- [Troubleshooting](../reference/troubleshooting.md) - Common issues
- [Simple Task](simple-task.md) - Basic example
