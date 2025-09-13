use crate::policy_match;
use crate::task::{
    ArcTaskEvent, ExposedTaskMetadata, TaskEndEvent, TaskError, TaskEvent, TaskEventEmitter,
    TaskFrame, TaskStartEvent, sequentialframe::SequentialTaskFrame,
};
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Defines a policy set for the [`ParallelTaskFrame`], these change the behavior of how the
/// parallel task frame operates, by default the parallel policy
/// [`ParallelTaskPolicy::RunSilenceFailures`] is used
pub enum ParallelTaskPolicy {
    /// Runs a task frame and its results do not affect the [`ParallelTaskFrame`]
    RunSilenceFailures,

    /// Runs a task frame, if it succeeds then it halts the other task frames and
    /// returns / halts [`ParallelTaskFrame`], if not then it ignores the results and continues
    RunUntilSuccess,

    /// Runs a task frame, if it fails then it halts the other task frames and
    /// returns the error and halts [`ParallelTaskFrame`], if not then it ignores the results
    /// and continues
    RunUntilFailure,
}

/// Represents a **parallel task frame** which wraps multiple task frames to execute at the same time.
/// This task frame type acts as a **composite node** within the task frame hierarchy, facilitating a
/// way to represent multiple tasks which have same timings. This is much more optimized and accurate
/// than dispatching those task frames on the scheduler as independent tasks. The order of
/// execution is unordered, and thus one task may be executed sooner than another, in this case,
/// it is advised to use [`SequentialTaskFrame`] as opposed to [`ParallelTaskFrame`]
///
/// # Events
/// For events, [`ParallelTask`] has 2 of them, these being `on_child_start` and `on_child_end`,
/// the former is for when a child task frame is about to start, the event hands out the target
/// task frame. For the latter, it is for when a child task frame ends, the event hands out the
/// target task frame and an optional error in case it fails
///
/// # Example
/// ```ignore
/// use std::sync::Arc;
/// use chronolog_core::schedule::TaskScheduleInterval;
/// use chronolog_core::scheduler::{Scheduler, CHRONOLOG_SCHEDULER};
/// use chronolog_core::task::executionframe::ExecutionTaskFrame;
/// use chronolog_core::task::parallelframe::ParallelTask;
/// use chronolog_core::task::Task;
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
/// let parallel_frame = ParallelTask::new(
///     vec![
///         Arc::new(primary_frame),
///         Arc::new(secondary_frame),
///         Arc::new(tertiary_frame)
///     ]
/// );
///
/// let task = Task::define(TaskScheduleInterval::from_secs_f64(1.5), parallel_frame);
///
/// CHRONOLOG_SCHEDULER.register(task).await;
/// ```
pub struct ParallelTaskFrame {
    tasks: Vec<Arc<dyn TaskFrame>>,
    policy: ParallelTaskPolicy,
    on_start: TaskStartEvent,
    on_end: TaskEndEvent,
    pub on_child_start: ArcTaskEvent<Arc<dyn TaskFrame>>,
    pub on_child_end: ArcTaskEvent<(Arc<dyn TaskFrame>, Option<TaskError>)>,
}

impl ParallelTaskFrame {
    pub fn new(tasks: Vec<Arc<dyn TaskFrame>>) -> Self {
        Self::new_with(tasks, ParallelTaskPolicy::RunSilenceFailures)
    }

    pub fn new_with(
        tasks: Vec<Arc<dyn TaskFrame>>,
        parallel_task_policy: ParallelTaskPolicy,
    ) -> Self {
        Self {
            tasks,
            policy: parallel_task_policy,
            on_start: TaskEvent::new(),
            on_end: TaskEvent::new(),
            on_child_end: TaskEvent::new(),
            on_child_start: TaskEvent::new(),
        }
    }
}

#[async_trait]
impl TaskFrame for ParallelTaskFrame {
    async fn execute(
        &self,
        metadata: Arc<dyn ExposedTaskMetadata + Send + Sync>,
        emitter: Arc<TaskEventEmitter>,
    ) -> Result<(), TaskError> {
        let (result_tx, mut result_rx) = mpsc::unbounded_channel();

        match self.tasks.len() {
            0 => {}
            1 => {
                self.tasks[0]
                    .execute(metadata.clone(), emitter.clone())
                    .await?
            }
            _ => {
                std::thread::scope(|s| {
                    for frame in self.tasks.iter() {
                        let frame_clone = frame.clone();
                        let emitter_clone = emitter.clone();
                        let metadata_clone = metadata.clone();
                        let result_tx = result_tx.clone();
                        let child_start = self.on_child_start.clone();
                        s.spawn(move || {
                            tokio::spawn(async move {
                                emitter_clone
                                    .clone()
                                    .emit(metadata_clone.clone(), child_start, frame_clone.clone())
                                    .await;
                                let result = frame_clone
                                    .execute(metadata_clone, emitter_clone.clone())
                                    .await;
                                let _ = result_tx.send((frame_clone, result));
                            })
                        });
                    }
                });
            }
        }

        drop(result_tx);

        while let Some((task, result)) = result_rx.recv().await {
            policy_match!(metadata, emitter, task, self, result, ParallelTaskPolicy);
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
