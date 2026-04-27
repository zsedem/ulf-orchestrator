# Quick Start

Run your first Ulf orchestration in about 10 minutes.

## Install Ulf

If you haven't installed Ulf yet, follow the full [Installation](installation.md) guide.

Quick install (npm):

```bash
npm install -g @ulf-orchestrator/ulf-cli
```

## Install a Backend CLI (Claude Recommended)

Ulf needs at least one AI CLI tool available on your PATH.

```bash
# Claude Code
npm install -g @anthropic-ai/claude-code

# Verify the CLI is available
claude --version
```

If the backend requires authentication, complete its login flow per the provider's instructions.

## Verify Setup with `ulf doctor`

Run the doctor command to validate your environment:

```bash
ulf doctor
```

Fix any **WARN** or **FAIL** items before continuing. If you see auth warnings, verify your backend CLI is logged in.

---

## Path A: Multi-Workspace Vibe-Coding (New — Recommended)

The multi-workspace model lets you create isolated environments per task, run a central daemon, and attach a middle-manager to each workspace.

### 1. Start the Daemon

```bash
ulf daemon start
```

The daemon auto-starts on most `ulf workspace` commands, but starting it explicitly ensures it's ready.

### 2. Create a Workspace

```bash
# Create a workspace for your task
ulf workspace create jira-007 --name "Fix login bug"

# Or create from a git repo
ulf workspace create feature-auth --from https://github.com/you/template.git

# Wait for setup to finish (optional)
ulf workspace create jira-007 --wait
```

Workspaces are created under `~/.ulf/workspaces/` by default. The daemon tracks them in `~/.ulf/daemon/workspaces.json`.

### 3. Attach the Middle-Manager

```bash
ulf workspace attach jira-007
```

This spawns an interactive Ulf session inside the workspace. The middle-manager:

- Explores the workspace to understand the codebase
- Suggests a concrete plan with workflow commands
- Runs background workflows that return immediately (loop ID reported)
- Exiting the session does **not** stop background loops

### 4. Manage Workspaces

```bash
# List all workspaces
ulf workspace list

# Show one workspace
ulf workspace get jira-007

# Check status across all workspaces
ulf workspace status

# Delete a workspace
ulf workspace delete jira-007

# Delete and remove files
ulf workspace delete jira-007 --remove-files
```

### 5. Configure Setup Prompts

Add a default setup prompt to `~/.ulf/config.yml` so every new workspace gets initialized automatically:

```yaml
workspace:
  default_setup_prompt: |
    Create a CLAUDE.md for this workspace with:
    - Project overview
    - Build/test commands
    - Code style guidelines
```

When `workspace.create` runs, the daemon executes this prompt autonomously and transitions the workspace to `Ready`.

---

## Path B: Traditional Project-Local Mode

For simple tasks in an existing project directory:

### 1. Initialize a Project

```bash
mkdir my-ulf-project
cd my-ulf-project
git init  # Ulf works best with git

# Create a default config
ulf init --backend claude
```

This creates `ulf.yml` in your project.

### 2. Create a Minimal Hat Collection

Ulf can run with hats (role-based personas) for more structured workflows. Create a minimal hat collection file:

```yaml
# hats.yml
event_loop:
  starting_event: "task.start"

hats:
  builder:
    name: "Builder"
    triggers: ["task.start"]
    publishes: ["task.done"]
    instructions: |
      Implement the task from PROMPT.md.
      Run any relevant tests.
      When finished, emit task.done and print LOOP_COMPLETE.
```

### 3. Define Your Task

Create a `PROMPT.md` file with your task:

```markdown
# Task: Create a Todo List CLI (Rust)

Build a Rust command-line todo list with:
- Add tasks
- List tasks
- Mark tasks complete
- Save to a JSON file

Include error handling and unit tests.
```

### 4. Run Ulf

```bash
# Traditional mode (uses ulf.yml)
ulf run

# Hat-based mode (uses hats.yml)
ulf run --config hats.yml

# Inline prompt example
ulf run -p "Add input validation to the user API endpoints"
```

## Understand the Output

While running, Ulf shows a TUI with:

- Current iteration number
- Elapsed time
- Active hat (if hat-based)
- Recent agent output

Ulf stops when one of these occurs:

- `LOOP_COMPLETE` is output (success)
- Maximum iterations reached (default: 100)
- Maximum runtime exceeded (default: 4 hours)
- You quit the TUI

When it finishes, review the generated files in your project directory and `.ulf/` run logs.

## Command-Line Options

```bash
# Limit iterations
ulf run --max-iterations 50

# Use different config file
ulf run -c custom-ulf.yml

# Resume interrupted session
ulf run --continue

# Quiet mode for CI
ulf run -q
```

## Example Tasks

### Simple Function

```markdown
Write a TypeScript function that validates email addresses.
Include unit tests.
```

### Web Scraper

```markdown
Create a web scraper that:
1. Fetches the Hacker News homepage
2. Extracts the top 10 stories
3. Saves them to JSON

Use Node.js with a simple HTML parser.
```

### CLI Tool

```markdown
Build a markdown to HTML converter:
- Accept input/output file arguments
- Support basic markdown syntax
- Add --watch mode
```

## Next Steps

- Read [Your First Task](first-task.md) for a detailed walkthrough
- Understand [Concepts](../concepts/index.md) like hats and events
- Explore [Presets](../guide/presets.md) for common workflows
- Learn about [Configuration](../guide/configuration.md) options
- Read the [Multi-Workspace Guide](../guide/multi-workspace.md) for vibe-coding with workspaces
