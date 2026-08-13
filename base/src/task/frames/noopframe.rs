use crate::errors::TaskError;
use crate::task::{TaskFrame, TaskFrameContext};
use std::marker::PhantomData;

/// The [`NoOperationTaskFrame`] is a no-code and zero-sized [`TaskFrame`] unlike other
/// wrapper-based [`TaskFrames`], it doesn't host a [`TaskFrame`] and always produces success. It can
/// host any kind of [`TaskError`] and any kind of argument (effectively ignoring them).
///
/// Another caveat is due to its nature, it doesn't include a workflow primitive inside the [`workflow`]
/// macro. It's useful optional-based [`TaskFrames`] where a default [`TaskFrame`] is required that doesn't
/// execute any code and is promptly skipped.
///
/// # Events
/// Since it always returns success and no operations are involved, the [`NoOperationTaskFrame`] doesn't
/// broadcast any kind of event.
///
/// # Constructor(s)
/// The primary way of constructing a [`NoOperationTaskFrame`] is via the [`NoOperationTaskFrame::default`]
/// which is from the [`Default`] trait implementation.
///
/// # Trait Implementation(s)
/// [`NoOperationTaskFrame`] implements a variety of traits apart from just [`TaskFrame`], one of which
/// as already discussed is [`Default`] for constructing it. Other traits include [`Copy`] for copying
/// it and [`Clone`] as a required super trait.
///
/// # Example(s)
/// ```rust
/// use chronographer_base::task::NoOperationTaskFrame;
///
/// let frame: NoOperationTaskFrame<String> = NoOperationTaskFrame::default();
/// ```
///
/// # See Also
/// - [`TaskFrame`] - The core trait that [`NoOperationTaskFrame`] implements and uses.
/// - [`TaskError`] - The error which [`NoOperationTaskFrame`] uses (for type reasons, apart from that its ignored).
/// - [`workflow`] - Used for defining ergonomically an entire workflow of wrapped-based [`TaskFrames`]
pub struct NoOperationTaskFrame<E, Args = ()>(PhantomData<(E, Args)>);

impl<E: TaskError, Args: 'static + Send + Sync> Default for NoOperationTaskFrame<E, Args> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E: TaskError, Args: 'static + Send + Sync> Clone for NoOperationTaskFrame<E, Args> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: TaskError, Args: 'static + Send + Sync> Copy for NoOperationTaskFrame<E, Args> {}

impl<E: TaskError, Args: 'static + Send + Sync> TaskFrame for NoOperationTaskFrame<E, Args> {
    type Error = E;
    type Args = Args;
    type Workflow = Self;

    async fn execute(
        &self,
        _ctx: &TaskFrameContext,
        _args: &Self::Args,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
