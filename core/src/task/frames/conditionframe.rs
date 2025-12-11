use crate::define_event;
use crate::errors::ChronographerErrors;
#[allow(unused_imports)]
use crate::task::FallbackTaskFrame;
use crate::task::TaskHookEvent;
use crate::task::noopframe::NoOperationTaskFrame;
use crate::task::{TaskContext, TaskError, TaskFrame};
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use typed_builder::TypedBuilder;

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
    async fn execute(&self, ctx: &TaskContext) -> bool;
}

#[async_trait]
impl<F, Fut> ConditionalFramePredicate for F
where
    F: Fn(&TaskContext) -> Fut + Send + Sync,
    Fut: Future<Output = bool> + Send,
{
    async fn execute(&self, ctx: &TaskContext) -> bool {
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
    fallback: &'static T2,

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
    frame: &'static T,

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

define_event!(
    /// # Event Triggering
    /// [`OnTruthyValueEvent`] is triggered when the [`ConditionalFrame`]'s predicate function
    /// (which is [`ConditionalFramePredicate`]) returns a true boolean value
    ///
    /// # See Also
    /// - [`ConditionalFrame`]
    /// - [`ConditionalFramePredicate`]
    OnTruthyValueEvent, ()
);

define_event!(
    /// # Event Triggering
    /// [`OnFalseyValueEvent`] is triggered when the [`ConditionalFrame`]'s predicate function
    /// (which is [`ConditionalFramePredicate`]) returns a false boolean value
    ///
    /// # See Also
    /// - [`ConditionalFrame`]
    /// - [`ConditionalFramePredicate`]
    OnFalseyValueEvent, ()
);

impl<T, T2> From<ConditionalFrameConfig<T, T2>> for ConditionalFrame<T, T2>
where
    T: TaskFrame + 'static + Send + Sync,
    T2: TaskFrame + 'static + Send + Sync,
{
    fn from(config: ConditionalFrameConfig<T, T2>) -> Self {
        ConditionalFrame {
            frame: &config.frame,
            fallback: &config.fallback,
            predicate: config.predicate,
            error_on_false: config.error_on_false,
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
    frame: T,
    fallback: T2,
    predicate: Arc<dyn ConditionalFramePredicate>,
    error_on_false: bool,
} // TODO: See how to persist conditional predicate

/// A type alias to alleviate the immense typing required to specify that
/// the [`ConditionalFrameConfigBuilder`] has already filled the fallback parameter
/// as a [`NoOperationTaskFrame`]
pub type NonFallbackCFCBuilder<T> = ConditionalFrameConfigBuilder<
    T,
    NoOperationTaskFrame,
    ((&'static NoOperationTaskFrame,), (), (), ()),
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
        ConditionalFrameConfig::builder().fallback(&NoOperationTaskFrame)
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
    async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
        let result = self.predicate.execute(ctx).await;
        if result {
            ctx.clone().emit::<OnTruthyValueEvent>(&()).await; // skipcq: RS-E1015
            return ctx.subdivide(self.frame).await;
        }

        ctx.clone().emit::<OnFalseyValueEvent>(&()).await; // skipcq: RS-E1015
        let result = ctx.subdivide(self.fallback.clone()).await;
        if self.error_on_false && result.is_ok() {
            return Err(Arc::new(ChronographerErrors::TaskConditionFail));
        }
        result
    }
}

/*
#[async_trait]
impl<F1, F2> PersistenceObject for ConditionalFrame<F1, F2>
where
    F1: TaskFrame + 'static + PersistenceObject,
    F2: TaskFrame + 'static + PersistenceObject,
{
    const PERSISTENCE_ID: &'static str =
        "chronographer::ConditionalFrame#251f88d9-cecd-475d-85d3-0601657aedf4";

    fn inject_context<T: PersistenceBackend>(&self, ctx: &PersistenceContext<T>) {
        todo!()
    }
}
 */
