mod error;
mod data;
mod processor;
mod pipeline;

pub use error::PipelineError;
pub use data::Data;
pub use processor::Processor;
pub use pipeline::{Pipeline, ProcessorRecord, pipeline};