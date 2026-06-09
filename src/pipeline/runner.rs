use std::sync::Arc;
use crate::data::Data;
use crate::error::PipelineError;
use m_log::{m_info, m_error};
use super::{Pipeline, ProcessorRecord, PipelineLog};

pub fn run_pipeline(pipeline: &mut Pipeline, data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError> {
    if pipeline.processors.is_empty() {
        return Err(PipelineError::EmptyPipeline);
    }
    
    pipelinelog_info!("Pipeline starting with {} processors", pipeline.processors.len());
    pipeline.history.clear();
    let mut current_data = data;
    
    for processor in &pipeline.processors {
        let record = ProcessorRecord {
            processor_name: processor.name().to_string(),
            input: current_data.clone_data(),
            output: None,
            error: None,
        };
        pipeline.history.push(record);
        
        let result = processor.process(current_data);
        
        match result {
            Ok(output_data) => {
                if let Some(last_record) = pipeline.history.last_mut() {
                    last_record.output = Some(output_data.clone_data());
                }
                pipelinelog_info!("Step '{}' completed successfully", processor.name());
                current_data = output_data;
            }
            Err(error) => {
                if let Some(last_record) = pipeline.history.last_mut() {
                    last_record.error = Some(error.clone());
                }
                pipelinelog_error!("Step '{}' failed: {}", processor.name(), error);
                return Err(error);
            }
        }
    }
    
    pipelinelog_info!("Pipeline completed successfully");
    Ok(current_data)
}
