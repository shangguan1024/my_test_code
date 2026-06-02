# PipelineDesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现函数式Pipeline系统，支持链式API、可扩展Processor、Arc<dyn Data>数据传递、错误短路终止、中间结果访问。

**Architecture:** Trait object composition + Builder pattern。Data/Processor为trait接口，Pipeline管理processor链并执行数据流，记录处理历史。

**Tech Stack:** Rust标准库（Arc, Any, Result），无外部依赖。

---

## File Structure

**新建文件：**
- `src/error.rs` - PipelineError enum定义，Display/Error trait实现
- `src/data.rs` - Data trait定义，trait object safety支持
- `src/processor.rs` - Processor trait定义，包含name()和process()方法
- `src/pipeline.rs` - Pipeline/ProcessorRecord结构，pipeline()/chain()/run()/history()实现
- `src/lib.rs` - 导出所有公共API，模块级文档
- `tests/data_test.rs` - Data trait测试
- `tests/processor_test.rs` - Processor trait测试
- `tests/pipeline_test.rs` - Pipeline集成测试

**修改文件：**
- `src/main.rs` - 示例演示（完整用例展示）

---

## Task 1: error.rs - 错误类型定义

**Files:**
- Create: `src/error.rs`

**说明：** error.rs是基础模块，无独立测试（测试覆盖在processor/pipeline测试中）。

- [ ] **Step 1: 创建error.rs文件**

创建文件并实现PipelineError enum：

```rust
use std::fmt;

pub enum PipelineError {
    TypeError(String),
    ValidationError(String),
    ProcessingError(String),
    EmptyPipeline,
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineError::TypeError(msg) => write!(f, "Type error: {}", msg),
            PipelineError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            PipelineError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            PipelineError::EmptyPipeline => write!(f, "Pipeline has no processors"),
        }
    }
}

impl std::error::Error for PipelineError {}
```

- [ ] **Step 2: 验证编译**

运行: `cargo build --lib`
预期: 编译成功，无错误

- [ ] **Step 3: Commit**

```bash
git add src/error.rs
git commit -m "feat: add PipelineError enum with Display and Error traits"
```

---

## Task 2: data.rs - Data trait定义与测试

**Files:**
- Create: `src/data.rs`
- Create: `tests/data_test.rs`

- [ ] **Step 1: 创建data.rs文件**

创建文件并实现Data trait：

```rust
use std::any::Any;
use std::sync::Arc;

pub trait Data: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clone_data(&self) -> Arc<dyn Data>;
}
```

- [ ] **Step 2: 编写测试 - 测试Data trait实现**

创建 `tests/data_test.rs`：

```rust
use pipeline::Data;
use std::sync::Arc;
use std::any::Any;

struct TestData {
    value: i32,
}

impl Data for TestData {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn clone_data(&self) -> Arc<dyn Data> {
        Arc::new(TestData { value: self.value })
    }
}

#[test]
fn test_data_as_any() {
    let data = TestData { value: 42 };
    let any_ref = data.as_any();
    let downcast = any_ref.downcast_ref::<TestData>();
    assert!(downcast.is_some());
    assert_eq!(downcast.unwrap().value, 42);
}

#[test]
fn test_data_clone() {
    let data: Arc<dyn Data> = Arc::new(TestData { value: 42 });
    let cloned = data.clone_data();
    
    let original_any = data.as_any();
    let cloned_any = cloned.as_any();
    
    let original_value = original_any.downcast_ref::<TestData>().unwrap().value;
    let cloned_value = cloned_any.downcast_ref::<TestData>().unwrap().value;
    
    assert_eq!(original_value, cloned_value);
}

#[test]
fn test_data_trait_object() {
    let data: Arc<dyn Data> = Arc::new(TestData { value: 100 });
    let any_ref = data.as_any();
    assert!(any_ref.downcast_ref::<TestData>().is_some());
}
```

- [ ] **Step 3: 运行测试验证失败**

运行: `cargo test --test data_test`
预期: FAIL - "pipeline crate not found"（lib.rs还未创建）

- [ ] **Step 4: 创建lib.rs临时导出**

创建 `src/lib.rs` 用于测试：

```rust
mod error;
mod data;

pub use error::PipelineError;
pub use data::Data;
```

- [ ] **Step 5: 运行测试验证失败**

运行: `cargo test --test data_test`
预期: FAIL - "TestData not found"（测试需要TestData定义）

- [ ] **Step 6: 修复测试**

修改 `tests/data_test.rs`，确保TestData定义在测试文件中（已包含）。

- [ ] **Step 7: 运行测试验证通过**

运行: `cargo test --test data_test`
预期: PASS - 3 tests passed

- [ ] **Step 8: Commit**

```bash
git add src/data.rs tests/data_test.rs src/lib.rs
git commit -m "feat: add Data trait with as_any and clone_data methods, add tests"
```

---

## Task 3: processor.rs - Processor trait定义与测试

**Files:**
- Create: `src/processor.rs`
- Create: `tests/processor_test.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: 创建processor.rs文件**

创建文件并实现Processor trait：

```rust
use std::sync::Arc;
use crate::data::Data;
use crate::error::PipelineError;

pub trait Processor: Send + Sync {
    fn name(&self) -> &str;
    fn process(&self, data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError>;
}
```

- [ ] **Step 2: 更新lib.rs导出**

修改 `src/lib.rs`：

```rust
mod error;
mod data;
mod processor;

pub use error::PipelineError;
pub use data::Data;
pub use processor::Processor;
```

- [ ] **Step 3: 编写测试 - 测试Processor trait实现**

创建 `tests/processor_test.rs`：

```rust
use pipeline::{Data, Processor, PipelineError};
use std::sync::Arc;
use std::any::Any;

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
            Ok(Arc::new(NumberData { value: num.value + 1 }))
        } else {
            Err(PipelineError::TypeError("Expected NumberData".to_string()))
        }
    }
}

#[test]
fn test_processor_name() {
    let processor = IncrementProcessor;
    assert_eq!(processor.name(), "IncrementProcessor");
}

#[test]
fn test_processor_process_success() {
    let processor = IncrementProcessor;
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 42 });
    let result = processor.process(data);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    let num = output.as_any().downcast_ref::<NumberData>().unwrap();
    assert_eq!(num.value, 43);
}

#[test]
fn test_processor_process_type_error() {
    let processor = IncrementProcessor;
    struct WrongData { text: String }
    
    impl Data for WrongData {
        fn as_any(&self) -> &dyn Any { self }
        fn clone_data(&self) -> Arc<dyn Data> {
            Arc::new(WrongData { text: self.text.clone() })
        }
    }
    
    let data: Arc<dyn Data> = Arc::new(WrongData { text: "hello".to_string() });
    let result = processor.process(data);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        PipelineError::TypeError(msg) => assert_eq!(msg, "Expected NumberData"),
        _ => panic!("Expected TypeError"),
    }
}

#[test]
fn test_processor_trait_object() {
    let processor: Arc<dyn Processor> = Arc::new(IncrementProcessor);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 10 });
    let result = processor.process(data);
    
    assert!(result.is_ok());
}
```

- [ ] **Step 4: 运行测试验证失败**

运行: `cargo test --test processor_test`
预期: FAIL - "processor module not found"（需更新lib.rs）

- [ ] **Step 5: 运行测试验证通过**

运行: `cargo test --test processor_test`
预期: PASS - 4 tests passed（lib.rs已更新）

- [ ] **Step 6: Commit**

```bash
git add src/processor.rs tests/processor_test.rs src/lib.rs
git commit -m "feat: add Processor trait with name and process methods, add tests"
```

---

## Task 4: pipeline.rs - Pipeline核心实现与测试

**Files:**
- Create: `src/pipeline.rs`
- Create: `tests/pipeline_test.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: 创建pipeline.rs文件 - ProcessorRecord结构**

创建 `src/pipeline.rs`，先定义ProcessorRecord：

```rust
use std::sync::Arc;
use crate::data::Data;
use crate::processor::Processor;
use crate::error::PipelineError;

pub struct ProcessorRecord {
    pub processor_name: String,
    pub input: Arc<dyn Data>,
    pub output: Option<Arc<dyn Data>>,
    pub error: Option<PipelineError>,
}

pub struct Pipeline {
    processors: Vec<Arc<dyn Processor>>,
    pub history: Vec<ProcessorRecord>,
}
```

- [ ] **Step 2: 实现pipeline()工厂函数**

继续在 `src/pipeline.rs` 添加：

```rust
pub fn pipeline(processor: Arc<dyn Processor>) -> Pipeline {
    Pipeline {
        processors: vec![processor],
        history: Vec::new(),
    }
}
```

- [ ] **Step 3: 实现chain()方法**

继续在 `src/pipeline.rs` 添加：

```rust
impl Pipeline {
    pub fn chain(self, processor: Arc<dyn Processor>) -> Pipeline {
        let mut new_processors = self.processors.clone();
        new_processors.push(processor);
        Pipeline {
            processors: new_processors,
            history: Vec::new(),
        }
    }
}
```

- [ ] **Step 4: 实现run()方法**

继续在 `src/pipeline.rs` 添加：

```rust
impl Pipeline {
    pub fn run(&mut self, data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError> {
        if self.processors.is_empty() {
            return Err(PipelineError::EmptyPipeline);
        }
        
        self.history.clear();
        let mut current_data = data;
        
        for processor in &self.processors {
            let record = ProcessorRecord {
                processor_name: processor.name().to_string(),
                input: current_data.clone_data(),
                output: None,
                error: None,
            };
            self.history.push(record);
            
            let result = processor.process(current_data);
            
            match result {
                Ok(output_data) => {
                    if let Some(last_record) = self.history.last_mut() {
                        last_record.output = Some(output_data.clone_data());
                    }
                    current_data = output_data;
                }
                Err(error) => {
                    if let Some(last_record) = self.history.last_mut() {
                        last_record.error = Some(error.clone());
                    }
                    return Err(error);
                }
            }
        }
        
        Ok(current_data)
    }
}
```

- [ ] **Step 5: 实现history()方法**

继续在 `src/pipeline.rs` 添加：

```rust
impl Pipeline {
    pub fn history(&self) -> &[ProcessorRecord] {
        &self.history
    }
}
```

- [ ] **Step 6: 更新lib.rs导出**

修改 `src/lib.rs`：

```rust
mod error;
mod data;
mod processor;
mod pipeline;

pub use error::PipelineError;
pub use data::Data;
pub use processor::Processor;
pub use pipeline::{Pipeline, ProcessorRecord, pipeline};
```

- [ ] **Step 7: 编写集成测试 - 创建pipeline_test.rs**

创建 `tests/pipeline_test.rs`：

```rust
use pipeline::{Data, Processor, Pipeline, PipelineError, pipeline};
use std::sync::Arc;
use std::any::Any;

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
            if num.value >= self.min_value {
                Ok(data.clone_data())
            } else {
                Err(PipelineError::ValidationError(
                    format!("Value must be >= {}, got {}", self.min_value, num.value)
                ))
            }
        } else {
            Err(PipelineError::TypeError("Expected NumberData".to_string()))
        }
    }
}

#[test]
fn test_pipeline_creation() {
    let p1 = Arc::new(IncrementProcessor);
    let pipeline = pipeline(p1);
    
    assert_eq!(pipeline.processors.len(), 1);
    assert_eq!(pipeline.history.len(), 0);
}

#[test]
fn test_pipeline_chain() {
    let p1 = Arc::new(IncrementProcessor);
    let p2 = Arc::new(ValidateProcessor { min_value: 10 });
    
    let pipeline = pipeline(p1).chain(p2);
    
    assert_eq!(pipeline.processors.len(), 2);
}

#[test]
fn test_pipeline_run_success() {
    let p1 = Arc::new(IncrementProcessor);
    let p2 = Arc::new(ValidateProcessor { min_value: 10 });
    
    let mut pipeline = pipeline(p1).chain(p2);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 42 });
    
    let result = pipeline.run(data);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    let num = output.as_any().downcast_ref::<NumberData>().unwrap();
    assert_eq!(num.value, 43);  // 42 + 1
}

#[test]
fn test_pipeline_run_error_short_circuit() {
    let p1 = Arc::new(IncrementProcessor);
    let p2 = Arc::new(ValidateProcessor { min_value: 50 });
    
    let mut pipeline = pipeline(p1).chain(p2);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 42 });
    
    let result = pipeline.run(data);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        PipelineError::ValidationError(msg) => {
            assert!(msg.contains("Value must be >= 50"));
        }
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_pipeline_history() {
    let p1 = Arc::new(IncrementProcessor);
    let p2 = Arc::new(ValidateProcessor { min_value: 10 });
    
    let mut pipeline = pipeline(p1).chain(p2);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 42 });
    
    pipeline.run(data).unwrap();
    
    let history = pipeline.history();
    assert_eq!(history.len(), 2);
    
    assert_eq!(history[0].processor_name, "IncrementProcessor");
    assert!(history[0].output.is_some());
    let output1 = history[0].output.as_ref().unwrap();
    let num1 = output1.as_any().downcast_ref::<NumberData>().unwrap();
    assert_eq!(num1.value, 43);
    
    assert_eq!(history[1].processor_name, "ValidateProcessor");
    assert!(history[1].output.is_some());
}

#[test]
fn test_pipeline_history_on_error() {
    let p1 = Arc::new(IncrementProcessor);
    let p2 = Arc::new(ValidateProcessor { min_value: 50 });
    
    let mut pipeline = pipeline(p1).chain(p2);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 42 });
    
    let _ = pipeline.run(data);
    
    let history = pipeline.history();
    assert_eq!(history.len(), 2);
    
    assert_eq!(history[0].processor_name, "IncrementProcessor");
    assert!(history[0].output.is_some());
    
    assert_eq!(history[1].processor_name, "ValidateProcessor");
    assert!(history[1].error.is_some());
}

#[test]
fn test_pipeline_empty() {
    let mut pipeline = Pipeline {
        processors: vec![],
        history: vec![],
    };
    
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 42 });
    let result = pipeline.run(data);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        PipelineError::EmptyPipeline => (),
        _ => panic!("Expected EmptyPipeline error"),
    }
}
```

- [ ] **Step 8: 运行测试验证失败**

运行: `cargo test --test pipeline_test`
预期: FAIL - 需要调整代码（Pipeline结构体需要public访问）

- [ ] **Step 9: 修复pipeline.rs - 使processors字段可访问**

修改 `src/pipeline.rs`：

```rust
pub struct Pipeline {
    pub processors: Vec<Arc<dyn Processor>>,  // 改为pub
    pub history: Vec<ProcessorRecord>,
}
```

- [ ] **Step 10: 运行测试验证通过**

运行: `cargo test --test pipeline_test`
预期: PASS - 7 tests passed

- [ ] **Step 11: Commit**

```bash
git add src/pipeline.rs tests/pipeline_test.rs src/lib.rs
git commit -m "feat: implement Pipeline with chain/run/history, add comprehensive tests"
```

---

## Task 5: 完善lib.rs - 添加文档和完整导出

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: 完善lib.rs文档和导出**

修改 `src/lib.rs`：

```rust
//! Pipeline库 - 函数式数据处理流
//! 
//! 提供可扩展的processor链式调用系统，支持中间结果访问和错误短路终止。
//! 
//! # 核心概念
//! 
//! - [`Data`] - 数据trait，所有processor之间传递的数据类型基础
//! - [`Processor`] - 处理单元trait，用户实现此trait定义处理逻辑
//! - [`Pipeline`] - 处理链管理器，支持链式组装和执行
//! 
//! # 快速示例
//! 
//! ```rust
//! use pipeline::{Data, Processor, Pipeline, pipeline, PipelineError};
//! use std::sync::Arc;
//! use std::any::Any;
//! 
//! // 定义数据类型
//! struct MyData { value: i32 }
//! 
//! impl Data for MyData {
//!     fn as_any(&self) -> &dyn Any { self }
//!     fn clone_data(&self) -> Arc<dyn Data> {
//!         Arc::new(MyData { value: self.value })
//!     }
//! }
//! 
//! // 定义processor
//! struct MyProcessor;
//! 
//! impl Processor for MyProcessor {
//!     fn name(&self) -> &str { "MyProcessor" }
//!     fn process(&self, data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError> {
//!         // 处理逻辑
//!         Ok(data.clone_data())
//!     }
//! }
//! 
//! // 使用Pipeline
//! let p = Arc::new(MyProcessor);
//! let mut pipeline = pipeline(p);
//! let data = Arc::new(MyData { value: 42 });
//! let result = pipeline.run(data)?;
//! ```

mod error;
mod data;
mod processor;
mod pipeline;

pub use error::PipelineError;
pub use data::Data;
pub use processor::Processor;
pub use pipeline::{Pipeline, ProcessorRecord, pipeline};
```

- [ ] **Step 2: 验证文档生成**

运行: `cargo doc --lib`
预期: 文档生成成功，无警告

- [ ] **Step 3: 运行所有测试**

运行: `cargo test`
预期: 所有测试通过

- [ ] **Step 4: Commit**

```bash
git add src/lib.rs
git commit -m "docs: add comprehensive library documentation and usage examples"
```

---

## Task 6: main.rs - 示例演示

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 实现main.rs示例**

修改 `src/main.rs`：

```rust
use pipeline::{Data, Processor, Pipeline, pipeline, PipelineError};
use std::sync::Arc;
use std::any::Any;

// 定义数据类型
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

// 定义processors
struct IncrementProcessor;

impl Processor for IncrementProcessor {
    fn name(&self) -> &str {
        "IncrementProcessor"
    }
    
    fn process(&self, data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError> {
        if let Some(num) = data.as_any().downcast_ref::<NumberData>() {
            println!("IncrementProcessor: {} -> {}", num.value, num.value + 1);
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
            println!("ValidateProcessor: checking {} >= {}", num.value, self.min_value);
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
            println!("MultiplyProcessor: {} * {} = {}", num.value, self.factor, new_value);
            Ok(Arc::new(NumberData { value: new_value }))
        } else {
            Err(PipelineError::TypeError("Expected NumberData".to_string()))
        }
    }
}

fn main() {
    println!("=== Pipeline Demo ===\n");
    
    // 示例1: 正常处理流程
    println!("Example 1: Normal processing flow");
    let p1 = Arc::new(IncrementProcessor);
    let p2 = Arc::new(ValidateProcessor { min_value: 10 });
    let p3 = Arc::new(MultiplyProcessor { factor: 2 });
    
    let mut pipeline = pipeline(p1).chain(p2).chain(p3);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 42 });
    
    match pipeline.run(data) {
        Ok(result) => {
            let num = result.as_any().downcast_ref::<NumberData>().unwrap();
            println!("\n✅ Final result: {}\n", num.value);
            
            println!("Processing history:");
            for record in pipeline.history() {
                println!("  - Processor: {}", record.processor_name);
                if let Some(output) = &record.output {
                    let out_num = output.as_any().downcast_ref::<NumberData>().unwrap();
                    println!("    Output: {}", out_num.value);
                }
            }
        }
        Err(e) => println!("\n❌ Error: {}\n", e),
    }
    
    println!("\n=== Example 2: Error handling (short-circuit) ===\n");
    
    // 示例2: 错误短路终止
    let p1 = Arc::new(IncrementProcessor);
    let p2 = Arc::new(ValidateProcessor { min_value: 50 });  // 较高的最小值
    let p3 = Arc::new(MultiplyProcessor { factor: 2 });
    
    let mut pipeline = pipeline(p1).chain(p2).chain(p3);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 42 });
    
    match pipeline.run(data) {
        Ok(result) => {
            let num = result.as_any().downcast_ref::<NumberData>().unwrap();
            println!("✅ Final result: {}", num.value);
        }
        Err(e) => {
            println!("❌ Pipeline failed: {}", e);
            println!("\nFailed at processor:");
            for record in pipeline.history() {
                if let Some(error) = &record.error {
                    println!("  - {}: {}", record.processor_name, error);
                }
            }
        }
    }
    
    println!("\n=== Pipeline Demo Complete ===");
}
```

- [ ] **Step 2: 运行示例验证**

运行: `cargo run`
预期: 输出包含两个示例的处理流程和结果

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: add comprehensive main.rs demo showing normal flow and error handling"
```

---

## Task 7: 最终验证和清理

**Files:**
- All files

- [ ] **Step 1: 运行所有测试**

运行: `cargo test --all`
预期: 所有测试通过

- [ ] **Step 2: 运行示例**

运行: `cargo run`
预期: 示例正常执行，输出符合预期

- [ ] **Step 3: 检查代码格式**

运行: `cargo fmt --check`
预期: 无格式问题

- [ ] **Step 4: 运行clippy检查**

运行: `cargo clippy --all-targets`
预期: 无clippy警告（或仅有可接受的警告）

- [ ] **Step 5: 生成文档**

运行: `cargo doc --no-deps`
预期: 文档生成成功

- [ ] **Step 6: 最终commit**

```bash
git add -A
git commit -m "chore: final verification and cleanup for Pipeline feature"
```

---

## Self-Review Checklist

**1. Spec Coverage:**
- ✅ REQ-001: Processor trait定义 → Task 3
- ✅ REQ-002: 函数式链式API → Task 4 (chain方法)
- ✅ REQ-003: Arc<dyn Data>传递 → Task 2, 3, 4
- ✅ REQ-004: 错误短路终止 → Task 4 (run方法error handling)
- ✅ REQ-005: 中间结果访问 → Task 4 (history方法)
- ✅ REQ-006: 动态组装 → Task 4 (chain可运行时调用)

**2. Placeholder Scan:**
- ✅ 无TBD/TODO
- ✅ 所有代码完整
- ✅ 所有命令明确

**3. Type Consistency:**
- ✅ Processor.name() -> &str (一致)
- ✅ Processor.process() -> Result<Arc<dyn Data>, PipelineError> (一致)
- ✅ PipelineError类型定义一致
- ✅ Data trait方法签名一致

---

## Execution Handoff

Plan complete and saved to `docs/features/PipelineDesign/implementation-plan.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**