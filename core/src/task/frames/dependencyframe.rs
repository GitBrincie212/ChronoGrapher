use crate::deserialization_err;
use crate::task::Debug;
use crate::errors::ChronographerErrors;
use crate::task::dependency::FrameDependency;
use crate::task::{Arc, TaskContext, TaskError, TaskEvent, TaskFrame};
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;
use crate::persistent_object::{AsPersistent, PersistenceCapability, PersistentObject};
use crate::serialized_component::SerializedComponent;
use crate::{acquire_mut_ir_map, deserialize_field, to_json};
use crate::retrieve_registers::RETRIEVE_REGISTRIES;

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
pub struct DependentFailureOnFail;

#[async_trait]
impl DependentFailBehavior for DependentFailureOnFail {
    async fn execute(&self) -> Result<(), TaskError> {
        Err(Arc::new(ChronographerErrors::TaskDependenciesUnresolved))
    }
}

/// When dependencies aren't resolved, return a `Ok(())`
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
    task: T,

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
            frame: config.task,
            dependencies: config.dependencies,
            dependent_behaviour: config.dependent_behaviour,
            on_dependency: TaskEvent::new(),
        }
    }
}

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
    frame: T,
    dependencies: Vec<Arc<dyn FrameDependency>>,
    dependent_behaviour: Arc<dyn DependentFailBehavior>,

    /// Event fired for when [`FrameDependency`] finishes the resolve process,
    /// it contains both the target dependency, and a boolean indicating if it
    /// has been resolved or not
    pub on_dependency: Arc<TaskEvent<(Arc<dyn FrameDependency>, bool)>>,
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
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError> {
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
                    ctx.emitter
                        .emit(ctx.as_restricted(), self.on_dependency.clone(), (dep, res))
                        .await;
                    if !res {
                        is_resolved = false;
                        break;
                    }
                }
                Err(_) => {
                    ctx.emitter
                        .emit(
                            ctx.as_restricted(),
                            self.on_dependency.clone(),
                            (dep, false),
                        )
                        .await;
                    is_resolved = false;
                    break;
                }
            }
        }

        if !is_resolved {
            return self.dependent_behaviour.execute().await;
        }

        self.frame.execute(ctx).await
    }
}

#[async_trait]
impl<T: TaskFrame + PersistentObject> PersistentObject for DependencyTaskFrame<T> {
    fn persistence_id() -> &'static str {
        "DependencyTaskFrame$chronographer_core"
    }

    async fn store(&self) -> Result<SerializedComponent, TaskError> {
        let wrapped_frame = to_json!(self.frame.store().await?);
        let mut dependencies = Vec::with_capacity(self.dependencies.len());

        for dep in &self.dependencies {
            let persistent_obj = match dep.as_persistent().await {
                PersistenceCapability::Persistable(obj) => obj,
                _ => {
                    return Err(Arc::new(ChronographerErrors::NonPersistentObject(
                        "FrameDependency".to_string(),
                    )) as Arc<dyn Debug + Send + Sync>);
                }
            };

            let serialized = persistent_obj.store().await?;
            dependencies.push(serialized.into_ir());
        }

        let dependent_fail_behavior = match self.dependent_behaviour.as_persistent().await {
            PersistenceCapability::Persistable(res) => {
                serde_json::to_value(res.store().await?)
                    .map_err(|x| Arc::new(x) as Arc<dyn Debug + Send + Sync>)?
            }
            _ => {
                return Err(Arc::new(ChronographerErrors::NonPersistentObject(
                    "DependentFailBehavior".to_string(),
                )));
            }
        };

        Ok(SerializedComponent::new::<Self>(
            json!({
                "wrapped_frame": wrapped_frame,
                "dependencies": dependencies,
                "dependent_fail_behavior": dependent_fail_behavior
            })
        ))
    }

    async fn retrieve(component: SerializedComponent) -> Result<Self, TaskError> {
        let mut repr = acquire_mut_ir_map!(DependencyTaskFrame, component);
        deserialize_field!(
            repr,
            serialized_frame,
            "wrapped_frame",
            DependencyTaskFrame,
            "Cannot deserialize the wrapped task frame"
        );

        deserialize_field!(
            repr,
            mut serialized_dependencies,
            "dependencies",
            DependencyTaskFrame,
            "Cannot deserialize the dependencies"
        );

        deserialize_field!(
            repr,
            serialized_dependent_fail_behavior,
            "dependent_fail_behavior",
            DependencyTaskFrame,
            "Cannot deserialize the DependentFailBehavior"
        );

        let wrapped_frame = T::retrieve(
            serde_json::from_value::<SerializedComponent>(serialized_frame)
                .map_err(|err| Arc::new(err) as Arc<dyn Debug + Send + Sync>)?,
        ).await?;

        let dependent_behaviour = RETRIEVE_REGISTRIES.retrieve_dependent_fail_behaviour(
            serde_json::from_value::<SerializedComponent>(serialized_dependent_fail_behavior)
                .map_err(|x| Arc::new(x) as Arc<dyn Debug + Send + Sync>)?
        ).await?;

        let dependencies: Vec<Value> = serialized_dependencies
            .as_array_mut()
            .ok_or_else(|| {
                deserialization_err!(
                    repr,
                    DependencyTaskFrame,
                    "Cannot deserialize the dependencies"
                )
            })?
            .drain(..)
            .collect();

        let mut deserialized_dependencies = Vec::with_capacity(dependencies.len());

        for dep in dependencies {
            let dep = RETRIEVE_REGISTRIES.retrieve_frame_dependency(
                serde_json::from_value::<SerializedComponent>(dep)
                    .map_err(|x| Arc::new(x) as Arc<dyn Debug + Send + Sync>)?
            ).await?;

            deserialized_dependencies.push(dep)
        }

        Ok(DependencyTaskFrame {
            frame: wrapped_frame,
            dependencies: deserialized_dependencies,
            dependent_behaviour,
            on_dependency: TaskEvent::new(),
        })
    }
}