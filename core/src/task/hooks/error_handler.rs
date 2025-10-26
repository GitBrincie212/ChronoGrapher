use crate::task::{OnTaskEnd, TaskContext, TaskError, TaskHook, TaskHookEvent};
use async_trait::async_trait;
use std::ops::Deref;
use std::sync::Arc;

#[allow(unused_imports)]
use crate::task::Task;

/// [`TaskErrorHandler`] is a logic part that deals with any errors, it is invoked when a task has
/// returned an error. It is executed after the `on_end` [`TaskEvent`], the handler returns nothing
/// back and its only meant to handle errors (example a rollback mechanism, assuming a default value
/// for some state... etc.)
///
/// # Trait Implementation(s)
/// There are 2 noteworthy implementations to list for the [`TaskErrorHandler`], those being:
/// - [`PanicTaskErrorHandler`] Used mostly for debugging, where it panics if it encounters an error,
///   effectively stopping the execution of the program. It also **SHOULD NOT** be used in production
///   environments over a manual implementation of [`TaskErrorHandler`]
///
/// As mentioned, by default [`Task`] uses [`SilentTaskErrorHandler`]
///
/// # Object Safety
/// This trait is object safe to use, as seen in the source code of [`Task`] struct
///
/// # See Also
/// - [`TaskErrorHandler`]
/// - [`TaskEvent`]
/// - [`SilentTaskErrorHandler`]
#[async_trait]
pub trait TaskErrorHandler: Send + Sync + 'static {
    async fn on_error(&self, ctx: Arc<TaskContext>, error: TaskError);
}

#[async_trait]
impl<T: TaskErrorHandler> TaskHook<OnTaskEnd> for T {
    async fn on_event(
        &self,
        _event: OnTaskEnd,
        ctx: Arc<TaskContext>,
        payload: &<OnTaskEnd as TaskHookEvent>::Payload,
    ) {
        if let Some(err) = payload {
            self.on_error(ctx.clone(), err.clone()).await;
        }
    }
}

#[async_trait]
impl<E> TaskErrorHandler for E
where
    E: Deref + Send + Sync + 'static,
    E::Target: TaskErrorHandler,
{
    async fn on_error(&self, ctx: Arc<TaskContext>, error: TaskError) {
        self.deref().on_error(ctx, error).await;
    }
}

/// An implementation of [`TaskErrorHandler`] to panic when a [`Task`] fails, this should
/// not be used in production-grade applications, it is recommended to handle errors with
/// your own logic
///
/// # Constructor(s)
/// One can simply construct an instance by using rust's struct initialization or via
/// [`PanicTaskErrorHandler::default`] from [`Default`]
///
/// # Trait Implementation(s)
/// [`PanicTaskErrorHandler`] implements [`Default`], [`Clone`] and [`Copy`]
///
/// # See Also
/// - [`Task`]
/// - [`TaskErrorHandler`]
#[derive(Default, Clone, Copy)]
pub struct PanicTaskErrorHandler;

#[async_trait]
impl TaskErrorHandler for PanicTaskErrorHandler {
    async fn on_error(&self, _ctx: Arc<TaskContext>, error: TaskError) {
        panic!("{:?}", error);
    }
}
