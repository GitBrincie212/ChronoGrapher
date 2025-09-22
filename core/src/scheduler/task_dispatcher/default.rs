use std::fmt::Debug;
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::task::{Task, TaskEventEmitter, TaskPriority};
use async_trait::async_trait;
use multipool::pool::ThreadPool;
use multipool::pool::modes::PriorityWorkStealingMode;
use std::sync::Arc;
use tokio::sync::broadcast;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
#[builder(build_method(into = Arc<DefaultTaskDispatcher>))]
pub struct DefaultTaskDispatcherConfig {
    workers: usize,
}

impl From<DefaultTaskDispatcherConfig> for Arc<DefaultTaskDispatcher> {
    fn from(config: DefaultTaskDispatcherConfig) -> Arc<DefaultTaskDispatcher> {
        let pool = multipool::ThreadPoolBuilder::new()
            .set_work_stealing()
            .enable_priority()
            .num_threads(config.workers)
            .build();
        Arc::new(DefaultTaskDispatcher { pool })
    }
}

pub struct DefaultTaskDispatcher {
    pool: ThreadPool<PriorityWorkStealingMode>,
}

impl Debug for DefaultTaskDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultTaskDispatcher").finish()
    }
}

impl DefaultTaskDispatcher {
    pub fn default_configs() -> Arc<Self> {
        DefaultTaskDispatcher::builder().workers(16).build()
    }

    pub fn builder() -> DefaultTaskDispatcherConfigBuilder {
        DefaultTaskDispatcherConfig::builder()
    }
}

#[async_trait]
impl SchedulerTaskDispatcher for DefaultTaskDispatcher {
    async fn dispatch(
        self: Arc<Self>,
        sender: Arc<broadcast::Sender<(Arc<Task>, usize)>>,
        emitter: Arc<TaskEventEmitter>,
        task: Arc<Task>,
        idx: usize,
    ) {
        let target_priority = match task.priority {
            TaskPriority::CRITICAL => 0,
            TaskPriority::IMPORTANT => 100,
            TaskPriority::HIGH => 200,
            TaskPriority::MODERATE => 300,
            TaskPriority::LOW => 400,
        };

        let idx_clone = idx;
        self.pool.spawn_with_priority(
            move || {
                let idx_clone = idx_clone;
                let task_clone = task.clone();
                let sender_clone = sender.clone();
                let emitter_clone = emitter.clone();
                async move {
                    task_clone
                        .clone()
                        .schedule_strategy()
                        .handle(task_clone.clone(), emitter_clone)
                        .await;
                    sender_clone.send((task_clone, idx_clone)).unwrap();
                }
            },
            target_priority,
        );
    }
}
