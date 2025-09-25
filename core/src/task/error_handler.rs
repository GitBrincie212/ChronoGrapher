use crate::task::{TaskError, TaskMetadata};
use async_trait::async_trait;
use std::sync::Arc;

#[allow(unused_imports)]
use crate::task::Task;

/// A [`TaskErrorContext`] which is sealed and exposes relevant information to [`TaskErrorHandler`]
///
/// # Constructor(s)
/// it cannot be created by outside parties and is handed by the scheduling strategy to control.
/// The error context contains the error and an exposed set of metadata in which the fields can
/// be accessed via [`TaskErrorContext::error`] and [`TaskErrorContext::metadata`] respectively
///
/// # Struct Field(s)
/// - **error** The error returned from [`Task`]'s [`TaskFrame`] (when it fails)
/// - **metadata** The task metadata associated with the [`Task`]
///
/// # See Also
/// - [`Task`]
/// - [`TaskError`]
/// - [`TaskMetadata`]
/// - [`TaskErrorHandler`]
pub struct TaskErrorContext {
    pub error: TaskError,
    pub metadata: Arc<TaskMetadata>,
}

/// [`TaskErrorHandler`] is a logic part that deals with any errors, it is invoked when a task has
/// returned an error. It is executed after the `on_end` [`TaskEvent`], the handler returns nothing
/// back and its only meant to handle errors (example a rollback mechanism, assuming a default value
/// for some state... etc.)
///
/// # Trait Implementation(s)
/// There are 2 noteworthy implementations to list for the [`TaskErrorHandler`], those being:
/// - [`SilentTaskErrorHandler`] Where it effectively is a no-op, and fully ignores the error,
///   this is the default option for [`Task`]. **HOWEVER**, just because its default doesn't mean
///   you should stick with it, for small demos it is fine, but for production environments.
///   It is more smart and wise to handle the errors gracefully yourself via implementing
///   the [`TaskErrorHandler`] trait
///
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
pub trait TaskErrorHandler: Send + Sync {
    async fn on_error(&self, ctx: Arc<TaskErrorContext>);
}

#[async_trait]
impl<E: TaskErrorHandler + ?Sized> TaskErrorHandler for Arc<E> {
    async fn on_error(&self, ctx: Arc<TaskErrorContext>) {
        self.as_ref().on_error(ctx).await;
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
    async fn on_error(&self, ctx: Arc<TaskErrorContext>) {
        panic!("{:?}", ctx.error);
    }
}

/// An implementation of [`TaskErrorHandler`] to silently ignore errors, in most cases, this
/// should not be used in production-grade applications as it makes debugging harder. However,
/// for small demos, or if all the possible errors do not contain any valuable information
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
pub struct SilentTaskErrorHandler;

#[async_trait]
impl TaskErrorHandler for SilentTaskErrorHandler {
    async fn on_error(&self, _ctx: Arc<TaskErrorContext>) {}
}
