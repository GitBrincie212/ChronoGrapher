use crate::errors::ChronographerErrors;
use crate::persistent_object::PersistentObject;
use crate::retrieve_registers::RetrieveRegistries;
use crate::serialized_component::SerializedComponent;
use crate::task::noopframe::NoOperationTaskFrame;
use crate::task::{ArcTaskEvent, TaskContext, TaskError, TaskEvent, TaskFrame};
use crate::utils::PersistenceUtils;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[allow(unused_imports)]
use crate::task::FallbackTaskFrame;

/// [`ConditionalFramePredicate`] is a trait that works closely with [`ConditionalFrame`], it is the
/// mechanism that returns true/false for the task to execute
///
/// # Required Method(s)
/// When implementing the [`ConditionalFramePredicate`], one has to supply an implementation
/// for the method being [`ConditionalFramePredicate::execute`] which is where the main logic
/// lives, where it accepts the [`TaskContext`]
///
/// # Trait Implementation(s)
/// By default [`ConditionalFramePredicate`] is implemented on any async function that accepts a
/// task context (specifically wrapped in an ``Arc``) and returns a boolean
///
/// # See Also
/// - [`TaskFrame`]
/// - [`ConditionalFrame`]
/// - [`TaskContext`]
#[async_trait]
pub trait ConditionalFramePredicate: Send + Sync {
    /// Executes the predicate and returns a boolean indicating to allow or not
    /// the [`TaskFrame`] to execute
    ///
    /// # Argument(s)
    /// The method accepts one argument, that being the [`TaskContext`] wrapped
    /// in an ``Arc``
    ///
    /// # Returns
    /// A boolean indicating if the [`TaskFrame`] should be executed
    /// or not, true for yes it should and false for the opposite
    ///
    /// # See Also
    /// - [`TaskContext`]
    /// - [`TaskFrame`]
    /// - [`ConditionalFramePredicate`]
    async fn execute(&self, ctx: Arc<TaskContext>) -> bool;
}

#[async_trait]
impl<F, Fut> ConditionalFramePredicate for F
where
    F: Fn(Arc<TaskContext>) -> Fut + Send + Sync,
    Fut: Future<Output = bool> + Send,
{
    async fn execute(&self, ctx: Arc<TaskContext>) -> bool {
        self(ctx).await
    }
}

/// [`ConditionalFrameConfig`] is a typed builder and by itself
/// it is not as useful, only useful for construction of a [`ConditionalFrame`]
#[derive(TypedBuilder)]
#[builder(build_method(into = ConditionalFrame<T, T2>))]
pub struct ConditionalFrameConfig<T, T2>
where
    T: TaskFrame + 'static + Send + Sync,
    T2: TaskFrame + 'static + Send + Sync,
{
    /// A fallback [`TaskFrame`] for handling the execution in
    /// case the [`ConditionalFramePredicate`] returned false, depending on the
    /// builder used, this might already be inserted
    ///
    /// # Default Value
    /// If the [`ConditionalFrame::builder`] is used then this value is
    /// not required to be supplied, thus using as a default value [`NoOperationTaskFrame`],
    /// if however [`ConditionalFrame::fallback_builder`] is used, then it has to be
    /// specified
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`ConditionalFrame::builder`]
    /// - [`ConditionalFrame::fallback_builder`]
    /// - [`ConditionalFramePredicate`]
    #[builder(setter(transform = |s: T2| Arc::new(s)))]
    fallback: Arc<T2>,

    /// The [`TaskFrame`] for handling the execution in
    /// case the [`ConditionalFramePredicate`] returned true,
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
    /// - [`ConditionalFramePredicate`]
    #[builder(setter(transform = |s: T| Arc::new(s)))]
    frame: Arc<T>,

    /// The [`ConditionalFramePredicate`] for handling the decision-making on whenever
    /// to execute the [`TaskFrame`] or not based on a boolean value
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
    /// - [`ConditionalFramePredicate`]
    #[builder(setter(transform = |s: impl ConditionalFramePredicate + 'static| {
        Arc::new(s) as Arc<dyn ConditionalFramePredicate>
    }))]
    predicate: Arc<dyn ConditionalFramePredicate>,

    /// A boolean value indicating to error or not when the
    /// [`ConditionalFramePredicate`] returns false
    ///
    /// # Default Value
    /// By default, every [`ConditionalFrame`] will return a success and not error out
    ///
    /// # Method Behavior
    /// This builder parameter method cannot be chained, as it is a typed builder,
    /// once set, you can never chain it. Since it is a typed builder, it has no fancy
    /// inner workings under the hood, just sets the value
    ///
    /// # See Also
    /// - [`TaskFrame`]
    /// - [`ConditionalFrame`]
    /// - [`ConditionalFramePredicate`]
    #[builder(default = false)]
    error_on_false: bool,
}

impl<T, T2> From<ConditionalFrameConfig<T, T2>> for ConditionalFrame<T, T2>
where
    T: TaskFrame + 'static + Send + Sync,
    T2: TaskFrame + 'static + Send + Sync,
{
    fn from(config: ConditionalFrameConfig<T, T2>) -> Self {
        ConditionalFrame {
            frame: config.frame,
            fallback: config.fallback,
            predicate: config.predicate,
            error_on_false: config.error_on_false,
            on_true: TaskEvent::new(),
            on_false: TaskEvent::new(),
        }
    }
}

/// Represents a **conditional task frame** which wraps a task frame and executes it depending on a
/// predicate function. This task frame type acts as a **wrapper node** within the task frame hierarchy,
/// facilitating a way to conditionally execute a task frame
///
/// A fallback can optionally be registered for the [`ConditionalFrame`] to allow the execution of another
/// task frame in case the predicate returns false (otherwise nothing is executed). The results from
/// the fallback will be returned if that is the case, otherwise the primary task may return
///
/// There is an important difference between [`ConditionalFrame`] and [`FallbackTaskFrame`], the
/// former relies on the predicate to determine which task frame to execute, while the latter relies
/// on whenever or not the primary task failed. In addition, [`ConditionalFrame`] **WILL NOT** execute
/// any task frame unless it has the boolean value, whereas [`FallbackTaskFrame`] **WILL** execute the
/// primary task always and potentially then the second task frame
///
/// # Events
/// For events, [`ConditionalFrame`] has only two events, those being [`ConditionalFrame::on_true`]
/// and [`ConditionalFrame::on_false`], they execute depending on the predicate and both host the
/// target [`TaskFrame`] which will be executed
///
/// # Constructor(s)
/// When construing a [`ConditionalFrame`] one can use [`ConditionalFrame::fallback_builder`] which
/// creates a builder that has a required parameter for a fallback task frame, or for convenience,
/// [`ConditionalFrame::builder`] that automatically fills it with a [`NoOperationTaskFrame`] which
/// does nothing
///
/// # Trait Implementation(s)
/// It is obvious that the [`ConditionalFrame`] implements [`TaskFrame`] since this
/// is a part of the default provided implementations, however there are many others
///
/// # Example
/// ```ignore
/// use std::sync::Arc;
/// use chronographer_core::schedule::TaskScheduleInterval;
/// use chronographer_core::scheduler::{Scheduler, CHRONOGRAPHER_SCHEDULER};
/// use chronographer_core::task::executionframe::ExecutionTaskFrame;
/// use chronographer_core::task::conditionframe::ConditionalFrame;
/// use chronographer_core::task::Task;
///
/// let primary_frame = ExecutionTaskFrame::new(
///     |_ctx| async {
///         println!("Primary task frame fired...");
///         Ok(())
///     }
/// );
///
/// // This is optional to specify
/// let secondary_frame = ExecutionTaskFrame::new(
///     |_ctx| async {
///         println!("Secondary task frame fired...");
///         Ok(())
///     }
/// );
///
/// let conditional_frame: ConditionalFrame<ExecutionTaskFrame<_>, ExecutionTaskFrame<_>> =
///     ConditionalFrame::builder()
///         .task(primary_frame)
///         .fallback(secondary_frame) // Remove this to not specify a fallback
///         .error_on_false(true) // Also an optional parameter, but can be useful in some cases
///         .predicate(|metadata| metadata.runs() % 2 == 0)
///         .build();
///
/// let task = Task::define(TaskScheduleInterval::from_secs_f64(3.21), conditional_frame);
///
/// CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
/// ```
///
/// # See Also
/// - [`TaskFrame`]
/// - [`ConditionalFramePredicate`]
/// - [`NoOperationTaskFrame`]
/// - [`ConditionalFrame::builder`]
/// - [`ConditionalFrame::fallback_builder`]
/// - [`TaskEvent`]
/// - [`FallbackTaskFrame`]
pub struct ConditionalFrame<T: 'static, T2: 'static = NoOperationTaskFrame> {
    frame: Arc<T>,
    fallback: Arc<T2>,
    predicate: Arc<dyn ConditionalFramePredicate>,
    error_on_false: bool,

    /// Event fired for when [`ConditionalFramePredicate`] returns true
    pub on_true: ArcTaskEvent<Arc<T>>,

    /// Event fired for when [`ConditionalFramePredicate`] returns false
    pub on_false: ArcTaskEvent<Arc<T2>>,
}

/// A type alias to alleviate the immense typing required to specify that
/// the [`ConditionalFrameConfigBuilder`] has already filled the fallback parameter
/// as a [`NoOperationTaskFrame`]
pub type NonFallbackCFCBuilder<T> = ConditionalFrameConfigBuilder<
    T,
    NoOperationTaskFrame,
    ((Arc<NoOperationTaskFrame>,), (), (), ()),
>;

impl<T> ConditionalFrame<T>
where
    T: TaskFrame + 'static + Send + Sync,
{
    /// Creates / Constructs a builder for the construction of [`ConditionalFrame`],
    /// that contains no fallback option. If one would wish to supply the fallback
    /// option as well, then there is also [`ConditionalFrame::fallback_builder`]
    /// for that purpose
    ///
    /// # Returns
    /// A fully created [`NonFallbackCFCBuilder`]
    ///
    /// # See Also
    /// - [`ConditionalFrame`]
    /// - [`ConditionalFrame::fallback_builder`]
    pub fn builder() -> NonFallbackCFCBuilder<T> {
        ConditionalFrameConfig::builder().fallback(NoOperationTaskFrame)
    }
}

impl<T, T2> ConditionalFrame<T, T2>
where
    T: TaskFrame + 'static + Send + Sync,
    T2: TaskFrame + 'static + Send + Sync,
{
    /// Creates / Constructs a builder for the construction of [`ConditionalFrame`],
    /// that requires a fallback option. If one would wish to not supply the fallback
    /// option, then there is also [`ConditionalFrame::builder`] for that convenience purpose
    ///
    /// # Returns
    /// A fully created [`ConditionalFrameConfigBuilder`]
    ///
    /// # See Also
    /// - [`ConditionalFrame`]
    /// - [`ConditionalFrame::builder`]
    pub fn fallback_builder() -> ConditionalFrameConfigBuilder<T, T2> {
        ConditionalFrameConfig::builder()
    }
}

#[async_trait]
impl<T: TaskFrame, F: TaskFrame> TaskFrame for ConditionalFrame<T, F> {
    async fn execute(&self, ctx: Arc<TaskContext>) -> Result<(), TaskError> {
        let result = self.predicate.execute(ctx.clone()).await;
        if result {
            ctx.emitter
                .emit(
                    ctx.as_restricted(),
                    self.on_true.clone(),
                    self.frame.clone(),
                )
                .await;
            return self.frame.execute(ctx).await;
        }
        ctx.emitter
            .emit(
                ctx.as_restricted(),
                self.on_false.clone(),
                self.fallback.clone(),
            )
            .await;
        let result = self.fallback.execute(ctx).await;
        if self.error_on_false && result.is_ok() {
            return Err(Arc::new(ChronographerErrors::TaskConditionFail));
        }
        result
    }
}

#[async_trait]
impl<T, F> PersistentObject for ConditionalFrame<T, F>
where
    T: TaskFrame + 'static + PersistentObject,
    F: TaskFrame + 'static + PersistentObject,
{
    fn persistence_id() -> &'static str {
        "ConditionalFrame$chronographer_core"
    }

    async fn persist(&self) -> Result<SerializedComponent, TaskError> {
        let frame = PersistenceUtils::serialize_persistent(self.frame.as_ref()).await?;
        let fallback = PersistenceUtils::serialize_persistent(self.fallback.as_ref()).await?;
        let errors_on_false = PersistenceUtils::serialize_field(self.error_on_false)?;
        let predicate =
            PersistenceUtils::serialize_potential_field(self.predicate.as_ref()).await?;
        Ok(SerializedComponent::new::<Self>(json!({
            "wrapped_primary": frame,
            "wrapped_fallback": fallback,
            "errors_on_false": errors_on_false,
            "predicate": predicate,
        })))
    }

    async fn retrieve(component: SerializedComponent) -> Result<Self, TaskError> {
        let mut repr = PersistenceUtils::transform_serialized_to_map(component)?;

        let primary_frame = PersistenceUtils::deserialize_concrete::<T>(
            &mut repr,
            "wrapped_primary",
            "Cannot deserialize the primary wrapped task frame",
        )
        .await?;

        let fallback_frame = PersistenceUtils::deserialize_concrete::<F>(
            &mut repr,
            "wrapped_fallback",
            "Cannot deserialize the fallback wrapped task frame",
        )
        .await?;

        let error_on_false = PersistenceUtils::deserialize_atomic::<bool>(
            &mut repr,
            "error_on_false",
            "Cannot deserialize the boolean to decide whenever or not to error on false",
        )?;

        let predicate = PersistenceUtils::deserialize_dyn(
            &mut repr,
            "predicate",
            RetrieveRegistries::retrieve_conditional_predicate,
            "Cannot deserialize the conditional predicate",
        )
        .await?;

        Ok(ConditionalFrame {
            frame: Arc::new(primary_frame),
            fallback: Arc::new(fallback_frame),
            error_on_false,
            predicate,
            on_true: TaskEvent::new(),
            on_false: TaskEvent::new(),
        })
    }
}
