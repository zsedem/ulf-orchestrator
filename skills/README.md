# Ulf Orchestrator Agent Skills

This directory is the canonical public skill package for external agent
harnesses that operate Ulf.

It ships two skills:

- `ulf-hats` for creating, inspecting, validating, and improving hat
  collections
- `ulf-loop` for running, monitoring, resuming, merging, and debugging Ulf
  loops

These are public agent skills. They are not part of Ulf's internal
`ulf tools skill` registry.

## Install with Claude Code

Add this repository as a marketplace source:

```text
/plugin marketplace add mikeyobrien/ulf-orchestrator
```

Then install the `ulf-orchestrator` plugin from the marketplace browser.

## Install with Vercel `npx skills`

List the skills in this repository:

```bash
npx skills add mikeyobrien/ulf-orchestrator --list
```

Install both skills for Claude Code:

```bash
npx skills add mikeyobrien/ulf-orchestrator \
  --skill ulf-hats \
  --skill ulf-loop \
  -a claude-code \
  -y
```

Install one skill for Codex-style agents:

```bash
npx skills add mikeyobrien/ulf-orchestrator \
  --skill ulf-loop \
  -a codex \
  -y
```

During local development you can also install from the checked-out repo:

```bash
npx skills add . --list
```
