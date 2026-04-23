# Migrating from Q Chat to Kiro CLI

The Amazon Q Developer CLI has been rebranded to **Kiro CLI** (v1.20+). Ulf Orchestrator v1.2.3+ fully supports this transition with the new `KiroAdapter`.

This guide helps you migrate your existing Q Chat configurations and workflows to Kiro CLI.

## Quick Summary

- **New Command:** `kiro-cli` (replaces `q`)
- **Adapter Flag:** `-a kiro` (replaces `-a q` or `-a qchat`)
- **Config Section:** `adapters.kiro` (replaces `adapters.qchat`)
- **Environment Vars:** `ULF_KIRO_*` (replaces `ULF_QCHAT_*`)

## Command Line Changes

To run Ulf with Kiro CLI:

```bash
# New way
ulf run -a kiro

# Legacy way (still works but deprecated)
ulf run -a q
ulf run -a qchat
```

If `kiro-cli` is not found, Ulf will automatically fall back to the `q` command if available, preserving backward compatibility.

## Configuration Changes

### ulf.yml

Update your `ulf.yml` configuration to use the new `kiro` section. The `q` and `qchat` sections are deprecated but still supported.

```yaml
# New Configuration
adapters:
  kiro:
    enabled: true
    timeout: 600
    args: []
    env: {}

# Deprecated Configuration
# adapters:
#   q:
#     enabled: true
#     timeout: 600
```

### Environment Variables

Update your environment variables to the new namespace:

| Legacy Variable | New Variable | Default |
|----------------|--------------|---------|
| `ULF_QCHAT_COMMAND` | `ULF_KIRO_COMMAND` | `kiro-cli` |
| `ULF_QCHAT_TIMEOUT` | `ULF_KIRO_TIMEOUT` | `600` |
| `ULF_QCHAT_PROMPT_FILE` | `ULF_KIRO_PROMPT_FILE` | `PROMPT.md` |
| `ULF_QCHAT_TRUST_TOOLS` | `ULF_KIRO_TRUST_TOOLS` | `true` |
| `ULF_QCHAT_NO_INTERACTIVE` | `ULF_KIRO_NO_INTERACTIVE` | `true` |

## System Paths

The Kiro CLI uses new directory paths for configuration and data. Ulf's adapter is aware of these changes, but you should update any manual setups or scripts.

| Component | Legacy Path (Q Chat) | New Path (Kiro) |
|-----------|----------------------|-----------------|
| **MCP Servers** | `~/.aws/amazonq/mcp.json` | `~/.kiro/settings/mcp.json` |
| **Prompts** | `~/.aws/amazonq/prompts` | `~/.kiro/prompts` |
| **Project Config** | `.amazonq/` | `.kiro/` |
| **Global Config** | `~/.aws/amazonq/` | `~/.kiro/` |
| **Logs** | `$TMPDIR/qchat-log` | `$TMPDIR/kiro-log` |

## Migration Steps

1.  **Install Kiro CLI**: Ensure you have installed the new Kiro CLI (version 1.20 or later).
2.  **Update Config**: Update your `ulf.yml` to replace `q` adapter config with `kiro`.
3.  **Update Scripts**: Change any CI/CD or startup scripts to use `ulf run -a kiro`.
4.  **Move MCP Config**: If you use custom MCP servers, move your `mcp.json` to the new location:
    ```bash
    mkdir -p ~/.kiro/settings
    cp ~/.aws/amazonq/mcp.json ~/.kiro/settings/mcp.json
    ```

## Backward Compatibility

Ulf maintains full backward compatibility:
- Running `-a q` still works (uses `KiroAdapter` internally with legacy settings).
- If `kiro-cli` is missing, it falls back to `q`.
- Old environment variables (`ULF_QCHAT_*`) are NOT automatically read by the `KiroAdapter` to strictly separate configurations, but the legacy `QChatAdapter` (which reads them) redirects to `KiroAdapter` logic where possible or operates as a fallback.

> **Note:** The `QChatAdapter` class is now deprecated and emits a warning when initialized.
