# Ulf Loop Diagnostics

## Current Diagnostics Files

Enable diagnostics with:

```bash
ULF_DIAGNOSTICS=1 ulf run -c ulf.yml -H .ulf/hats/my-workflow.yml -p "..."
```

The session directory lives under `.ulf/diagnostics/<timestamp>/`.

Key files:

- `agent-output.jsonl` for agent text and tool calls
- `orchestration.jsonl` for hat selection, events, and backpressure
- `performance.jsonl` for timing and token metrics
- `errors.jsonl` for parse and validation failures
- `trace.jsonl` for lower-level tracing
- `prompt-log.md` for the full prompt sent to the agent each iteration

Useful commands:

```bash
SESSION=".ulf/diagnostics/$(ls -t .ulf/diagnostics | head -1)"
jq 'select(.event.type == "hat_selected")' "$SESSION/orchestration.jsonl"
jq 'select(.type == "tool_call")' "$SESSION/agent-output.jsonl"
jq '.' "$SESSION/errors.jsonl"
jq '{iteration, duration_ms}' "$SESSION/performance.jsonl"

# View the full prompt for a specific iteration
grep -A 1000 "^# Iteration 3" "$SESSION/prompt-log.md" | sed '/^---$/q'
```

## Suspend and Resume Artifacts

Hook-driven suspension uses these operator-facing files:

- `.ulf/suspend-state.json`
- `.ulf/resume-requested`

Related control-signal files that can appear during loop operation:

- `.ulf/stop-requested`
- `.ulf/restart-requested`

Normal operator flow:

1. inspect `.ulf/suspend-state.json`
2. run `ulf loops resume <id>`
3. let Ulf consume `.ulf/resume-requested`

Avoid writing these files by hand unless the CLI path is unavailable and you
have already confirmed the recovery mechanics.

## State Files Worth Inspecting

- `.ulf/loop.lock` for the primary loop pid and prompt
- `.ulf/loops.json` for tracked loop metadata
- `.ulf/merge-queue.jsonl` for queued/merging/review events

When the user wants a concise operator summary, prefer `ulf loops list --json`
over hand-parsing the files.
