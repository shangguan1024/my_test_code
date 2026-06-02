use std::any::Any;
use std::sync::Arc;

pub trait Data: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clone_data(&self) -> Arc<dyn Data>;
}