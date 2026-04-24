# Backends

Ulf supports multiple AI CLI backends. This guide covers setup and selection.

## Supported Backends

| Backend | CLI Tool | Notes |
|---------|----------|-------|
| Claude Code | `claude` | Recommended, primary support |
| Kiro | `kiro-cli` | Amazon/AWS |
| Gemini CLI | `gemini` | Google |
| Codex | `codex` | OpenAI |
| Amp | `amp` | Sourcegraph |
| Copilot CLI | `copilot` | GitHub |
| OpenCode | `opencode` | Community |
| Pi | `pi` | Multi-provider |

## Auto-Detection

Ulf automatically detects installed backends:

```bash
ulf init
# Auto-detects available backend
```

Detection order (first available wins):
1. Claude
2. Kiro
3. Gemini
4. Codex
5. Amp
6. Copilot
7. OpenCode
8. Pi

## Explicit Selection

Override auto-detection:

```bash
# Via CLI
ulf init --backend kiro
ulf run --backend gemini

# Via config
# ulf.yml
cli:
  backend: "claude"
```

## Backend Setup

Each backend below includes:
- **Install** instructions
- **Auth & env vars** (API keys or login)
- **Hat YAML** configuration
- **`ulf doctor`** validation notes

Backend names (used in YAML and CLI flags): `claude`, `kiro`, `gemini`, `codex`, `amp`, `copilot`, `opencode`, `pi`.

### Claude Code (`claude`)

The recommended backend with full feature support.

```bash
# Install
npm install -g @anthropic-ai/claude-code

# Authenticate
claude login

# Verify
claude --version
```

**Auth & env vars:**
- `claude login` (preferred)
- `ANTHROPIC_API_KEY` (used by `ulf doctor` auth hints)

**Hat YAML:**
```yaml
hats:
  planner:
    backend: "claude"
```

**Doctor checks:**
- `claude --version` must succeed
- Warns if `ANTHROPIC_API_KEY` is missing

**Features:**
- Full streaming support
- All hat features
- Memory integration

### Kiro (`kiro`)

Amazon/AWS AI assistant.

```bash
# Install
# Visit https://kiro.dev/

# Verify
kiro-cli --version
```

**Auth & env vars:**
- Complete Kiro CLI authentication (AWS/SSO) per Kiro docs
- `KIRO_API_KEY` (optional; used by `ulf doctor` auth hints)

**Hat YAML:**
```yaml
hats:
  coder:
    backend: "kiro"
```

**Kiro agent selection (optional):**
```yaml
hats:
  reviewer:
    backend:
      type: "kiro"
      agent: "codex"
```

**Doctor checks:**
- `kiro-cli --version` must succeed
- Warns if `KIRO_API_KEY` is missing (OK if you authenticated via CLI)

### Gemini CLI (`gemini`)

Google's AI CLI.

```bash
# Install
npm install -g @google/gemini-cli

# Configure API key
export GEMINI_API_KEY=your-key

# Verify
gemini --version
```

**Auth & env vars:**
- `GEMINI_API_KEY` (used by `ulf doctor` auth hints)

**Hat YAML:**
```yaml
hats:
  analyst:
    backend: "gemini"
```

**Doctor checks:**
- `gemini --version` must succeed
- Warns if `GEMINI_API_KEY` is missing

### Codex (`codex`)

OpenAI's code-focused model.

```bash
# Install
# Visit https://github.com/openai/codex

# Configure
export OPENAI_API_KEY=your-key

# Verify
codex --version
```

**Auth & env vars:**
- `OPENAI_API_KEY` or `CODEX_API_KEY` (either satisfies `ulf doctor` auth hints)

**Hat YAML:**
```yaml
hats:
  coder:
    backend: "codex"
```

**Doctor checks:**
- `codex --version` must succeed
- Warns if neither `OPENAI_API_KEY` nor `CODEX_API_KEY` is set

### Amp (`amp`)

Sourcegraph's AI assistant.

```bash
# Install
# Visit https://github.com/sourcegraph/amp

# Verify
amp --version
```

**Auth & env vars:**
- Authenticate via `amp` CLI per Sourcegraph docs
- No auth env vars are checked by `ulf doctor` for Amp

**Hat YAML:**
```yaml
hats:
  helper:
    backend: "amp"
```

**Doctor checks:**
- `amp --version` must succeed

### Copilot CLI (`copilot`)

GitHub's AI assistant.

```bash
# Install
npm install -g @github/copilot

# Authenticate
copilot auth login

# Verify
copilot --version
```

**Auth & env vars:**
- Authenticate via Copilot CLI (`copilot auth login` or `gh auth login`)
- No auth env vars are checked by `ulf doctor` for Copilot

**Hat YAML:**
```yaml
hats:
  reviewer:
    backend: "copilot"
```

**Doctor checks:**
- `copilot --version` must succeed

### OpenCode (`opencode`)

Community AI CLI.

```bash
# Install
curl -fsSL https://opencode.ai/install | bash

# Verify
opencode --version
```

**Auth & env vars:**
- Set one of: `OPENCODE_API_KEY`, `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`
- OpenCode can proxy multiple providers; use the env var matching your provider

**Hat YAML:**
```yaml
hats:
  strategist:
    backend: "opencode"
```

**Doctor checks:**
- `opencode --version` must succeed
- Warns if none of `OPENCODE_API_KEY`, `ANTHROPIC_API_KEY`, `OPENAI_API_KEY` are set

### Pi (`pi`)

Multi-provider AI coding assistant.

```bash
# Install
npm install -g @mariozechner/pi-coding-agent

# Verify
pi --version
```

**Auth & env vars:**
- Set one of: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GEMINI_API_KEY`, or any supported provider key
- Pi routes to the provider specified via `--provider` (default: google)
- Pass API key explicitly with `--api-key` or rely on provider-specific env vars

**Hat YAML:**
```yaml
hats:
  coder:
    backend: "pi"
```

**Pi provider selection (optional):**
```yaml
hats:
  coder:
    backend:
      type: "pi"
      args: ["--provider", "anthropic", "--model", "claude-sonnet-4"]
```

**Doctor checks:**
- `pi --version` must succeed
- Warns if no provider API key is set

## Per-Hat Backend Override

Different hats can use different backends:

```yaml
hats:
  planner:
    backend: "claude"  # Use Claude for planning
    triggers: ["task.start"]
    instructions: "Create a plan..."

  coder:
    backend: "kiro"    # Use Kiro for coding
    triggers: ["plan.ready"]
    instructions: "Implement..."
```

## Custom Backends

For unsupported CLIs, use the custom backend:

```yaml
cli:
  backend: "custom"
  custom_command: "my-ai-cli"
  prompt_mode: "arg"  # or "stdin"
```

**Prompt modes:**

| Mode | How Prompt is Passed |
|------|---------------------|
| `arg` | `my-ai-cli -p "prompt"` |
| `stdin` | `echo "prompt" \| my-ai-cli` |

## Backend Comparison

| Feature | Claude | Kiro | Gemini | Codex | Pi |
|---------|--------|------|--------|-------|----|
| Streaming | Yes | Yes | Yes | Yes | Yes |
| Tool use | Full | Full | Partial | Partial | Full |
| Context size | Large | Large | Large | Medium | Large |
| Speed | Fast | Fast | Fast | Medium | Fast |
| Cost | $$ | $ | $ | $$ | $ |

## Troubleshooting

### Backend Not Found

```
ERROR: No AI agents detected
```

**Solution:**
1. Install a supported backend
2. Ensure it's in your PATH
3. Test directly: `claude -p "test"` or `pi -p "test"`

### Authentication Failed

```
ERROR: Authentication required
```

**Solution:**
```bash
# Claude
claude login

# Copilot
copilot auth login

# Gemini - set API key
export GEMINI_API_KEY=your-key

# Pi - set provider API key
export ANTHROPIC_API_KEY=your-key
```

If the CLI is already authenticated but `ulf doctor` still warns, ensure the
expected env vars above are set (doctor checks are hints, not hard failures).

### Wrong Backend Used

```bash
# Force specific backend
ulf run --backend claude

# Or set in config
cli:
  backend: "claude"
```

### Backend Hanging

Some backends need interactive authentication on first run:

```bash
# Run backend directly first
claude -p "test"

# Then use with Ulf
ulf run
```

## Backend Presets in Config

Define reusable backend configurations in `~/.ulf/config.yml`:

```yaml
# ~/.ulf/config.yml
workspace:
  backend_presets:
    # Default Claude with verbose output
    claude-verbose:
      backend: "claude"
      args: ["--verbose"]

    # Claude restricted to Bash and Edit tools
    claude-safe:
      backend: "claude"
      args: ["--allowedTools", "Bash", "Edit"]

    # Kiro with short timeout
    kiro-fast:
      backend: "kiro"
      idle_timeout_secs: 60

    # Gemini for quick tasks
    gemini-quick:
      backend: "gemini"
      args: ["--model", "gemini-pro"]

    # Codex with specific model
    codex-4o:
      backend: "codex"
      args: ["--model", "gpt-4o"]

    # Pi with Anthropic provider
    pi-anthropic:
      backend: "pi"
      args: ["--provider", "anthropic"]

  # Use a preset for middle-manager sessions
  middle_manager:
    backend_preset: "claude-verbose"
```

Use a preset when attaching to a workspace:

```bash
ulf workspace attach my-project --backend-preset claude-safe
```

Or set per-hat backend in a hat collection:

```yaml
hats:
  planner:
    backend: "claude"
    instructions: "Create a plan..."

  coder:
    backend: "kiro"
    instructions: "Implement the plan..."

  reviewer:
    backend: "gemini"
    instructions: "Review the code..."
```

## Best Practices

1. **Pick one primary backend** — Consistency helps
2. **Test backend directly** — Before using with Ulf
3. **Use per-hat overrides sparingly** — Can complicate debugging
4. **Keep backends updated** — New features, bug fixes
5. **Create backend presets** — For common backend configurations you reuse

## Next Steps

- Configure [Presets](presets.md) for your workflow
- Learn about [Cost Management](cost-management.md)
- Explore [Writing Prompts](prompts.md)
