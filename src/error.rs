use std::fmt;

#[derive(Debug, Clone)]
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