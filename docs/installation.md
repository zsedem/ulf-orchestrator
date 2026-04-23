# Installation Guide

Comprehensive installation instructions for Ulf Orchestrator.

## Prerequisites

- **OS**: macOS, Linux, or Windows
- **Node.js**: 18+ (required for npm installs)
- **Rust**: 1.70+ (required for cargo installs)

## Installation Methods

### Method 1: npm (Recommended)

```bash
npm install -g @ulf-orchestrator/ulf-cli
```

### Method 2: GitHub Releases installer

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/mikeyobrien/ulf-orchestrator/releases/latest/download/ulf-cli-installer.sh | sh
```

### Method 3: Cargo

```bash
cargo install ulf-cli
```

### Method 4: Prebuilt Binary (cargo-dist)

Download the latest `ulf-cli-<target>.tar.xz` artifact from GitHub Releases, extract it, then place `ulf` on your PATH.

```bash
# Example (replace with the correct archive for your platform)
mkdir -p ~/bin
curl -L -o ulf.tar.xz "<release-archive-url>"
tar -xJf ulf.tar.xz
mv ulf ~/bin/
export PATH="$HOME/bin:$PATH"
```

> Homebrew is not currently published from this repository's automated release flow.

## Verify Installation

```bash
ulf --version
```

## Next Steps

- Install at least one supported AI backend CLI (Claude Code, Gemini CLI, Copilot CLI, etc.)
- Configure your backend API keys or auth
- Follow the quick start guide: `getting-started/quick-start.md`
