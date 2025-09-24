use crate::policy_match;
use crate::task::{ArcTaskEvent, TaskContext, TaskError, TaskEvent, TaskFrame};
use async_trait::async_trait;
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
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::executionframe::ExecutionTaskFrame;
/// use chronographer_core::task::parallelframe::ParallelTask;
/// use chronographer_core::task::Task;
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
/// CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
/// ```
pub struct ParallelTaskFrame {
    tasks: Vec<Arc<dyn TaskFrame>>,
    policy: ParallelTaskPolicy,
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
            on_child_end: TaskEvent::new(),
            on_child_start: TaskEvent::new(),
        }
    }
}

#[async_trait]
impl TaskFrame for ParallelTaskFrame {
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        let (result_tx, mut result_rx) = mpsc::unbounded_channel();

        match self.tasks.len() {
            0 => {}
            1 => {
                self.tasks[0]
                    .execute(ctx)
                    .await?
            }
            _ => {
                std::thread::scope(|s| {
                    for frame in self.tasks.iter() {
                        let frame_clone = frame.clone();
                        let context_clone = ctx.clone();
                        let result_tx = result_tx.clone();
                        let child_start = self.on_child_start.clone();
                        s.spawn(move || {
                            tokio::spawn(async move {
                                context_clone.emitter
                                    .clone()
                                    .emit(context_clone.metadata.clone(), child_start, frame_clone.clone())
                                    .await;
                                let result = frame_clone
                                    .execute(context_clone)
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
            policy_match!(ctx.metadata, ctx.emitter, task, self, result, ParallelTaskPolicy);
        }

        Ok(())
    }
}
