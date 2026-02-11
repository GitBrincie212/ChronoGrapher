use chronographer::prelude::*;
use std::sync::Arc;
use async_trait::async_trait;
use chronographer::errors::ChronographerErrors::ThresholdReachError;
use chronographer::task::TaskHookContext;

pub struct MyCoolTaskHook;

#[async_trait]
impl TaskHook<OnTaskStart> for MyCoolTaskHook {
    async fn on_event(&self, ctx: &TaskHookContext, payload: &<OnTaskStart as TaskHookEvent>::Payload) {
        println!("Interested event triggered!");
    }
}

#[async_trait]
impl TaskHook<OnTaskEnd> for MyCoolTaskHook {
    async fn on_event(&self, ctx: &TaskHookContext, payload: &<OnTaskEnd as TaskHookEvent>::Payload) {
        println!("Interested event triggered!");
    }
}

#[tokio::main]
async fn main() {
    let exec_frame = DynamicTaskFrame::new(|_ctx| async {
        println!("Trying primary task...");
        //sleep(Duration::from_secs_f64(1.234)).await;
        Err(Arc::new(ThresholdReachError) as DynArcError)
    });

    //let timeout_frame = DelayTaskFrame::new(exec_frame, Duration::from_secs(3));

    let task = Task::simple(TaskScheduleInterval::from_secs(4), exec_frame);
    let _ = CHRONOGRAPHER_SCHEDULER.schedule(&task).await;
    CHRONOGRAPHER_SCHEDULER.start().await;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
