use crate::task::frames::misc::{GroupedTaskFramesExecBehavior, GroupedTaskFramesQuitOnFailure};
use crate::task::{ArcTaskEvent, TaskContext, TaskError, TaskEvent, TaskFrame};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

#[allow(unused_imports)]
use crate::task::SequentialTaskFrame;

/// Represents a **parallel task frame** which wraps multiple [`TaskFrame`] to execute at the same time.
/// This task frame type acts as a **composite node** within the [`TaskFrame`] hierarchy, facilitating a
/// way to represent multiple [`TaskFrame`] which have same timings. This is much more optimized and accurate
/// than dispatching those task frames on the scheduler as independent tasks. The order of
/// execution is unordered, and thus one task may be executed sooner than another, in this case,
/// it is advised to use [`SequentialTaskFrame`] as opposed to [`ParallelTaskFrame`]
///
/// # Constructor(s)
/// When constructing a [`ParallelTask`], one can use either [`ParallelTask::new`] for no explicit
/// [`GroupedTaskFramesExecBehavior`] policy (convenience) or [`ParallelTask::new_with`]
/// if they do want to specify the [`GroupedTaskFramesExecBehavior`] policy as well
///
/// # Events
/// For events, [`ParallelTask`] has 2 of them, these being [`ParallelTask::on_child_start`] and
/// [`ParallelTask::on_child_end`],the former is for when a child task frame is about to start, the
/// event hands out the target [`TaskFrame`]. For the latter, it is for when a child task frame ends,
/// the event hands out the target task frame and an optional error in case it fails
///
/// # Trait Implementation(s)
/// It is obvious that the [`ParallelTask`] implements [`TaskFrame`] since this
/// is a part of the default provided implementations, however there are many others
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
///     |_ctx| async {
///         println!("Primary task frame fired...");
///         Ok(())
///     }
/// );
///
/// let secondary_frame = ExecutionTaskFrame::new(
///     |_ctx| async {
///         println!("Secondary task frame fired...");
///         Ok(())
///     }
/// );
///
/// let tertiary_frame = ExecutionTaskFrame::new(
///     |_ctx| async {
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
///
/// # See Also
/// - [`TaskFrame`]
/// - [`SequentialTaskFrame`]
/// - [`GroupedTaskFramesExecBehavior`]
//noinspection DuplicatedCode
pub struct ParallelTaskFrame {
    tasks: Vec<Arc<dyn TaskFrame>>,
    policy: Arc<dyn GroupedTaskFramesExecBehavior>,

    /// Event fired for when a child [`TaskFrame`] starts execution
    pub on_child_start: ArcTaskEvent<Arc<dyn TaskFrame>>,

    /// Event fired for when a child [`TaskFrame`] has ended execution
    pub on_child_end: ArcTaskEvent<(Arc<dyn TaskFrame>, Option<TaskError>)>,
}

impl ParallelTaskFrame {
    /// Creates / Constructs a new [`ParallelTaskFrame`] instance based on
    /// the child [`TaskFrame`] collection supplied. If one wishes to
    /// also supply their own [`GroupedTaskFramesExecBehavior`], then they can use
    /// [`ParallelTaskFrame::new_with`]
    ///
    /// # Argument(s)
    /// This method accepts one single argument, that is the collection of [`TaskFrame`] to wrap
    /// around the [`ParallelTaskFrame`] to execute concurrently
    ///
    /// # Returns
    /// A fully created [`ParallelTaskFrame`] with the wrapped ``tasks``
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`GroupedTaskFramesExecBehavior`]
    /// - [`ParallelTaskFrame::new_with`]
    /// - [`ParallelTaskFrame`]
    pub fn new(tasks: Vec<Arc<dyn TaskFrame>>) -> Self {
        Self::new_with(tasks, GroupedTaskFramesQuitOnFailure)
    }

    /// Creates / Constructs a new [`ParallelTaskFrame`] instance based on
    /// the child [`TaskFrame`] collection and a [`GroupedTaskFramesExecBehavior`] policy supplied.
    /// If one wishes to prefer the default [`GroupedTaskFramesExecBehavior`], then they can use
    /// [`ParallelTaskFrame::new`] for convenience
    ///
    /// # Argument(s)
    /// This method accepts two arguments, those being the collection of [`TaskFrame`] to wrap
    /// around the [`ParallelTaskFrame`] to execute concurrently and a [`GroupedTaskFramesExecBehavior`]
    /// policy
    ///
    /// # Returns
    /// A fully created [`ParallelTaskFrame`] with the wrapped ``tasks`` and a custom ``policy``
    /// as a [`GroupedTaskFramesExecBehavior`]
    /// ``
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`GroupedTaskFramesExecBehavior`]
    /// - [`ParallelTaskFrame::new`]
    /// - [`ParallelTaskFrame`]
    pub fn new_with(
        tasks: Vec<Arc<dyn TaskFrame>>,
        policy: impl GroupedTaskFramesExecBehavior + 'static,
    ) -> Self {
        Self {
            tasks,
            policy: Arc::new(policy),
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
            1 => self.tasks[0].execute(ctx.clone()).await?,
            _ => {
                std::thread::scope(|s| {
                    for frame in self.tasks.iter() {
                        let frame_clone = frame.clone();
                        let context_clone = ctx.clone();
                        let result_tx = result_tx.clone();
                        let child_start = self.on_child_start.clone();
                        let child_end = self.on_child_end.clone();
                        s.spawn(move || {
                            tokio::spawn(async move {
                                let restricted_context = context_clone.as_restricted();
                                context_clone
                                    .emitter
                                    .clone()
                                    .emit(
                                        restricted_context.clone(),
                                        child_start,
                                        frame_clone.clone(),
                                    )
                                    .await;
                                let result = frame_clone.execute(context_clone.clone()).await;
                                context_clone
                                    .emitter
                                    .clone()
                                    .emit(
                                        restricted_context.clone(),
                                        child_end, 
                                        (frame_clone, result.clone().err()),
                                    )
                                    .await;
                                let _ = result_tx.send(result);
                            })
                        });
                    }
                });
            }
        }

        drop(result_tx);

        while let Some(result) = result_rx.recv().await {
            let should_quit = self.policy.should_quit(result).await;
            if let Some(res) = should_quit {
                return res;
            }
        }

        Ok(())
    }
}
