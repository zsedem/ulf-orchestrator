# Examples

Practical examples showing Ulf in action.

## In This Section

| Example | Description |
|---------|-------------|
| [Simple Task](simple-task.md) | Basic traditional mode usage |
| [TDD Workflow](tdd-workflow.md) | Test-driven development with hats |
| [Automated PDD Design](pdd-design.md) | Example-only design workflow with simulated requirements interview |
| [Spec-Driven Development](spec-driven.md) | Example-only workflow pattern, not a shipped builtin |
| [Multi-Hat Workflow](multi-hat.md) | Complex coordination between hats |
| [Debugging](debugging.md) | Using Ulf to investigate bugs |

## Quick Examples

### Multi-Workspace Mode

Create an isolated workspace and attach a middle-manager:

```bash
# Create a workspace for your task
ulf workspace create factorial-task --name "Build factorial function"

# Attach and start coding
ulf workspace attach factorial-task
# Inside: "Write a function that calculates factorial. Include tests."

# Check on it later
ulf workspace status
```

### Traditional Mode

Simple loop until completion:

```bash
ulf init --backend claude

cat > PROMPT.md << 'EOF'
Write a function that calculates factorial.
Include tests.
EOF

ulf run
```

### Hat-Based Mode

Using a built-in hat collection:

```bash
ulf init --backend claude

cat > PROMPT.md << 'EOF'
Implement a URL validator function.
Must handle:
- HTTP and HTTPS protocols
- IPv4 addresses
- Domain names
- Port numbers
EOF

ulf run -c ulf.yml -H builtin:code-assist
```

### Inline Prompts

Skip the prompt file:

```bash
ulf run -p "Add input validation to the signup form"
```

### Custom Configuration

Override defaults:

```bash
ulf run --max-iterations 50 -p "Refactor the authentication module"
```

## Example Workflows

### Feature Development (Multi-Workspace)

```bash
# Create a workspace for the feature
ulf workspace create user-dashboard --name "User Dashboard"

# Attach and describe the feature
ulf workspace attach user-dashboard
# Inside: "Add a user dashboard with profile summary, activity feed, and quick actions"
```

### Feature Development (Traditional)

```bash
# Initialize core config
ulf init --backend claude

# Create detailed prompt
cat > PROMPT.md << 'EOF'
# Feature: User Dashboard

Add a user dashboard with:
- Profile summary widget
- Recent activity feed
- Quick action buttons

Use React components.
Follow existing UI patterns.
EOF

# Run Ulf with the default implementation hats
ulf run -c ulf.yml -H builtin:code-assist
```

### Bug Investigation

```bash
# Use debug hat collection
ulf run -c ulf.yml -H builtin:debug -p "Users report login fails on Safari. Error: 'Invalid token'. Investigate and fix."
```

### Code Review

```bash
# Use review hat collection
ulf run -c ulf.yml -H builtin:review -p "Review the changes in src/api/auth.rs for security issues"
```

## Full Examples

Detailed walkthroughs are available:

- [Simple Task](simple-task.md) — Step-by-step traditional mode
- [TDD Workflow](tdd-workflow.md) — Red-green-refactor with hats
- [Automated PDD Design](pdd-design.md) — Simulated interview that ends with a reviewed design package
- [Spec-Driven](spec-driven.md) — Example specification-first pattern
- [Multi-Hat](multi-hat.md) — Complex hat coordination
- [Debugging](debugging.md) — Bug investigation workflow
