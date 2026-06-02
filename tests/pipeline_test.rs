use hello_world::{Data, Processor, PipelineError, Pipeline, pipeline};
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

struct DoubleProcessor;

impl Processor for DoubleProcessor {
    fn name(&self) -> &str {
        "DoubleProcessor"
    }
    
    fn process(&self, data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError> {
        if let Some(num) = data.as_any().downcast_ref::<NumberData>() {
            Ok(Arc::new(NumberData { value: num.value * 2 }))
        } else {
            Err(PipelineError::TypeError("Expected NumberData".to_string()))
        }
    }
}

struct ErrorProcessor;

impl Processor for ErrorProcessor {
    fn name(&self) -> &str {
        "ErrorProcessor"
    }
    
    fn process(&self, _data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError> {
        Err(PipelineError::ProcessingError("Intentional error".to_string()))
    }
}

#[test]
fn test_pipeline_creation() {
    let processor: Arc<dyn Processor> = Arc::new(IncrementProcessor);
    let pipe = pipeline(processor);
    
    assert_eq!(pipe.processors.len(), 1);
    assert_eq!(pipe.history.len(), 0);
}

#[test]
fn test_pipeline_chain() {
    let processor1: Arc<dyn Processor> = Arc::new(IncrementProcessor);
    let processor2: Arc<dyn Processor> = Arc::new(DoubleProcessor);
    
    let pipe = pipeline(processor1).chain(processor2);
    
    assert_eq!(pipe.processors.len(), 2);
    assert_eq!(pipe.history.len(), 0);
}

#[test]
fn test_pipeline_run_success() {
    let processor: Arc<dyn Processor> = Arc::new(IncrementProcessor);
    let mut pipe = pipeline(processor);
    
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 10 });
    let result = pipe.run(data);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    let num = output.as_any().downcast_ref::<NumberData>().unwrap();
    assert_eq!(num.value, 11);
}

#[test]
fn test_pipeline_run_error_short_circuit() {
    let processor1: Arc<dyn Processor> = Arc::new(IncrementProcessor);
    let processor2: Arc<dyn Processor> = Arc::new(ErrorProcessor);
    let processor3: Arc<dyn Processor> = Arc::new(DoubleProcessor);
    
    let mut pipe = pipeline(processor1)
        .chain(processor2)
        .chain(processor3);
    
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 10 });
    let result = pipe.run(data);
    
    assert!(result.is_err());
    match result {
        Err(PipelineError::ProcessingError(msg)) => assert_eq!(msg, "Intentional error"),
        _ => panic!("Expected ProcessingError"),
    }
}

#[test]
fn test_pipeline_history() {
    let processor1: Arc<dyn Processor> = Arc::new(IncrementProcessor);
    let processor2: Arc<dyn Processor> = Arc::new(DoubleProcessor);
    
    let mut pipe = pipeline(processor1).chain(processor2);
    
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 5 });
    let result = pipe.run(data);
    
    assert!(result.is_ok());
    
    let history = pipe.history();
    assert_eq!(history.len(), 2);
    
    assert_eq!(history[0].processor_name, "IncrementProcessor");
    let input0 = history[0].input.as_any().downcast_ref::<NumberData>().unwrap();
    assert_eq!(input0.value, 5);
    let output0 = history[0].output.as_ref().unwrap();
    let output0_val = output0.as_any().downcast_ref::<NumberData>().unwrap();
    assert_eq!(output0_val.value, 6);
    
    assert_eq!(history[1].processor_name, "DoubleProcessor");
    let input1 = history[1].input.as_any().downcast_ref::<NumberData>().unwrap();
    assert_eq!(input1.value, 6);
    let output1 = history[1].output.as_ref().unwrap();
    let output1_val = output1.as_any().downcast_ref::<NumberData>().unwrap();
    assert_eq!(output1_val.value, 12);
}

#[test]
fn test_pipeline_history_on_error() {
    let processor1: Arc<dyn Processor> = Arc::new(IncrementProcessor);
    let processor2: Arc<dyn Processor> = Arc::new(ErrorProcessor);
    let processor3: Arc<dyn Processor> = Arc::new(DoubleProcessor);
    
    let mut pipe = pipeline(processor1)
        .chain(processor2)
        .chain(processor3);
    
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 10 });
    let result = pipe.run(data);
    
    assert!(result.is_err());
    
    let history = pipe.history();
    assert_eq!(history.len(), 2);
    
    assert_eq!(history[0].processor_name, "IncrementProcessor");
    assert!(history[0].output.is_some());
    assert!(history[0].error.is_none());
    
    assert_eq!(history[1].processor_name, "ErrorProcessor");
    assert!(history[1].output.is_none());
    assert!(history[1].error.is_some());
}

#[test]
fn test_pipeline_empty() {
    let mut pipe = Pipeline {
        processors: Vec::new(),
        history: Vec::new(),
    };
    
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 10 });
    let result = pipe.run(data);
    
    assert!(result.is_err());
    match result {
        Err(PipelineError::EmptyPipeline) => (),
        _ => panic!("Expected EmptyPipeline error"),
    }
}