use std::sync::Arc;
use crate::data::Data;
use crate::error::PipelineError;

pub trait Processor: Send + Sync {
    fn name(&self) -> &str;
    fn process(&self, data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError>;
}