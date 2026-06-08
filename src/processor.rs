use std::sync::Arc;
use crate::data::Data;
use crate::error::PipelineError;
use m_log::define_module;

define_module!(ProcessorModule, info=true, warn=true, error=true, debug=false);

pub trait Processor: Send + Sync {
    fn name(&self) -> &str;
    fn process(&self, data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError>;
}