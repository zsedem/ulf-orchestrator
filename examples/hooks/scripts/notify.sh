#!/usr/bin/env sh
set -eu

# Minimal completion notifier example.
phase_event="${ULF_HOOK_PHASE_EVENT:-unknown-phase-event}"
loop_id="${ULF_LOOP_ID:-unknown-loop-id}"
iteration="${ULF_ITERATION:-unknown-iteration}"

printf 'notify: %s for %s (iteration %s)\n' "$phase_event" "$loop_id" "$iteration"
