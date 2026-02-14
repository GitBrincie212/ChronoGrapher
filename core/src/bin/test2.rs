use std::io::ErrorKind;
use async_trait::async_trait;
use chronographer::prelude::*;
use chronographer::task::TaskHookContext;
use std::sync::Arc;
use chronographer::scheduler::{DefaultSchedulerConfig, Scheduler};

pub struct MyCoolTaskHook;

#[async_trait]
impl TaskHook<OnTaskStart> for MyCoolTaskHook {
    async fn on_event(
        &self,
        _ctx: &TaskHookContext,
        _payload: &<OnTaskStart as TaskHookEvent>::Payload<'_>,
    ) {
        println!("Interested event triggered!");
    }
}

#[async_trait]
impl TaskHook<OnTaskEnd> for MyCoolTaskHook {
    async fn on_event(
        &self,
        _ctx: &TaskHookContext,
        _payload: &<OnTaskEnd as TaskHookEvent>::Payload<'_>,
    ) {
        println!("Interested event triggered!");
    }
}

#[tokio::main]
async fn main() {
    let scheduler = Scheduler::<DefaultSchedulerConfig<DynArcError>>::default();
    
    let exec_frame = DynamicTaskFrame::new(|_ctx| async {
        println!("Trying primary task...");
        //sleep(Duration::from_secs_f64(1.234)).await;
        Err(Arc::new(std::io::Error::new(ErrorKind::Other, "uh oh")) as DynArcError)
    });

    //let timeout_frame = DelayTaskFrame::new(exec_frame, Duration::from_secs(3));

    let task = Task::simple(TaskScheduleInterval::from_secs(4), exec_frame);
    let _ = scheduler.schedule(&task).await;
    scheduler.start().await;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
