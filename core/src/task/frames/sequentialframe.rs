use crate::task::{
    ArcTaskEvent, GroupedTaskFramesExecBehavior, GroupedTaskFramesQuitOnFailure, TaskContext,
    TaskError, TaskEvent, TaskFrame,
};
use async_trait::async_trait;
use std::sync::Arc;

#[allow(unused_imports)]
use crate::task::ParallelTaskFrame;

/// Represents a **sequential task frame** which wraps multiple [`TaskFrame`] to execute at the same time
/// in a sequential manner. This task frame type acts as a **composite node** within the [`TaskFrame`] hierarchy,
/// facilitating a way to represent multiple [`TaskFrame`] which have same timings but depend on each
/// previous task frame finishing. The order of execution is ordered, and thus why its sequential,
/// in the case where execution order matters, it is advised to use [`ParallelTaskFrame`]
/// as opposed to [`SequentialTaskFrame`]
///
/// # Constructor(s)
/// When constructing a [`SequentialTaskFrame`], one can use either [`SequentialTaskFrame::new`] for no explicit
/// [`GroupedTaskFramesExecBehavior`] policy (convenience) or [`SequentialTaskFrame::new_with`]
/// if they do want to specify the [`GroupedTaskFramesExecBehavior`] policy as well
///
/// # Events
/// For events, [`SequentialTaskFrame`] has 2 of them, these being [`SequentialTaskFrame::on_child_start`] and
/// [`SequentialTaskFrame::on_child_end`],the former is for when a child task frame is about to start, the
/// event hands out the target [`TaskFrame`]. For the latter, it is for when a child task frame ends,
/// the event hands out the target task frame and an optional error in case it fails
///
/// # Trait Implementation(s)
/// It is obvious that the [`SequentialTaskFrame`] implements [`TaskFrame`] since this
/// is a part of the default provided implementations, however there are many others
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
/// # See Also
/// - [`TaskFrame`]
/// - [`ParallelTaskFrame`]
/// - [`GroupedTaskFramesExecBehavior`]
//noinspection DuplicatedCode
pub struct SequentialTaskFrame {
    tasks: Vec<Arc<dyn TaskFrame>>,
    policy: Arc<dyn GroupedTaskFramesExecBehavior>,

    /// Event fired for when a child [`TaskFrame`] starts execution
    pub on_child_start: ArcTaskEvent<Arc<dyn TaskFrame>>,

    /// Event fired for when a child [`TaskFrame`] has ended execution
    pub on_child_end: ArcTaskEvent<(Arc<dyn TaskFrame>, Option<TaskError>)>,
}

impl SequentialTaskFrame {
    /// Creates / Constructs a new [`SequentialTaskFrame`] instance based on
    /// the child [`TaskFrame`] collection supplied. If one wishes to
    /// also supply their own [`GroupedTaskFramesExecBehavior`], then they can use
    /// [`SequentialTaskFrame::new_with`]
    ///
    /// # Argument(s)
    /// This method accepts one single argument, that is the collection of [`TaskFrame`] to wrap
    /// around the [`SequentialTaskFrame`] to execute concurrently
    ///
    /// # Returns
    /// A fully created [`SequentialTaskFrame`] with the wrapped ``tasks``
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`GroupedTaskFramesExecBehavior`]
    /// - [`SequentialTaskFrame::new_with`]
    /// - [`SequentialTaskFrame`]
    pub fn new(tasks: Vec<Arc<dyn TaskFrame>>) -> SequentialTaskFrame {
        Self::new_with(tasks, GroupedTaskFramesQuitOnFailure)
    }

    /// Creates / Constructs a new [`SequentialTaskFrame`] instance based on
    /// the child [`TaskFrame`] collection and a [`GroupedTaskFramesExecBehavior`] policy supplied.
    /// If one wishes to prefer the default [`GroupedTaskFramesExecBehavior`], then they can use
    /// [`SequentialTaskFrame::new`] for convenience
    ///
    /// # Argument(s)
    /// This method accepts two arguments, those being the collection of [`TaskFrame`] to wrap
    /// around the [`SequentialTaskFrame`] to execute concurrently and a [`GroupedTaskFramesExecBehavior`]
    /// policy
    ///
    /// # Returns
    /// A fully created [`SequentialTaskFrame`] with the wrapped ``tasks`` and a custom ``policy``
    /// as a [`GroupedTaskFramesExecBehavior`]
    /// ``
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`GroupedTaskFramesExecBehavior`]
    /// - [`SequentialTaskFrame::new`]
    /// - [`SequentialTaskFrame`]
    pub fn new_with(
        tasks: Vec<Arc<dyn TaskFrame>>,
        sequential_policy: impl GroupedTaskFramesExecBehavior + 'static,
    ) -> SequentialTaskFrame {
        Self {
            tasks,
            policy: Arc::new(sequential_policy),
            on_child_end: TaskEvent::new(),
            on_child_start: TaskEvent::new(),
        }
    }
}

#[async_trait]
impl TaskFrame for SequentialTaskFrame {
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        let restricted_context = ctx.as_restricted();
        for task in self.tasks.iter() {
            ctx.emitter
                .clone()
                .emit(
                    restricted_context.clone(),
                    self.on_child_start.clone(),
                    task.clone(),
                )
                .await;
            let result = task.execute(ctx.clone()).await;
            ctx.emitter
                .clone()
                .emit(
                    restricted_context.clone(),
                    self.on_child_end.clone(),
                    (task.clone(), result.clone().err()),
                )
                .await;
            let should_quit = self.policy.should_quit(result).await;
            if let Some(res) = should_quit {
                return res;
            }
        }
        Ok(())
    }
}
