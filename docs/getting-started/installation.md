# Installation

This guide covers all installation methods for Ulf Orchestrator.

## Prerequisites

### AI CLI Tools

Ulf needs at least one AI CLI tool to function. Install one of the following:

=== "Claude Code (Recommended)"

    ```bash
    # Via npm
    npm install -g @anthropic-ai/claude-code

    # Or visit https://claude.ai/code for setup instructions
    ```

=== "Kiro"

    ```bash
    # Visit https://kiro.dev/ for installation
    ```

=== "Gemini CLI"

    ```bash
    npm install -g @google/gemini-cli
    ```

=== "Codex"

    ```bash
    # Visit https://github.com/openai/codex
    ```

=== "Amp"

    ```bash
    # Visit https://github.com/sourcegraph/amp
    ```

=== "Copilot CLI"

    ```bash
    npm install -g @github/copilot
    ```

=== "OpenCode"

    ```bash
    curl -fsSL https://opencode.ai/install | bash
    ```

## Installing Ulf

### Via npm (Recommended)

The easiest way to install Ulf:

```bash
# Install globally
npm install -g @ulf-orchestrator/ulf-cli

# Or run directly with npx
npx @ulf-orchestrator/ulf-cli --version
```

### Via GitHub Releases installer

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/mikeyobrien/ulf-orchestrator/releases/latest/download/ulf-cli-installer.sh | sh
```

### Via Cargo

If you have Rust installed:

```bash
cargo install ulf-cli
```

### From Source

For the latest development version:

```bash
# Clone the repository
git clone https://github.com/mikeyobrien/ulf-orchestrator.git
cd ulf-orchestrator

# Build release binary
cargo build --release

# Add to PATH
export PATH="$PATH:$(pwd)/target/release"

# Or create symlink
sudo ln -s $(pwd)/target/release/ulf /usr/local/bin/ulf
```

## First-Time Setup

After installing Ulf, run the interactive install agent for guided setup:

```bash
# Guided onboarding — asks about backends, workspaces, Telegram, hooks, etc.
ulf run --config presets/install-agent.yml
```

This interactive agent will:
1. Detect which AI backends you already have installed
2. Help you install one if needed
3. Ask about workspace mode, Telegram, hooks, preflight checks, shell completions
4. Generate a complete `~/.ulf/config.yml` tailored to your choices
5. Introduce key concepts and suggest your first tasks

### Example Configurations

Ulf ships with example configs you can copy and customize:

```bash
# Minimal workspace setup
cp examples/configs/minimal-workspace.yml ~/.ulf/config.yml

# Team shared config with guardrails
cp examples/configs/team-shared.yml ~/.ulf/config.yml

# Multi-backend with per-hat assignments
cp examples/configs/multi-backend.yml ~/.ulf/config.yml

# With Telegram integration
cp examples/configs/with-telegram.yml ~/.ulf/config.yml

# With lifecycle hooks
cp examples/configs/with-hooks.yml ~/.ulf/config.yml

# With Agent Waves for parallel execution
cp examples/configs/with-waves.yml ~/.ulf/config.yml
```

### Manual Configuration

Create your user configuration manually:

```bash
mkdir -p ~/.ulf

cat > ~/.ulf/config.yml << 'EOF'
# ~/.ulf/config.yml — User-level defaults
# This file is loaded automatically for every Ulf session.

cli:
  backend: "claude"

event_loop:
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 100

# Default setup prompt for new workspaces (multi-workspace mode)
workspace:
  default_setup_prompt: |
    Create a CLAUDE.md for this workspace with:
    - Project overview and tech stack
    - Build, test, and lint commands
    - Code style and architecture conventions

  # Backend presets for per-workspace or per-session overrides
  backend_presets:
    claude-verbose:
      backend: "claude"
      args: ["--verbose"]
    kiro-fast:
      backend: "kiro"
      idle_timeout_secs: 60

  # Middle-manager prompt extensions
  middle_manager:
    backend_preset: "claude-verbose"
    prompt_extensions:
      - "Always run tests before suggesting the task is complete."
      - "Prefer small, focused commits with descriptive messages."
EOF
```

## Verify Installation

```bash
# Check version
ulf --version

# Show help
ulf --help

# List available presets
ulf init --list-presets

# Run environment diagnostics
ulf doctor
```

## Migrating from v1 (Legacy)

If you have the legacy Ulf v1 installed, uninstall it first:

```bash
# If installed via pip
pip uninstall ulf-orchestrator

# If installed via pipx
pipx uninstall ulf-orchestrator

# If installed via uv
uv tool uninstall ulf-orchestrator

# Verify removal
which ulf  # Should return nothing or point to new Rust version
```

The v1 release is no longer maintained. See [Migration from v1](../reference/migration-v1.md) for details.

## Troubleshooting

### Command Not Found

If `ulf` is not found after installation:

```bash
# For npm global installs, ensure npm bin is in PATH
export PATH="$PATH:$(npm config get prefix)/bin"

# For cargo installs
export PATH="$PATH:$HOME/.cargo/bin"
```

### No AI Agents Detected

Ulf auto-detects available AI CLI tools. If none are found:

1. Install one of the supported AI CLI tools (see Prerequisites)
2. Ensure the tool is in your PATH
3. Try running the AI CLI directly to verify it works

### Permission Denied

If you get permission errors:

```bash
# For npm
sudo npm install -g @ulf-orchestrator/ulf-cli

# For symlinks
sudo ln -s $(pwd)/target/release/ulf /usr/local/bin/ulf
```

## Next Steps

Now that Ulf is installed, proceed to the [Quick Start](quick-start.md) guide.
