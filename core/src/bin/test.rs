use async_trait::async_trait;
use chronographer::prelude::*;
use chronographer::scheduler::{DefaultSchedulerConfig, Scheduler};
use chronographer::task::{TaskFrame, TaskFrameContext};
use chronographer::utils::SharedHook;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

struct MyTaskFrameA(pub MyTaskFrameB, AtomicUsize);
struct MyTaskFrameB;

#[async_trait]
impl TaskFrame for MyTaskFrameA {
    type Error = Box<dyn TaskError>;

    async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        self.1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let hook = Arc::new(SharedHook(Arc::new(format!(
            "Hello World {}",
            self.1.load(std::sync::atomic::Ordering::Relaxed)
        ))));
        ctx.attach_hook::<(), SharedHook<Arc<String>>>(hook.clone())
            .await;
        println!("Sharing data {:?}", hook.0.as_ref());
        ctx.subdivide(&self.0).await?;
        ctx.detach_hook::<(), SharedHook<Arc<String>>>().await;
        Ok(())
    }
}

#[async_trait]
impl TaskFrame for MyTaskFrameB {
    type Error = Box<dyn TaskError>;

    async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        println!(
            "{:?}",
            ctx.get_hook::<(), SharedHook<Arc<String>>>()
                .map(|x| x.0.clone())
        );
        if let Some(val) = ctx
            .get_hook::<(), SharedHook<Arc<String>>>()
            .map(|x| x.0.clone())
        {
            println!("Accessed shared value {:?}", val)
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let scheduler = Scheduler::<DefaultSchedulerConfig<Box<dyn TaskError>>>::default();

    let mytask = Task::new(
        TaskScheduleInterval::from_secs(1),
        MyTaskFrameA(MyTaskFrameB, AtomicUsize::new(0)),
    );

    let _ = scheduler.schedule(&mytask).await;
    scheduler.start().await;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
