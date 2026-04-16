use chronographer::{
    errors::TaskError,
    task::{Task, TaskFrame, TaskFrameContext, TaskScheduleImmediate},
};
use std::{fmt, marker::PhantomData};

#[derive(thiserror::Error, Debug)]
enum DummyError {
    #[allow(dead_code)]
    Error(Box<dyn TaskError>),
}

impl fmt::Display for DummyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(write!(f, "Error!")?)
    }
}

#[derive(Debug)]
struct MyTaskFrame(PhantomData<i32>);

impl Default for MyTaskFrame {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl Clone for MyTaskFrame {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for MyTaskFrame {}

impl TaskFrame for MyTaskFrame {
    type Error = DummyError;
    type Args = ();

    async fn execute(
        &self,
        _ctx: &TaskFrameContext,
        _args: &Self::Args,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[tokio::test]
async fn noop_task_frame_execution_test() {
    let data: PhantomData<i32> = PhantomData;
    let frame = MyTaskFrame(data);
    let task = Task::new(TaskScheduleImmediate, frame);

    let exec_result = task.into_erased().run().await;
    assert!(
        exec_result.is_ok(),
        "NoOperationTaskFrame execution should always returns unit"
    );
}
