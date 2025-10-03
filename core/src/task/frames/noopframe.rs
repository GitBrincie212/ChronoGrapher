use crate::persistent_object::PersistentObject;
use crate::serialized_component::SerializedComponent;
use crate::task::{Arc, TaskContext, TaskError, TaskFrame};
use async_trait::async_trait;
use serde_json::json;

/// Represents a **no-operation task frame** that does nothing. This task frame type
/// acts as a **leaf node** within the task frame hierarchy. Its primary role is to
/// represent a hollow task frame that has no operations
///
/// This is useful for skipping execution of a task frame that is required, making it effectively
/// just a placeholder (that is why it is a no-operation task frame)
///
/// # Constructor(s)
/// When constructing a [`NoOperationTaskFrame`], one can use the default rust struct initialization,
/// or they can use [`NoOperationTaskFrame::default`] via [`Default`]
///
/// # Events
/// When it comes to events, [`NoOperationTaskFrame`], it has no local task frame events
///
/// # Trait Implementation(s)
/// It is obvious that the [`NoOperationTaskFrame`] implements [`TaskFrame`] since this
/// is a part of the default provided implementations, however, there are many others.
/// [`NoOperationTaskFrame`] also implements [`Default`]
///
/// # See Also
/// - [`TaskFrame`]
/// - [`NoOperationTaskFrame::default`]
#[derive(Default)]
pub struct NoOperationTaskFrame;

#[async_trait]
impl TaskFrame for NoOperationTaskFrame {
    async fn execute(&self, _ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        Ok(())
    }
}

#[async_trait]
impl PersistentObject for NoOperationTaskFrame {
    fn persistence_id() -> &'static str {
        "NoOperationTaskFrame$chronographer_core"
    }

    async fn store(&self) -> Result<SerializedComponent, TaskError> {
        Ok(SerializedComponent::new::<Self>(json!({})))
    }

    async fn retrieve(
        _component: SerializedComponent,
    ) -> Result<Self, TaskError> {
        Ok(NoOperationTaskFrame)
    }
}
