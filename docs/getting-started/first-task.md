# Your First Task

Let's walk through creating and running a complete task with Ulf.

## Choose Your Mode

Ulf offers three modes. Choose based on your task complexity:

| Mode | When to Use |
|------|-------------|
| **Multi-Workspace** | Feature work, vibe-coding, isolated environments *(Recommended)* |
| **Traditional** | Simple tasks, quick automation, existing projects |
| **Hat-Based** | Complex workflows, multi-step processes, role separation |

For this guide, we'll start with multi-workspace mode (recommended), then show traditional and hat-based modes.

## Multi-Workspace Example (Recommended)

### 1. Start the Daemon

```bash
ulf daemon start
```

The daemon auto-starts on most `ulf workspace` commands, but starting it explicitly ensures it's ready.

### 2. Create a Workspace

```bash
# Create a workspace for your task
ulf workspace create calc-task --name "Build a calculator"

# Or create with a setup prompt
ulf workspace create calc-task \
  --name "Build a calculator" \
  --setup-prompt "Create a Rust project with cargo init and a CLAUDE.md"

# Wait for setup to finish
ulf workspace create calc-task --wait
```

### 3. Attach and Work

```bash
ulf workspace attach calc-task
```

Inside the middle-manager session, tell Ulf what to build:

```
Build a Rust calculator module with add, subtract, multiply, divide.
Handle division by zero. Include unit tests. Run cargo test to verify.
```

### 4. Manage and Iterate

```bash
# Check workspace status
ulf workspace status

# List all workspaces
ulf workspace list

# When done, delete the workspace
ulf workspace delete calc-task --remove-files
```

## Traditional Mode Example

For simple tasks directly in an existing project directory:

### 1. Initialize

```bash
mkdir my-first-ulf-task
cd my-first-ulf-task
git init  # Ulf works best with git

ulf init --backend claude
```

### 2. Create Your Prompt

Create `PROMPT.md`:

```markdown
# Task: Build a Simple Calculator (Rust)

Create a Rust calculator module with:

## Requirements
- Functions: add, subtract, multiply, divide
- Handle division by zero gracefully
- Include unit tests

## Acceptance Criteria
- All functions work correctly
- Tests pass with `cargo test`
- Code is formatted with `cargo fmt`
```

### 3. Run Ulf

```bash
ulf run
```

Ulf will:

1. Read your prompt
2. Start the AI agent
3. Iterate until `LOOP_COMPLETE` is output
4. Show progress in the TUI

### 4. Review Results

When Ulf completes, check your directory:

```bash
ls -la
# src/lib.rs
# tests/calculator.rs
# etc.

# Run the tests
cargo test
```

## Hat-Based Mode Example

For more complex tasks, use hats to separate concerns.

### 1. Initialize Core Config

```bash
ulf init --backend claude
```

Then run with a specialized hat collection (recommended: code-assist):

```bash
ulf run -c ulf.yml -H builtin:code-assist
```

This uses specialized hats:

- **Tester** - Writes failing tests first
- **Implementer** - Makes tests pass
- **Refactorer** - Cleans up the code

### 2. Create Your Prompt

```markdown
# Task: Build a URL Shortener

Create a URL shortening service with:

## Requirements
- Generate short codes for URLs
- Retrieve original URLs from short codes
- Handle invalid inputs gracefully
- Persist mappings to SQLite

## Constraints
- Short codes: 6 alphanumeric characters
- No duplicate short codes
```

### 3. Run with Hat Coordination

```bash
ulf run
```

The TUI shows which hat is active:

```
[iter 3] 00:02:15 Tester
```

### 4. View Event History

```bash
ulf events
```

Shows the event flow between hats:

```
task.start -> Tester
test.written -> Implementer
test.passed -> Refactorer
refactor.done -> Tester
...
```

## Tips for Good Prompts

### Be Specific

```markdown
# Bad
Make a web app.

# Good
Create an Axum web app with:
- GET /health endpoint returning {"status": "ok"}
- POST /users accepting JSON {name, email}
- SQLite database for persistence
```

### Include Acceptance Criteria

```markdown
## Acceptance Criteria
- [ ] All endpoints respond correctly
- [ ] Invalid JSON returns 400 error
- [ ] Database persists across restarts
```

### Specify Constraints

```markdown
## Constraints
- Use Axum (not Actix)
- Rust 1.75+
- No external API calls
```

## Monitoring and Control

### View Progress

The TUI shows real-time progress. Key information:

- **Iteration count** - How many cycles Ulf has run
- **Elapsed time** - Total runtime
- **Active hat** - Which persona is working (hat-based mode)
- **Agent output** - What the AI is doing

### Stop Early

Press `q` in the TUI to quit gracefully.

### Resume Interrupted Sessions

```bash
ulf run --continue
```

### Check Metrics

After completion, check `.agent/` for:

- `scratchpad.md` - Iteration state (per-hat scratchpads may also exist)
- `memories.md` - Persistent learning
- `tasks.jsonl` - Task tracking

## Common Issues

### Task Not Completing

If Ulf runs forever:

1. Check your prompt has clear completion criteria
2. Ensure `LOOP_COMPLETE` can be reasonably output
3. Set a lower `--max-iterations` for testing

### Wrong Backend

```bash
# Explicitly specify backend
ulf run --backend kiro
```

### Agent Errors

Check the agent is installed and authenticated:

```bash
# Test Claude directly
claude -p "Hello"

# Test Kiro
kiro -p "Hello"
```

## Next Steps

- Read the [Multi-Workspace Guide](../guide/multi-workspace.md) for vibe-coding workflows
- Learn about [Hats & Events](../concepts/hats-and-events.md)
- Explore [Presets](../guide/presets.md) for your workflow
- Master [Writing Prompts](../guide/prompts.md)
