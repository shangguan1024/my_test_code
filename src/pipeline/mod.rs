use std::sync::Arc;
use crate::data::Data;
use crate::error::PipelineError;
use crate::Processor;
use m_log::define_module;

define_module!(PipelineLog, info=true, warn=true, error=true, debug=false);

mod runner;

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
        runner::run_pipeline(self, data)
    }
    
    pub fn history(&self) -> &[ProcessorRecord] {
        &self.history
    }

    pub fn dump(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Pipeline[{} processors]:", self.processors.len()));

        if self.history.is_empty() {
            if self.processors.is_empty() {
                lines.push("  (empty)".to_string());
            } else {
                lines.push("  (not yet executed)".to_string());
            }
        } else {
            for (i, record) in self.history.iter().enumerate() {
                let step = format!("  Step {}: {}", i + 1, record.processor_name);
                if let Some(error) = &record.error {
                    lines.push(format!("{} -> Err({})", step, error));
                } else if record.output.is_some() {
                    lines.push(format!("{} -> Ok (input=Data, output=Data)", step));
                } else {
                    lines.push(format!("{} -> (running)", step));
                }
            }
            for i in self.history.len()..self.processors.len() {
                lines.push(format!("  Step {}: {} -> [skipped: error short-circuited]",
                    i + 1, self.processors[i].name()));
            }
        }

        lines.join("\n")
    }
}
