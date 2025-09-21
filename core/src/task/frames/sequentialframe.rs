use crate::policy_match;
use crate::task::{
    ArcTaskEvent, TaskEndEvent, TaskError, TaskEvent, TaskEventEmitter, TaskFrame, TaskMetadata,
    TaskStartEvent,
};
use async_trait::async_trait;
use std::sync::Arc;

/// Defines a policy set for the [`SequentialTaskFrame`], these change the behavior of how the
/// parallel task frame operates, by default the parallel policy
/// [`SequentialTaskPolicy::RunSilenceFailures`] is used
pub enum SequentialTaskPolicy {
    /// Runs a task frame and its results do not affect the [`SequentialTaskFrame`]
    RunSilenceFailures,

    /// Runs a task frame, if it succeeds then it halts the other task frames and
    /// halts [`SequentialTaskFrame`], if not then it ignores the results and continues
    RunUntilSuccess,

    /// Runs a task frame, if it fails then it halts the other task frames and
    /// returns the error, halting [`SequentialTaskFrame`], if not then it ignores the results
    /// and continues
    RunUntilFailure,
}

/// Represents a **sequential task frame** which wraps multiple task frames to execute at the same time
/// in a sequential manner. This task frame type acts as a **composite node** within the task frame hierarchy,
/// facilitating a way to represent multiple task frames which have same timings but depend on each
/// previous task frame finishing. The order of execution is ordered, and thus why its sequential,
/// in the case where execution order does not matter and tasks do not require sequential execution,
/// it is advised to use [`ParallelTaskFrame`] as opposed to [`SequentialTaskFrame`]
///
/// # Events
/// For events, [`SequentialTaskFrame`] has 2 of them, these being `on_child_start` and `on_child_end`,
/// the former is for when a child task frame is about to start, the event hands out the target task
/// frame. For the latter, it is for when a child task frame ends, the event hands out the target
/// task frame and an optional error in case it fails
///
/// # Example
/// ```ignore
/// use std::sync::Arc;
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::{ExecutionTaskFrame, Task};
/// use chronographer_core::task::sequentialframe::SequentialTaskFrame;
///
/// let primary_frame = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Primary task frame fired...");
///         Ok(())
///     }
/// );
///
/// let secondary_frame = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Secondary task frame fired...");
///         Ok(())
///     }
/// );
///
/// let tertiary_frame = ExecutionTaskFrame::new(
///     |_metadata| async {
///         println!("Tertiary task frame fired...");
///         Err(())
///     }
/// );
///
/// let parallel_frame = SequentialTaskFrame::new(
///     vec![
///         Arc::new(primary_frame),
///         Arc::new(secondary_frame),
///         Arc::new(tertiary_frame)
///     ]
/// );
///
/// let task = Task::define(TaskScheduleInterval::from_secs_f64(1.25), parallel_frame);
///
/// CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
/// ```
pub struct SequentialTaskFrame {
    tasks: Vec<Arc<dyn TaskFrame>>,
    policy: SequentialTaskPolicy,
    on_start: TaskStartEvent,
    on_end: TaskEndEvent,
    pub on_child_start: ArcTaskEvent<Arc<dyn TaskFrame>>,
    pub on_child_end: ArcTaskEvent<(Arc<dyn TaskFrame>, Option<TaskError>)>,
}

impl SequentialTaskFrame {
    pub fn new(tasks: Vec<Arc<dyn TaskFrame>>) -> SequentialTaskFrame {
        Self::new_with(tasks, SequentialTaskPolicy::RunSilenceFailures)
    }

    pub fn new_with(
        tasks: Vec<Arc<dyn TaskFrame>>,
        sequential_policy: SequentialTaskPolicy,
    ) -> SequentialTaskFrame {
        Self {
            tasks,
            policy: sequential_policy,
            on_start: TaskEvent::new(),
            on_end: TaskEvent::new(),
            on_child_end: TaskEvent::new(),
            on_child_start: TaskEvent::new(),
        }
    }
}

#[async_trait]
impl TaskFrame for SequentialTaskFrame {
    async fn execute(
        &self,
        metadata: Arc<dyn TaskMetadata + Send + Sync>,
        emitter: Arc<TaskEventEmitter>,
    ) -> Result<(), TaskError> {
        for task in self.tasks.iter() {
            emitter
                .clone()
                .emit(metadata.clone(), self.on_child_start.clone(), task.clone())
                .await;
            let result = task.execute(metadata.clone(), emitter.clone()).await;
            policy_match!(metadata, emitter, task, self, result, SequentialTaskPolicy);
        }
        Ok(())
    }

    fn on_start(&self) -> TaskStartEvent {
        self.on_start.clone()
    }

    fn on_end(&self) -> TaskEndEvent {
        self.on_end.clone()
    }
}
