use crate::define_event;
use crate::errors::ChronographerErrors;
use crate::task::Debug;
use crate::task::TaskHookEvent;
use crate::task::dependency::FrameDependency;
use crate::task::{Arc, TaskContext, TaskError, TaskFrame};
use async_trait::async_trait;
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;

/// [`DependentFailBehavior`] is a trait for implementing a behavior when dependencies aren't resolved
/// in [`DependencyTaskFrame`]. It takes nothing and returns a result for the [`DependencyTaskFrame`] to
/// return
///
/// # Trait Implementation(s)
/// There are 2 implementations of the [`DependentFailBehavior`] trait present, those being:
/// - [`DependentFailureOnFail`] Returns a [`ChronographerErrors::TaskDependenciesUnresolved`]
/// - [`DependentSuccessOnFail`] Returns a ``Ok(())``
///
/// By default, [`DependencyTaskFrame`] uses [`DependentFailureOnFail`]
///
/// # Object Safety
/// [`DependentFailBehavior`] is object safe as seen in the source code of [`DependencyTaskFrame`]
///
/// # See Also
/// - [`DependencyTaskFrame`]
/// - [`DependentFailureOnFail`]
/// - [`DependentSuccessOnFail`]
#[async_trait]
pub trait DependentFailBehavior: Send + Sync {
    /// The main logic to execute that determines the result to return back
    /// to the [`DependencyTaskFrame`] (so it can also return it back)
    ///
    /// # Returns
    /// A result based on a constant or something more dynamic, where
    /// it maps one to one with the results from [`DependencyTaskFrame`]
    ///
    /// # See Also
    /// - [`DependencyTaskFrame`]
    /// - [`DependentFailBehavior`]
    async fn execute(&self) -> Result<(), TaskError>;
}

#[async_trait]
impl<DFB: DependentFailBehavior> DependentFailBehavior for Arc<DFB> {
    async fn execute(&self) -> Result<(), TaskError> {
        self.as_ref().execute().await
    }
}

/// When dependencies aren't resolved, return an error, more specifically
/// the ``ChronographerErrors::TaskDependenciesUnresolved`` error
#[derive(Default, Clone, Copy)]
pub struct DependentFailureOnFail;

#[async_trait]
impl DependentFailBehavior for DependentFailureOnFail {
    async fn execute(&self) -> Result<(), TaskError> {
        Err(Arc::new(ChronographerErrors::TaskDependenciesUnresolved))
    }
}

/// When dependencies aren't resolved, return a `Ok(())`
#[derive(Default, Clone, Copy)]
pub struct DependentSuccessOnFail;

#[async_trait]
impl DependentFailBehavior for DependentSuccessOnFail {
    async fn execute(&self) -> Result<(), TaskError> {
        Ok(())
    }
}

/// [`DependencyTaskFrameConfig`] is a typed builder and by itself
/// it is not as useful, only useful for construction of a [`DependencyTaskFrame`]
#[derive(TypedBuilder)]
#[builder(build_method(into = DependencyTaskFrame<T>))]
pub struct DependencyTaskFrameConfig<T: TaskFrame> {
    /// The [`TaskFrame`] that is wrapped for handling all its [`FrameDependency`]
    ///
    /// # Default Value
    /// This builder method has no default value, as it is required
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`FrameDependency`]
    frame: T,

    /// A collection of [`FrameDependency`] tied to the inner [`TaskFrame`]. Where
    /// all the dependencies listed must be resolved in order to execute the wrapped
    /// [`TaskFrame`] (effectively acting as an AND for a collection of booleans)
    ///
    /// # Default Value
    /// This builder method has no default value, as it is required
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`FrameDependency`]
    dependencies: Vec<Arc<dyn FrameDependency>>,

    /// An implementation of the [`DependentFailBehavior`] for managing the behavior of the
    /// [`DependencyTaskFrame`] when dependencies aren't resolved
    ///
    /// # Default Value
    /// By default, all [`DependencyTaskFrame`] use [`DependentFailureOnFail`], which
    /// means when dependencies aren't resolved, the [`DependencyTaskFrame`] fails with
    /// an error, specifically [`ChronographerErrors::TaskDependenciesUnresolved`]
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`DependentFailureOnFail`]
    #[builder(
        default = Arc::new(DependentFailureOnFail),
        setter(transform = |ts: impl DependentFailBehavior + 'static| Arc::new(ts) as Arc<dyn DependentFailBehavior>)
    )]
    dependent_behaviour: Arc<dyn DependentFailBehavior>,
}

impl<T: TaskFrame> From<DependencyTaskFrameConfig<T>> for DependencyTaskFrame<T> {
    fn from(config: DependencyTaskFrameConfig<T>) -> Self {
        Self {
            frame: Arc::new(config.frame),
            dependencies: config.dependencies,
            dependent_behaviour: config.dependent_behaviour,
        }
    }
}

define_event!(
    /// [`OnDependencyValidation`] is an implementation of [`TaskHookEvent`] (a system used closely
    /// with [`TaskHook`]). The concrete payload type of [`OnDependencyValidation`]
    /// is ``(Arc<dyn FrameDependency>, bool)``, the first value describes the ``FrameDependency``
    /// being inspected and the second value describes if the ``FrameDependency`` was resolved
    /// or not
    ///
    /// # Constructor(s)
    /// When constructing a [`OnDependencyValidation`] due to the fact this is a marker ``struct``, making
    /// it as such zero-sized, one can either use [`OnDependencyValidation::default`] or via simply pasting
    /// the struct name ([`OnDependencyValidation`])
    ///
    /// # Trait Implementation(s)
    /// It is obvious that [`OnDependencyValidation`] implements the [`TaskHookEvent`], but also many
    /// other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`]
    /// and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]
    ///
    /// # Event Triggering
    /// [`OnDependencyValidation`] is triggered when the [`DependencyTaskFrame`] has finished
    /// validating a [`FrameDependency`] (the same one present in the payload)
    ///
    /// # Cloning Semantics
    /// When cloning / copy a [`OnDependencyValidation`] it fully creates a
    /// new independent version of that instance
    ///
    /// # See Also
    /// - [`DependencyTaskFrame`]
    /// - [`TaskHook`]
    /// - [`TaskHookEvent`]
    /// - [`Task`]
    /// - [`TaskFrame`]
    OnDependencyValidation, (Arc<dyn FrameDependency>, bool)
);

/// Represents an **dependent task frame** that directly wraps a task frame and executes it only if
/// all dependencies are resolved. This task frame type acts asa **wrapper node** within the task frame
/// hierarchy. Allowing the creation of task frames that depend on other tasks, in addition to allowing
/// dynamic execution (which opens the door for optimizations in case dependencies are expensive to compute)
///
/// # Constructor(s)
/// When construing a [`DependencyTaskFrame`] the only way to do so is via
/// [`DependencyTaskFrame::builder`] which creates a builder for [`DependencyTaskFrame`], then
/// simply supply the required fields and done
///
///
/// # Behavior
/// - Before executing the [`TaskFrame`] it calls [`FrameDependency::is_resolved`] on all
///   dependencies and checks if all of them are true
/// - if they are then the [`TaskFrame`] executes, otherwise [`DependentFailBehavior`] takes over
///
/// # Events
/// When it comes to events, [`DependencyTaskFrame`], there is only one, that being
/// [`ConditionalFrame::on_dependency`] which is triggered for every dependency and
/// hosts the [`FrameDependency`] as well as if it has been resolved (as boolean)
///
/// # Trait Implementation(s)
/// It is obvious that the [`DependencyTaskFrame`] implements [`TaskFrame`] since this
/// is a part of the default provided implementations, however there are many others
///
/// # Example
/// ```ignore
/// use std::sync::Arc;
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::executionframe::ExecutionTaskFrame;
/// use chronographer_core::task::{DependencyTaskFrame, Task};
/// use chronographer_core::task::dependency::TaskDependency;
///
/// let exec_frame1 = ExecutionTaskFrame::new(
///     |_ctx| async {
///         println!("Hello from primary execution task!");
///         Ok(())
///     }
/// );
///
/// let exec_frame2 = ExecutionTaskFrame::new(
///     |_ctx| async {
///         println!("Hello from secondary execution task!");
///         Ok(())
///     }
/// );
///
/// let task1 = Arc::new(Task::define(TaskScheduleInterval::from_secs(5), exec_frame1));
/// let task1_dependency = TaskDependency::builder()
///     .task(task1.clone())
///     .build();
///
/// let dependent_frame2 = DependencyTaskFrame::builder()
///     .task(exec_frame2)
///     .dependencies(
///         vec![
///             Arc::new(task1_dependency)
///         ]
///     )
///     .build();
///
/// let task2 = Task::define(TaskScheduleInterval::from_secs(5), dependent_frame2);
///
/// CHRONOGRAPHER_SCHEDULER.schedule(task1.clone()).await;
/// ```
///
/// # See Also
/// - [`TaskFrame`]
/// - [`FrameDependency`]
/// - [`TaskEvent`]
/// - [`DependentFailBehavior`]
/// - [`DependencyTaskFrame::builder`]
pub struct DependencyTaskFrame<T: TaskFrame> {
    frame: Arc<T>,
    dependencies: Vec<Arc<dyn FrameDependency>>,
    dependent_behaviour: Arc<dyn DependentFailBehavior>,
}

impl<T: TaskFrame> DependencyTaskFrame<T> {
    /// Creates / Constructs a builder for the construction of [`DependencyTaskFrame`],
    ///
    /// # Returns
    /// A fully created [`DependencyTaskFrameConfigBuilder`]
    ///
    /// # See Also
    /// - [`DependencyTaskFrame`]
    pub fn builder() -> DependencyTaskFrameConfigBuilder<T> {
        DependencyTaskFrameConfig::builder()
    }
}

#[async_trait]
impl<T: TaskFrame> TaskFrame for DependencyTaskFrame<T> {
    async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
        let mut handles: Vec<JoinHandle<bool>> = Vec::with_capacity(self.dependencies.len());

        for dep in &self.dependencies {
            let dep = dep.clone();
            handles.push(tokio::spawn(async move { dep.is_resolved().await }));
        }

        let mut is_resolved = true;
        for (index, handle) in handles.into_iter().enumerate() {
            let dep = self.dependencies[index].clone();
            match handle.await {
                Ok(res) => {
                    ctx.emit::<OnDependencyValidation>(&(dep, res))
                        .await;
                    if !res {
                        is_resolved = false;
                        break;
                    }
                }
                Err(_) => {
                    ctx.emit::<OnDependencyValidation>(&(dep, false))
                        .await;
                    is_resolved = false;
                    break;
                }
            }
        }

        if !is_resolved {
            return self.dependent_behaviour.execute().await;
        }

        ctx.subdivide(self.frame.clone()).await
    }
}
