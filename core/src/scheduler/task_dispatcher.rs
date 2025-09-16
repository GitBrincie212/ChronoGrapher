pub mod default;

pub use default::*;

use crate::task::{Task, TaskEventEmitter};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast;

/// [`SchedulerTaskDispatcher`] is a trait for implementing a scheduler task dispatcher. It acts as
/// a central point for when a task wants to execute, on the default implementation, it routes the
/// task to a thread pool based on its priority. Allowing Chronolog to stay responsive even when
/// under heavy task workload
#[async_trait]
pub trait SchedulerTaskDispatcher: Send + Sync {
    async fn dispatch(
        self: Arc<Self>,
        sender: Arc<broadcast::Sender<(Arc<Task>, usize)>>,
        emitter: Arc<TaskEventEmitter>,
        task: Arc<Task>,
        idx: usize,
    );
}
