use async_trait::async_trait;
use chronographer::prelude::*;
use chronographer::task::TaskFrame;
use chronographer::utils::SharedHook;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

struct MyTaskFrameA(pub Arc<MyTaskFrameB>, AtomicUsize);
struct MyTaskFrameB;

#[async_trait]
impl TaskFrame for MyTaskFrameA {
    async fn execute(&self, ctx: &TaskContext) -> Result<(), DynArcError> {
        self.1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let hook = Arc::new(SharedHook(Arc::new(format!(
            "Hello World {}",
            self.1.load(std::sync::atomic::Ordering::Relaxed)
        ))));
        ctx.attach_hook::<(), SharedHook<Arc<String>>>(hook.clone())
            .await;
        println!("Sharing data {:?}", hook.0.as_ref());
        ctx.subdivide(self.0.clone()).await?;
        ctx.detach_hook::<(), SharedHook<Arc<String>>>().await;
        Ok(())
    }
}

#[async_trait]
impl TaskFrame for MyTaskFrameB {
    async fn execute(&self, ctx: &TaskContext) -> Result<(), DynArcError> {
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
    let mytask = Task::simple(
        TaskScheduleInterval::from_secs(1),
        MyTaskFrameA(Arc::new(MyTaskFrameB), AtomicUsize::new(0)),
    );

    let _ = CHRONOGRAPHER_SCHEDULER.schedule(&mytask).await;
    CHRONOGRAPHER_SCHEDULER.start().await;
    loop {}
}
