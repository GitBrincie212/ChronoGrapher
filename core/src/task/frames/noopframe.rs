use crate::persistence::{PersistenceContext, PersistenceObject};
use crate::task::{Arc, TaskContext, TaskError, TaskFrame};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
/// is a part of the default provided implementations, but also:
/// - [`Debug`] (just prints the name, nothing more, nothing less)
/// - [`Clone`]
/// - [`Copy`]
/// - [`Default`]
/// - [`PersistenceObject`]
/// - [`Serialize`]
/// - [`Deserialize`]
///
/// # See Also
/// - [`TaskFrame`]
/// - [`NoOperationTaskFrame::default`]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct NoOperationTaskFrame;

#[async_trait]
impl TaskFrame for NoOperationTaskFrame {
    async fn execute(&self, _ctx: &TaskContext) -> Result<(), TaskError> {
        Ok(())
    }
}

impl PersistenceObject for NoOperationTaskFrame {
    const PERSISTENCE_ID: &'static str =
        "chronographer::NoOperationTaskFrame#25ce069e-d1be-47aa-a68e-f5e659ffdb27";

    fn inject_context(&self, _ctx: &PersistenceContext) {}
}
