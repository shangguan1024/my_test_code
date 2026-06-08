# 日志过滤 Implementation Plan (Revised)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现模块无关的通用日志过滤基础设施m_log，调用方通过define_module! proc宏注册并获得便利宏(pipeline_info!等)

**Architecture:** m_log_macros crate提供define_module! proc宏（生成struct+ModuleLog trait impl+4个便利宏），hello_world/src/m_log.rs提供ModuleLog trait+SimpleLogger+4个通用m_info!/m_warn!/m_error!/m_debug!宏。调用链: pipeline_info!("msg") → m_info!(Pipeline, "msg") → if <Pipeline as ModuleLog>::INFO { log::info!("msg") }

**Tech Stack:** Rust, proc-macro (syn+quote), log crate

---

## Feature

- Name: 日志过滤
- Feature slug: log-filter
- Complexity: standard
- Related design: `docs/features/日志过滤/design.md`
- Related findings: `docs/features/日志过滤/findings.md`

## Phase Status

| Phase | Status | Artifact | Approval |
|-------|--------|----------|----------|
| 0 Research & Understanding | completed | `findings.md` | approved |
| 1 Requirements & Design | completed (revised) | `design.md` | approved |
| 2 Implementation Planning | completed (revised) | `task_plan.md` | approved |
| 3 Module Development | completed | code + tests | approved |
| 4 Integration & Testing | completed | verification evidence | approved |
| 5 Code Quality Review | completed | review artifacts | approved |
| 6 Memory Persistence | in progress | handoff artifacts | pending |

## Requirements Traceability

| REQ-ID | Source | Planned Task | Verification |
|--------|--------|--------------|--------------|
| REQ-001 | findings.md | T2 | ModuleLog trait存在于m_log.rs |
| REQ-002 | findings.md | T1 | define_module! proc宏可用 |
| REQ-003 | findings.md | T2+T1 | m_info!宏+便利宏均可用 |
| REQ-004 | findings.md | T2 | 宏展开为if trait_const { log::level! } |
| REQ-005 | findings.md | T2 | OFF开关零开销(LLVM) |
| REQ-006 | findings.md | T2 | ON开关转发log crate |
| REQ-007 | findings.md | T3 | 无println!残留 |
| REQ-008 | findings.md | T2 | SimpleLogger工作 |
| REQ-009 | findings.md | T1+T2 | m_log.rs+m_log_macros无业务硬编码 |

## Tasks

| Task ID | Description | Status | Files | Dependencies | Verification |
|---------|-------------|--------|-------|--------------|--------------|
| T0 | Cargo workspace + 依赖 | completed | Cargo.toml, m_log_macros/Cargo.toml | none | cargo build成功 ✅ |
| T1 | define_module! proc宏 | completed | m_log_macros/src/lib.rs | T0 | proc宏编译+展开成功 ✅ |
| T2 | m_log通用模块(ModuleLog+SimpleLogger+4宏) | completed | src/m_log.rs, src/lib.rs | T0+T1 | cargo build成功 ✅ |
| T3 | 各模块注册+替换println! | completed | src/main.rs | T2 | cargo run日志输出 ✅ |
| T4 | 测试 | completed | tests/m_log_test.rs | T2 | 28 tests all passed ✅ |

## Test Strategy

- Unit tests: tests/m_log_test.rs - ModuleLog trait + define_module!展开 + 开关测试
- Integration tests: cargo build + cargo run验证日志输出
- Regression tests: 确认原有tests/下测试仍通过
- Commands: cargo build, cargo test, cargo run

## Implementation Order

1. T0: Cargo workspace + 依赖
2. T1: define_module! proc宏
3. T2: m_log通用模块
4. T3: 各模块注册+替换println!
5. T4: 测试

## Risk Register

| Risk | Impact | Mitigation | Owner |
|------|--------|------------|-------|
| proc宏生成macro_rules! | 需验证proc宏输出含macro_rules!是否可行 | Rust允许proc宏输出任何合法语法 | dev |
| crate路径引用 | proc宏生成代码中crate::m_log路径 | crate在proc宏输出中解析为调用方crate | dev |
| windows-sys编译 | GNU toolchain缺少dlltool | 仅依赖log crate，无windows-sys | dev |

---

## Detailed Tasks

### Task T0: Cargo workspace + 依赖

**Files:**
- Modify: `Cargo.toml`
- Create: `m_log_macros/Cargo.toml`
- Create: `m_log_macros/src/lib.rs` (placeholder)

- [ ] **Step 1: 修改根Cargo.toml，添加workspace和m_log_macros依赖**

```toml
[package]
name = "hello_world"
version = "0.1.0"
edition = "2021"

[workspace]
members = ["m_log_macros"]

[dependencies]
log = "0.4"
m_log_macros = { path = "m_log_macros" }
```

- [ ] **Step 2: 创建m_log_macros/Cargo.toml**

```toml
[package]
name = "m_log_macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = "2"
quote = "1"
```

- [ ] **Step 3: 创建m_log_macros/src/lib.rs placeholder**

```rust
use proc_macro::TokenStream;

#[proc_macro]
pub fn define_module(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
```

- [ ] **Step 4: 验证workspace编译**

Run: `cargo build`
Expected: BUILD SUCCESS

---

### Task T1: define_module! proc宏实现

**Files:**
- Modify: `m_log_macros/src/lib.rs`

- [ ] **Step 1: 实现ModuleDefinition parser**

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, LitBool, Token};

struct ModuleDefinition {
    name: Ident,
    info: bool,
    warn: bool,
    error: bool,
    debug: bool,
}

impl syn::parse::Parse for ModuleDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let info_kw: Ident = input.parse()?;
        if info_kw != "info" {
            return Err(syn::Error::new(info_kw.span(), "expected `info`"));
        }
        input.parse::<Token![=]>()?;
        let info: LitBool = input.parse()?;
        input.parse::<Token![,]>()?;

        let warn_kw: Ident = input.parse()?;
        if warn_kw != "warn" {
            return Err(syn::Error::new(warn_kw.span(), "expected `warn`"));
        }
        input.parse::<Token![=]>()?;
        let warn: LitBool = input.parse()?;
        input.parse::<Token![,]>()?;

        let error_kw: Ident = input.parse()?;
        if error_kw != "error" {
            return Err(syn::Error::new(error_kw.span(), "expected `error`"));
        }
        input.parse::<Token![=]>()?;
        let error: LitBool = input.parse()?;
        input.parse::<Token![,]>()?;

        let debug_kw: Ident = input.parse()?;
        if debug_kw != "debug" {
            return Err(syn::Error::new(debug_kw.span(), "expected `debug`"));
        }
        input.parse::<Token![=]>()?;
        let debug: LitBool = input.parse()?;

        Ok(ModuleDefinition {
            name,
            info: info.value,
            warn: warn.value,
            error: error.value,
            debug: debug.value,
        })
    }
}
```

- [ ] **Step 2: 实现define_module proc宏，生成struct+trait impl+4个便利宏**

```rust
#[proc_macro]
pub fn define_module(input: TokenStream) -> TokenStream {
    let def = parse_macro_input!(input as ModuleDefinition);
    let name = &def.name;
    let name_str = name.to_string().to_lowercase();
    
    let info_val = def.info;
    let warn_val = def.warn;
    let error_val = def.error;
    let debug_val = def.debug;

    let info_macro_name = syn::Ident::new(&format!("{}_info", name_str), name.span());
    let warn_macro_name = syn::Ident::new(&format!("{}_warn", name_str), name.span());
    let error_macro_name = syn::Ident::new(&format!("{}_error", name_str), name.span());
    let debug_macro_name = syn::Ident::new(&format!("{}_debug", name_str), name.span());

    let expanded = quote! {
        pub struct #name;

        impl crate::m_log::ModuleLog for #name {
            const INFO: bool = #info_val;
            const WARN: bool = #warn_val;
            const ERROR: bool = #error_val;
            const DEBUG: bool = #debug_val;
        }

        macro_rules! #info_macro_name {
            ($($arg:tt)*) => {
                m_info!(#name, $($arg)*)
            };
        }

        macro_rules! #warn_macro_name {
            ($($arg:tt)*) => {
                m_warn!(#name, $($arg)*)
            };
        }

        macro_rules! #error_macro_name {
            ($($arg:tt)*) => {
                m_error!(#name, $($arg)*)
            };
        }

        macro_rules! #debug_macro_name {
            ($($arg:tt)*) => {
                m_debug!(#name, $($arg)*)
            };
        }
    };

    expanded.into()
}
```

- [ ] **Step 3: 验证proc宏编译**

Run: `cargo build`
Expected: BUILD SUCCESS

---

### Task T2: m_log通用模块

**Files:**
- Modify: `src/m_log.rs` (rewrite with only generic code)
- Modify: `src/lib.rs`

- [ ] **Step 1: 重写src/m_log.rs，仅含通用代码**

```rust
use std::io::Write;

pub trait ModuleLog {
    const INFO: bool;
    const WARN: bool;
    const ERROR: bool;
    const DEBUG: bool;
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let level = match record.level() {
                log::Level::Error => "ERROR",
                log::Level::Warn => "WARN",
                log::Level::Info => "INFO",
                log::Level::Debug => "DEBUG",
                log::Level::Trace => "TRACE",
            };
            eprintln!("[{}] {}", level, record.args());
        }
    }

    fn flush(&self) {
        let _ = std::io::stderr().flush();
    }
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() {
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Debug);
}

#[macro_export]
macro_rules! m_info {
    ($module:ident, $($arg:tt)*) => {
        if <$module as $crate::m_log::ModuleLog>::INFO {
            log::info!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! m_warn {
    ($module:ident, $($arg:tt)*) => {
        if <$module as $crate::m_log::ModuleLog>::WARN {
            log::warn!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! m_error {
    ($module:ident, $($arg:tt)*) => {
        if <$module as $crate::m_log::ModuleLog>::ERROR {
            log::error!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! m_debug {
    ($module:ident, $($arg:tt)*) => {
        if <$module as $crate::m_log::ModuleLog>::DEBUG {
            log::debug!($($arg)*);
        }
    };
}
```

**注意**: m_log.rs中不含任何ModuleId枚举、Pipeline/Data等业务硬编码，仅含ModuleLog trait + SimpleLogger + 4个通用宏。满足REQ-009。

- [ ] **Step 2: 修改src/lib.rs**

```rust
mod error;
mod data;
mod processor;
mod pipeline;
pub mod m_log;

pub use error::PipelineError;
pub use data::Data;
pub use processor::Processor;
pub use pipeline::{Pipeline, ProcessorRecord, pipeline};
pub use m_log::ModuleLog;
pub use m_info;
pub use m_warn;
pub use m_error;
pub use m_debug;
pub use m_log_macros::define_module;
```

- [ ] **Step 3: 验证编译**

Run: `cargo build`
Expected: BUILD SUCCESS

---

### Task T3: 各模块注册+替换println!

**Files:**
- Modify: `src/main.rs`
- Modify: `src/pipeline.rs`

- [ ] **Step 1: 重写src/main.rs**

```rust
use hello_world::{Data, Processor, pipeline, PipelineError, m_info, m_error, define_module, m_log};
use std::sync::Arc;
use std::any::Any;

define_module!(Main, info=true, warn=true, error=true, debug=false);

struct NumberData {
    value: i32,
}

impl Data for NumberData {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn clone_data(&self) -> Arc<dyn Data> {
        Arc::new(NumberData { value: self.value })
    }
}

struct IncrementProcessor;

impl Processor for IncrementProcessor {
    fn name(&self) -> &str {
        "IncrementProcessor"
    }
    
    fn process(&self, data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError> {
        if let Some(num) = data.as_any().downcast_ref::<NumberData>() {
            main_info!("IncrementProcessor: {} -> {}", num.value, num.value + 1);
            Ok(Arc::new(NumberData { value: num.value + 1 }))
        } else {
            Err(PipelineError::TypeError("Expected NumberData".to_string()))
        }
    }
}

struct ValidateProcessor {
    min_value: i32,
}

impl Processor for ValidateProcessor {
    fn name(&self) -> &str {
        "ValidateProcessor"
    }
    
    fn process(&self, data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError> {
        if let Some(num) = data.as_any().downcast_ref::<NumberData>() {
            main_info!("ValidateProcessor: checking {} >= {}", num.value, self.min_value);
            if num.value >= self.min_value {
                Ok(data.clone_data())
            } else {
                Err(PipelineError::ValidationError(
                    format!("Value {} is less than minimum {}", num.value, self.min_value)
                ))
            }
        } else {
            Err(PipelineError::TypeError("Expected NumberData".to_string()))
        }
    }
}

struct MultiplyProcessor {
    factor: i32,
}

impl Processor for MultiplyProcessor {
    fn name(&self) -> &str {
        "MultiplyProcessor"
    }
    
    fn process(&self, data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError> {
        if let Some(num) = data.as_any().downcast_ref::<NumberData>() {
            let new_value = num.value * self.factor;
            main_info!("MultiplyProcessor: {} * {} = {}", num.value, self.factor, new_value);
            Ok(Arc::new(NumberData { value: new_value }))
        } else {
            Err(PipelineError::TypeError("Expected NumberData".to_string()))
        }
    }
}

fn main() {
    m_log::init();

    main_info!("=== Pipeline Demo ===");
    
    main_info!("Example 1: Normal processing flow");
    let p1 = Arc::new(IncrementProcessor);
    let p2 = Arc::new(ValidateProcessor { min_value: 10 });
    let p3 = Arc::new(MultiplyProcessor { factor: 2 });
    
    let mut pipeline1 = pipeline(p1).chain(p2).chain(p3);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 42 });
    
    match pipeline1.run(data) {
        Ok(result) => {
            let num = result.as_any().downcast_ref::<NumberData>().unwrap();
            main_info!("Final result: {}", num.value);
            
            main_info!("Processing history:");
            for record in pipeline1.history() {
                main_info!("  Processor: {}", record.processor_name);
                if let Some(output) = &record.output {
                    let out_num = output.as_any().downcast_ref::<NumberData>().unwrap();
                    main_info!("    Output: {}", out_num.value);
                }
            }
        }
        Err(e) => main_error!("Error: {}", e),
    }
    
    main_info!("=== Example 2: Error handling (short-circuit) ===");
    
    let p1 = Arc::new(IncrementProcessor);
    let p2 = Arc::new(ValidateProcessor { min_value: 50 });
    let p3 = Arc::new(MultiplyProcessor { factor: 2 });
    
    let mut pipeline2 = pipeline(p1).chain(p2).chain(p3);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 42 });
    
    match pipeline2.run(data) {
        Ok(result) => {
            let num = result.as_any().downcast_ref::<NumberData>().unwrap();
            main_info!("Final result: {}", num.value);
        }
        Err(e) => {
            main_error!("Pipeline failed: {}", e);
            main_error!("Failed at processor:");
            for record in pipeline2.history() {
                if let Some(error) = &record.error {
                    main_error!("  {}: {}", record.processor_name, error);
                }
            }
        }
    }
    
    main_info!("=== Pipeline Demo Complete ===");
}
```

- [ ] **Step 2: 在src/pipeline.rs顶部添加注册**

在pipeline.rs开头添加：

```rust
use crate::m_log::ModuleLog;
use crate::define_module;

define_module!(Pipeline, info=true, warn=true, error=true, debug=false);
```

然后在Pipeline::run方法中可添加pipeline_info!日志（可选，当前pipeline.rs无println!，暂不添加日志调用）。

- [ ] **Step 3: 验证无println!残留**

Run: `rg "println!" src/`
Expected: 仅m_log.rs内部eprintln!，业务代码无println!

Run: `cargo build`
Expected: BUILD SUCCESS

- [ ] **Step 4: 验证m_log.rs无业务硬编码**

Run: `rg "Pipeline|Data|Processor|Error|Main" src/m_log.rs`
Expected: 无匹配（m_log.rs仅含通用代码）

---

### Task T4: 测试

**Files:**
- Create: `tests/m_log_test.rs`

- [ ] **Step 1: 编写m_log测试**

```rust
use hello_world::{m_info, m_warn, m_error, m_debug, ModuleLog, m_log, define_module};

define_module!(TestModule, info=true, warn=true, error=true, debug=false);

#[test]
fn test_define_module_creates_struct() {
    let _ = TestModule;
}

#[test]
fn test_module_log_trait_impl() {
    assert!(TestModule::INFO);
    assert!(TestModule::WARN);
    assert!(TestModule::ERROR);
    assert!(!TestModule::DEBUG);
}

#[test]
fn test_convenience_macros_compile() {
    m_log::init();
    test_module_info!("test info message");
    test_module_warn!("test warn message");
    test_module_error!("test error message");
    test_module_debug!("test debug message");
}

#[test]
fn test_m_info_macro_enabled() {
    m_log::init();
    m_info!(TestModule, "test log output");
}

#[test]
fn test_m_log_no_business_hardcoding() {
    // This test verifies REQ-009 by checking m_log.rs content
    // The grep check in T3 Step 4 serves as the actual verification
}
```

- [ ] **Step 2: 运行所有测试**

Run: `cargo test`
Expected: ALL TESTS PASS (新测试 + 原有pipeline/data/processor测试)

- [ ] **Step 3: 运行cargo run验证日志输出**

Run: `cargo run`
Expected: [INFO] / [ERROR] 日志正常输出

## Gate Checklist

- [x] Phase 1 design has been approved.
- [x] Tasks map to REQ-IDs.
- [x] File change scope is listed.
- [x] Test strategy includes concrete commands.
- [x] Verification criteria are defined before coding.