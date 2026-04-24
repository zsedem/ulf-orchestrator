# Example Configurations

These example YAML configurations demonstrate common Ulf setups. Copy one to `~/.ulf/config.yml` as a starting point, or mix and match sections.

## Files

| File | Use Case |
|------|----------|
| `minimal-workspace.yml` | Simple workspace-based vibe-coding with defaults |
| `team-shared.yml` | Team defaults with guardrails and backend presets |
| `multi-backend.yml` | Different backends for different hats in a workflow |
| `with-telegram.yml` | Human-in-the-loop via Telegram |
| `with-hooks.yml` | Lifecycle hooks for CI/CD integration |
| `with-waves.yml` | Agent Waves for parallel hat execution |

## Usage

```bash
mkdir -p ~/.ulf

# Pick one and copy
cp examples/configs/minimal-workspace.yml ~/.ulf/config.yml

# Or start from the install agent for guided setup
ulf run --config presets/install-agent.yml
```

## Config Layers

Ulf merges configuration in this order:

1. `~/.ulf/config.yml` — User-level defaults (loaded automatically)
2. `./ulf.yml` — Project-level overrides
3. `-c core.field=value` — CLI overrides applied last

Mappings are deep-merged; scalars and arrays are replaced.
