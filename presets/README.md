# Ulf Hat Collections

This directory contains the canonical built-in hat collections Ulf still ships and supports.

Built-ins are embedded into the CLI from these files and exposed through `ulf init --list-presets`.

## Supported Builtins

| Collection | Source | Best for |
|---|---|---|
| `autoresearch` | `presets/autoresearch.yml` | Autonomous experiment loop for any measurable improvement |
| `code-assist` | `presets/code-assist.yml` | Default implementation workflow |
| `debug` | `presets/debug.yml` | Investigation and fix verification |
| `research` | `presets/research.yml` | Read-only exploration and synthesis |
| `review` | `presets/review.yml` | Adversarial code review |
| `pdd-to-code-assist` | `presets/pdd-to-code-assist.yml` | Advanced end-to-end idea-to-code workflow |

## Internal Presets

These remain loadable for Ulf internals or testing, but are intentionally hidden from normal builtin listings:

- `hatless-baseline`
- `merge-loop`

## Product Positioning

- `code-assist` is the recommended default for implementation work.
- `pdd-to-code-assist` is intentionally kept as an advanced, fun example. It is slower, more expensive, and less predictable than `code-assist`.
- Other historical presets are now treated as documentation examples instead of supported builtins.

## Quick Start

```bash
ulf init --backend claude
ulf init --list-presets

ulf run -c ulf.yml -H builtin:autoresearch -p "Improve test coverage in src/core/"
ulf run -c ulf.yml -H builtin:code-assist -p "Add OAuth login"
ulf run -c ulf.yml -H builtin:debug -p "Investigate intermittent timeout"
ulf run -c ulf.yml -H builtin:research -p "Map auth architecture"
ulf run -c ulf.yml -H builtin:review -p "Review changes in src/api/"
ulf run -c ulf.yml -H builtin:pdd-to-code-assist -p "Build a new import pipeline"
```

## Examples Instead of Builtins

Example workflow patterns now live in the docs rather than as shipped preset files. See:

- `docs/examples/`
- `presets/COLLECTION.md`

## Source Of Truth

- Canonical builtins: `presets/*.yml`
- Builtin index: `presets/index.json`
- Embedded CLI mirror: `crates/ulf-cli/presets/*.yml`
- Sync script: `./scripts/sync-embedded-files.sh`
