# API Reference

Technical reference documentation for Ulf's crates.

## Crate Overview

| Crate | Purpose | Documentation |
|-------|---------|---------------|
| [ulf-proto](ulf-proto.md) | Protocol types: Event, Hat, Topic | Core data structures |
| [ulf-core](ulf-core.md) | Orchestration engine | EventLoop, Config |
| [ulf-adapters](ulf-adapters.md) | CLI backends | Backend integrations |
| [ulf-tui](ulf-tui.md) | Terminal UI | TUI components |
| [ulf-cli](ulf-cli.md) | Binary entry point | CLI commands |

## Quick Links

### Core Types

```rust
// Events
use ulf_proto::{Event, Topic, EventBus};

// Hats
use ulf_proto::{Hat, HatId};

// Configuration
use ulf_core::config::{Config, EventLoopConfig, CliConfig};
```

### Common Operations

```rust
// Load configuration
let config = Config::load("ulf.yml")?;

// Create event loop
let event_loop = EventLoop::new(config);

// Run orchestration
event_loop.run().await?;
```

## Rust Documentation

Generate and view Rust docs:

```bash
# Generate docs
cargo doc --no-deps --open

# Generate with dependencies
cargo doc --open
```

## Stability

| Crate | Status |
|-------|--------|
| ulf-proto | Stable |
| ulf-core | Stable |
| ulf-adapters | Stable |
| ulf-tui | Experimental |
| ulf-cli | Stable |
| ulf-e2e | Internal |
| ulf-bench | Internal |

"Stable" means the public API is unlikely to change in breaking ways.
"Experimental" means the API may change.
"Internal" means the crate is not intended for external use.
