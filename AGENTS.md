# AI Agent Context

## Project: hello_world

Rust project with pipeline processing system + m_log log filtering infrastructure.

## Architecture

- **m_log/src/lib.rs**: Standalone reusable package — ModuleLog trait + SimpleLogger + 4 macros + re-export define_module
- **m_log/m_log_macros/src/lib.rs**: Internal proc-macro crate — define_module! (generates struct + m_log::ModuleLog impl + convenience macros)
- **src/main.rs**: Demo entry point with define_module!(Main, ...) registration
- **src/lib.rs**: Library exports (Data, Processor, ProcessorModule, Pipeline, PipelineModule, ModuleLog, define_module)
- **src/pipeline.rs**: Pipeline chain processing + define_module!(PipelineModule, ...) logging
- **src/processor.rs**: Processor trait definition + define_module!(ProcessorModule, ...) logging
- **src/data.rs**: Data trait definition
- **src/error.rs**: PipelineError definition

## m_log Usage Pattern

```rust
use m_log::{define_module, m_info, m_error, ModuleLog, init as m_log_init};

define_module!(MyModule, info=true, warn=true, error=true, debug=false);
m_log_init();  // Call once at startup
mymodule_info!("log message");  // Convenience macro (only within defining module)
m_info!(MyModule, "log message");  // Direct macro (works cross-module)
```

Cross-module usage:
```rust
// In pipeline.rs: define_module!(PipelineModule, ...)
// In main.rs:    m_info!(PipelineModule, "msg") — requires pub export of PipelineModule
```

## Key Conventions

- m_log/src/lib.rs must contain NO business module hardcoding (REQ-009)
- Convenience macro names: module name lowercase + level (e.g. main_info!, pipeline_error!)
- Convenience macros (macro_rules!) only visible within defining module; use m_info!(Module, ...) for cross-module
- #[macro_export] macros at m_log crate root; convenience macros local macro_rules!
- m_log is a standalone reusable package; m_log_macros is its internal proc-macro dependency
- proc-macro generates `m_log::ModuleLog` (not `crate::m_log::ModuleLog`)
- GNU toolchain: use `cargo test -p hello_world` (not `--all`, dlltool issue)

## Verification Commands

- `cargo build` - Compile check
- `cargo test -p hello_world` - Run all tests
- `cargo run` - Verify log output
- `rg "println!" src/` - Verify no println! in business code
- `rg "Pipeline|Data|MainModule" m_log/src/lib.rs` - Verify no hardcoding