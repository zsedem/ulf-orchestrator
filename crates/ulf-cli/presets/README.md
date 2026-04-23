# Ulf Hat Collections

This directory contains the embedded hat collection files packaged into the `ulf` binary.

The editable source of truth lives in `presets/*.yml`; these files are mirrors used for `include_str!` packaging.

## Supported Builtins

| Collection | Canonical source | Best for |
|---|---|---|
| `code-assist` | `presets/code-assist.yml` | Default implementation workflow |
| `debug` | `presets/debug.yml` | Investigation and fix verification |
| `research` | `presets/research.yml` | Read-only exploration and synthesis |
| `review` | `presets/review.yml` | Adversarial code review |
| `pdd-to-code-assist` | `presets/pdd-to-code-assist.yml` | Advanced end-to-end idea-to-code workflow |

## Internal Presets

- `hatless-baseline`
- `merge-loop`

These stay embedded for Ulf internals and testing, but are hidden from normal preset listings.

## Quick Start

```bash
ulf init --backend claude
ulf init --list-presets

ulf run -c ulf.yml -H builtin:code-assist -p "Add OAuth login"
ulf run -c ulf.yml -H builtin:debug -p "Investigate intermittent timeout"
ulf run -c ulf.yml -H builtin:research -p "Map auth architecture"
ulf run -c ulf.yml -H builtin:review -p "Review changes in src/api/"
ulf run -c ulf.yml -H builtin:pdd-to-code-assist -p "Build a new import pipeline"
```

## Notes

- `code-assist` is the recommended default for implementation work.
- `pdd-to-code-assist` is kept as an advanced, fun example workflow rather than the default recommendation.
- Historical presets now belong in documentation examples, not the builtin product surface.
