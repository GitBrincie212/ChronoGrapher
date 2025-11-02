use chronographer_core::scheduler::CHRONOGRAPHER_SCHEDULER;
use chronographer_core::task::{ExecutionTaskFrame, Task, TaskScheduleInterval};

#[tokio::main]
#[allow(clippy::empty_loop)]
async fn main() {
    let task = Task::define(
        TaskScheduleInterval::from_secs(4),
        ExecutionTaskFrame::new(|_ctx| async {
            println!("Hello World");
            Ok(())
        })
    );

    CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
    loop {}
}
