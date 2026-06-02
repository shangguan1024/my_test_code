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
    pub processors: Vec<Arc<dyn Processor>>,
    pub history: Vec<ProcessorRecord>,
}

pub fn pipeline(processor: Arc<dyn Processor>) -> Pipeline {
    Pipeline {
        processors: vec![processor],
        history: Vec::new(),
    }
}

impl Pipeline {
    pub fn chain(self, processor: Arc<dyn Processor>) -> Pipeline {
        let mut new_processors = self.processors.clone();
        new_processors.push(processor);
        Pipeline {
            processors: new_processors,
            history: Vec::new(),
        }
    }
    
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
    
    pub fn history(&self) -> &[ProcessorRecord] {
        &self.history
    }
}