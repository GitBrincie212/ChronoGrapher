pub mod error_handler;
pub mod metadata;

use std::any::{TypeId};
use std::sync::Arc;
use async_trait::async_trait;
use dashmap::DashMap;
use crate::task::{TaskContext, TaskError};

/// [`TaskExtension`] is a trait for defining an extension, essentially a new part for [`Task`] to
/// embody, it allows granular control over the task's lifecycle
///
/// # Object Safety
/// [`TaskExtension`] is object safe as seen in [`TaskExtenders`]'s source code
///
/// # See Also
/// - [`Task`]
#[async_trait]
pub trait TaskExtension {
    async fn on_task_start(&mut self, ctx: Arc<TaskContext<true>>) {}
    async fn on_task_end(&mut self, ctx: Arc<TaskContext<true>>, error: Option<TaskError>) {}
    async fn on_task_queued(&mut self) {}
    async fn on_task_dispatch(&mut self) {}
    async fn on_task_reschedule(&mut self) {}
}

#[derive(Default)]
pub struct TaskExtenders(pub(crate) DashMap<TypeId, Arc<dyn TaskExtension>>);