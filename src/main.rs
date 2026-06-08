use hello_world::{Data, Processor, ProcessorModule, pipeline, PipelineError};
use m_log::{define_module, m_info, m_error};
use std::sync::Arc;
use std::any::Any;

define_module!(Main, info=true, warn=true, error=true, debug=false);

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
            m_info!(ProcessorModule, "IncrementProcessor: {} -> {}", num.value, num.value + 1);
            Ok(Arc::new(NumberData { value: num.value + 1 }))
        } else {
            Err(PipelineError::TypeError("Expected NumberData".to_string()))
        }
    }
}

struct ValidateProcessor {
    min_value: i32,
}

impl Processor for ValidateProcessor {
    fn name(&self) -> &str {
        "ValidateProcessor"
    }
    
    fn process(&self, data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError> {
        if let Some(num) = data.as_any().downcast_ref::<NumberData>() {
            m_info!(ProcessorModule, "ValidateProcessor: checking {} >= {}", num.value, self.min_value);
            if num.value >= self.min_value {
                Ok(data.clone_data())
            } else {
                Err(PipelineError::ValidationError(
                    format!("Value {} is less than minimum {}", num.value, self.min_value)
                ))
            }
        } else {
            Err(PipelineError::TypeError("Expected NumberData".to_string()))
        }
    }
}

struct MultiplyProcessor {
    factor: i32,
}

impl Processor for MultiplyProcessor {
    fn name(&self) -> &str {
        "MultiplyProcessor"
    }
    
    fn process(&self, data: Arc<dyn Data>) -> Result<Arc<dyn Data>, PipelineError> {
        if let Some(num) = data.as_any().downcast_ref::<NumberData>() {
            let new_value = num.value * self.factor;
            m_info!(ProcessorModule, "MultiplyProcessor: {} * {} = {}", num.value, self.factor, new_value);
            Ok(Arc::new(NumberData { value: new_value }))
        } else {
            Err(PipelineError::TypeError("Expected NumberData".to_string()))
        }
    }
}

fn main() {
    main_info!("=== Pipeline Demo ===");
    
    main_info!("Example 1: Normal processing flow");
    let p1 = Arc::new(IncrementProcessor);
    let p2 = Arc::new(ValidateProcessor { min_value: 10 });
    let p3 = Arc::new(MultiplyProcessor { factor: 2 });
    
    let mut pipeline1 = pipeline(p1).chain(p2).chain(p3);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 42 });
    
    match pipeline1.run(data) {
        Ok(result) => {
            let num = result.as_any().downcast_ref::<NumberData>().unwrap();
            main_info!("Final result: {}", num.value);
            
            main_info!("Processing history:");
            for record in pipeline1.history() {
                main_info!("  Processor: {}", record.processor_name);
                if let Some(output) = &record.output {
                    let out_num = output.as_any().downcast_ref::<NumberData>().unwrap();
                    main_info!("    Output: {}", out_num.value);
                }
            }
        }
        Err(e) => main_error!("Error: {}", e),
    }
    
    main_info!("=== Example 2: Error handling (short-circuit) ===");
    
    let p1 = Arc::new(IncrementProcessor);
    let p2 = Arc::new(ValidateProcessor { min_value: 50 });
    let p3 = Arc::new(MultiplyProcessor { factor: 2 });
    
    let mut pipeline2 = pipeline(p1).chain(p2).chain(p3);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 42 });
    
    match pipeline2.run(data) {
        Ok(result) => {
            let num = result.as_any().downcast_ref::<NumberData>().unwrap();
            main_info!("Final result: {}", num.value);
        }
        Err(e) => {
            main_error!("Pipeline failed: {}", e);
            main_error!("Failed at processor:");
            for record in pipeline2.history() {
                if let Some(error) = &record.error {
                    main_error!("  {}: {}", record.processor_name, error);
                }
            }
        }
    }
    
    main_info!("=== Pipeline Demo Complete ===");
}
