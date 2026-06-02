use hello_world::Data;
use std::sync::Arc;
use std::any::Any;

struct TestData {
    value: i32,
}

impl Data for TestData {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn clone_data(&self) -> Arc<dyn Data> {
        Arc::new(TestData { value: self.value })
    }
}

#[test]
fn test_data_as_any() {
    let data = TestData { value: 42 };
    let any_ref = data.as_any();
    let downcast = any_ref.downcast_ref::<TestData>();
    assert!(downcast.is_some());
    assert_eq!(downcast.unwrap().value, 42);
}

#[test]
fn test_data_clone() {
    let data: Arc<dyn Data> = Arc::new(TestData { value: 42 });
    let cloned = data.clone_data();
    
    let original_any = data.as_any();
    let cloned_any = cloned.as_any();
    
    let original_value = original_any.downcast_ref::<TestData>().unwrap().value;
    let cloned_value = cloned_any.downcast_ref::<TestData>().unwrap().value;
    
    assert_eq!(original_value, cloned_value);
}

#[test]
fn test_data_trait_object() {
    let data: Arc<dyn Data> = Arc::new(TestData { value: 100 });
    let any_ref = data.as_any();
    assert!(any_ref.downcast_ref::<TestData>().is_some());
}