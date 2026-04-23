#!/usr/bin/env sh
set -eu

# Minimal blocking guard example.
# Fails fast when required hook context variables are missing.
: "${ULF_HOOK_PHASE_EVENT:?env-guard: missing ULF_HOOK_PHASE_EVENT}"
: "${ULF_LOOP_ID:?env-guard: missing ULF_LOOP_ID}"
: "${ULF_WORKSPACE:?env-guard: missing ULF_WORKSPACE}"

printf 'env-guard: context OK for %s (loop %s)\n' "$ULF_HOOK_PHASE_EVENT" "$ULF_LOOP_ID"
