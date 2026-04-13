use async_trait::async_trait;
use chronographer::{
    errors::TaskError,
    task::{Task, TaskFrame, TaskFrameBuilder, TaskFrameContext, TaskScheduleImmediate},
};
use std::sync::{Arc, Mutex};

struct PrimaryTaskFrame;

#[async_trait]
impl TaskFrame for PrimaryTaskFrame {
    type Args = ();
    type Error = String;

    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), Self::Error> {
        Err("primary failed".to_string())
    }
}

struct SecondaryTaskFrame {
    received_error: Arc<Mutex<Option<String>>>,
}

#[async_trait]
impl TaskFrame for SecondaryTaskFrame {
    type Error = String;
    type Args = ((), String);
    async fn execute(&self, ctx: &TaskFrameContext, args: &Self::Args) -> Result<(), String> {
        let (_, err) = args;
        *self.received_error.lock().unwrap() = Some(err.clone());
        Ok(())
    }
}

#[tokio::test]

async fn test_sharing_error_to_secondary_task_frame() {
    let received_error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let primary = PrimaryTaskFrame;
    let secondary = SecondaryTaskFrame {
        received_error: received_error.clone(),
    };

    let builder = TaskFrameBuilder::new(primary)
        .with_fallback(secondary)
        .build();

    let task = Task::new(TaskScheduleImmediate, builder);

    task.into_erased().run().await.unwrap();

    assert_eq!(
        Some("primary failed".to_string()),
        *received_error.lock().unwrap()
    );
}
