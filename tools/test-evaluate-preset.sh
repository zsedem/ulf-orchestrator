#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

output_log="$tmp_dir/output.log"
session_file="$tmp_dir/session.jsonl"
fake_ulf="$tmp_dir/fake-ulf"
fake_ulf_on_path="$tmp_dir/ulf"

cat > "$output_log" <<'EOF'
===============================================================================
 ITERATION 1 | ? planner | 0s elapsed | 1/10
===============================================================================
===============================================================================
 ITERATION 2 | ? reviewer | 5s elapsed | 2/10
===============================================================================
EOF

: > "$session_file"

cat > "$fake_ulf" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "--version" ]]; then
  echo "ulf fake 1.0.0"
else
  printf 'fake-ulf %s\n' "$*"
fi
EOF
chmod +x "$fake_ulf"
ln -s "$fake_ulf" "$fake_ulf_on_path"

EVALUATE_PRESET_LIB_ONLY=1 source "$SCRIPT_DIR/evaluate-preset.sh"

assert_eq() {
    local actual=$1
    local expected=$2
    local message=$3
    if [[ "$actual" != "$expected" ]]; then
        echo "Assertion failed: $message" >&2
        echo "  expected: $expected" >&2
        echo "  actual:   $actual" >&2
        exit 1
    fi
}

iterations="$(count_iterations_from_output "$output_log")"
hats="$(extract_hats_from_output "$output_log")"
recording_empty="$(session_recording_empty "$session_file")"
termination="$(termination_source "$session_file" 0)"
ULF_EVAL_BINARY="$fake_ulf"
resolve_ulf_command
resolved_version="$(run_ulf --version)"
resolved_run="$(run_ulf run --dry-run)"
resolved_timed_run="$(run_ulf_with_timeout 5 run --dry-run)"
resolved_path_ulf="$(command -v ulf)"

assert_eq "$iterations" "2" "ASCII iteration markers should be counted"
assert_eq "$hats" "planner,reviewer" "ASCII iteration markers should yield hat ids"
assert_eq "$recording_empty" "true" "zero-byte session file should be reported as empty"
assert_eq "$termination" "process_exit" "zero-byte session with zero exit should report process_exit termination"
assert_eq "$resolved_version" "ulf fake 1.0.0" "ULF_EVAL_BINARY should override version command"
assert_eq "$resolved_run" "fake-ulf run --dry-run" "ULF_EVAL_BINARY should override run command"
assert_eq "$resolved_timed_run" "fake-ulf run --dry-run" "timeout wrapper should invoke resolved Ulf binary directly"
assert_eq "$resolved_path_ulf" "$fake_ulf_on_path" "PATH should prefer the selected Ulf binary for in-agent tool calls"

echo "evaluate-preset parsing tests passed"
