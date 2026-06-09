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
//! use hello_world::{Data, Processor, Pipeline, pipeline, PipelineError};
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
//! # Ok::<(), PipelineError>(())
//! ```

mod error;
mod data;
mod processor;
mod pipeline;

pub use error::PipelineError;
pub use data::Data;
pub use processor::{Processor, ProcessorModule};
pub use pipeline::{Pipeline, ProcessorRecord, pipeline, PipelineModule};
pub use m_log;
pub use m_log::ModuleLog;
pub use m_log::define_module;
pub use m_log::{m_info, m_warn, m_error, m_debug};