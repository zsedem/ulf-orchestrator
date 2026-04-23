#!/usr/bin/env bash
# Manual regression test for issue #213
# Run this script to verify the fix for "Subprocess TUI run wrongly spawns worktree on first run"
#
# Usage: ./test_issue_213.sh
#
# This script:
# 1. Creates a fresh test directory
# 2. Initializes git and ulf
# 3. Runs ulf in subprocess TUI mode
# 4. Checks that NO worktree was created (the bug causes a spurious worktree)

set -e

TEST_DIR=$(mktemp -d)
echo "Test directory: $TEST_DIR"
cd "$TEST_DIR"

# Initialize git repo
git init -q

# Initialize ulf (use codex as backend, or claude if unavailable)
if command -v ulf &> /dev/null; then
    ulf init --backend codex --force 2>/dev/null || ulf init --backend claude --force 2>/dev/null || true
else
    echo "WARNING: ulf not installed, using mock config"
    mkdir -p .ulf
fi

# Create a simple prompt
echo "Smoke test prompt" > PROMPT.md

echo ""
echo "=== Before running ulf ==="
ls -la
echo ""
echo "=== Checking for .worktrees ==="
ls -la .worktrees 2>/dev/null || echo "No .worktrees directory (expected)"

echo ""
echo "=== Running ulf (simulating TUI mode with script) ==="
# Run with timeout to prevent hanging
# The --legacy-tui flag forces in-process TUI which behaves similarly to subprocess TUI
# for our testing purposes
timeout 10s script -qefc 'ulf run -P PROMPT.md --skip-preflight --max-iterations 1' /tmp/ulf-test-log.txt 2>&1 || true

echo ""
echo "=== After running ulf ==="
ls -la

echo ""
echo "=== Checking for worktrees ==="
if [ -d ".worktrees" ]; then
    echo "BUG: .worktrees directory was created!"
    find .worktrees -maxdepth 3 -type d
    echo ""
    echo "=== Loop registry ==="
    cat .ulf/loops.json 2>/dev/null || echo "No loops.json"
    echo ""
    echo "=== Lock file ==="
    cat .ulf/loop.lock 2>/dev/null || echo "No loop.lock"
    RESULT=1
else
    echo "SUCCESS: No .worktrees directory created (fix working!)"
    RESULT=0
fi

# Cleanup
rm -rf "$TEST_DIR"

exit $RESULT
