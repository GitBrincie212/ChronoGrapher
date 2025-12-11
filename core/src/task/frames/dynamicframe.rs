use crate::task::{TaskContext, TaskError, TaskFrame};
use async_trait::async_trait;

/// Represents a **no-operation task frame** that does nothing. This task frame type
/// acts as a **leaf node** within the task frame hierarchy. Its primary role is to
/// represent a hollow task frame that has no operations
///
/// This is useful for skipping execution of a task frame that is required, making it effectively
/// just a placeholder (that is why it is a no-operation task frame)
///
/// # Constructor(s)
/// When constructing a [`DynamicTaskFrame`], one can use the default rust struct initialization,
/// or they can use [`DynamicTaskFrame::default`] via [`Default`]
///
/// # Events
/// When it comes to events, [`DynamicTaskFrame`], it has no local task frame events
///
/// # Trait Implementation(s)
/// It is obvious that the [`DynamicTaskFrame`] implements [`TaskFrame`] since this
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
/// - [`DynamicTaskFrame::default`]
pub struct DynamicTaskFrame<T>(T);

impl<T, F> DynamicTaskFrame<T>
where
    T: (Fn(&TaskContext) -> F) + Send + Sync + 'static,
    F: Future<Output = Result<(), TaskError>> + Send + 'static
{
    pub fn new(func: T) -> Self {
        Self(func)
    }
}

#[async_trait]
impl<T, F> TaskFrame for DynamicTaskFrame<T>
where
    T: (Fn(&TaskContext) -> F) + Send + Sync + 'static,
    F: Future<Output = Result<(), TaskError>> + Send + 'static
{
    async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
        self.0(ctx).await
    }
}

#[macro_export]
macro_rules! dynamic_taskframe {
    ($block: block) => {{
        DynamicTaskFrame::new(|ctx| async {
            $block
        })
    }};
}