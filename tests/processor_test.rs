use hello_world::{Data, Processor, PipelineError};
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
    match result {
        Err(PipelineError::TypeError(msg)) => assert_eq!(msg, "Expected NumberData"),
        Err(_) => panic!("Expected TypeError"),
        Ok(_) => panic!("Expected error"),
    }
}

#[test]
fn test_processor_trait_object() {
    let processor: Arc<dyn Processor> = Arc::new(IncrementProcessor);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 10 });
    let result = processor.process(data);
    
    assert!(result.is_ok());
}