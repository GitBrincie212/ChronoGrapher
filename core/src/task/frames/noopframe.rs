use crate::task::{Arc, TaskContext, TaskError, TaskFrame};
use async_trait::async_trait;

/// Represents a **no-operation task frame** that does nothing. This task frame type
/// acts as a **leaf node** within the task frame hierarchy. Its primary role is to
/// represent a hollow task frame that has no operations
///
/// This is useful for skipping execution of a task frame that is required, making it effectively
/// just a placeholder (that is why it is a no-operation task frame)
///
/// # Events
/// When it comes to events, [`NoOperationTaskFrame`], it has no local task frame events
pub struct NoOperationTaskFrame;

#[async_trait]
impl TaskFrame for NoOperationTaskFrame {
    async fn execute(&self, _ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        Ok(())
    }
}
