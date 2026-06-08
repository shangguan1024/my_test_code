# AI Agent Context

## Project: hello_world

Rust project with pipeline processing system + m_log log filtering infrastructure.

## Architecture

- **src/m_log.rs**: Module-independent log filtering (ModuleLog trait + SimpleLogger + 4 macros)
- **m_log_macros/src/lib.rs**: define_module! proc macro (generates struct + trait impl + convenience macros)
- **src/main.rs**: Demo entry point with define_module!(Main, ...) registration
- **src/lib.rs**: Library exports (Data, Processor, Pipeline, ModuleLog, define_module)
- **src/pipeline.rs**: Pipeline chain processing
- **src/data.rs**: Data trait definition
- **src/processor.rs**: Processor trait definition
- **src/error.rs**: PipelineError definition

## m_log Usage Pattern

```rust
use hello_world::{m_log, define_module, m_info, m_error};

define_module!(MyModule, info=true, warn=true, error=true, debug=false);
m_log::init();  // Call once at startup
mymodule_info!("log message");  // Convenience macro
m_info!(MyModule, "log message");  // Direct macro
```

## Key Conventions

- m_log.rs must contain NO business module hardcoding (REQ-009)
- Convenience macro names: module name lowercase + level (e.g. main_info!, pipeline_error!)
- #[macro_export] macros at crate root; convenience macros local macro_rules!
- GNU toolchain: use `cargo test -p hello_world` (not `--all`, dlltool issue)

## Verification Commands

- `cargo build` - Compile check
- `cargo test -p hello_world` - Run all 28 tests
- `cargo run` - Verify log output
- `rg "println!" src/` - Verify no println! in business code
- `rg "Pipeline|Data|MainModule" src/m_log.rs` - Verify no hardcoding