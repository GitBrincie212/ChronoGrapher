use std::fmt::Debug;
use std::sync::Arc;
use chronographer::prelude::*;

#[tokio::main]
async fn main() {
    let exec_frame = DynamicTaskFrame::new(|_ctx| async {
        println!("Trying primary task...");
        //sleep(Duration::from_secs_f64(1.234)).await;
        Err(Arc::new("task failed") as Arc<dyn Debug + Send + Sync>)
    });

    //let timeout_frame = DelayTaskFrame::new(exec_frame, Duration::from_secs(3));

    let task = Task::simple(TaskScheduleInterval::from_secs(4), exec_frame);
    let _ = CHRONOGRAPHER_SCHEDULER.schedule(&task).await;
    CHRONOGRAPHER_SCHEDULER.start().await;
    loop {}
}