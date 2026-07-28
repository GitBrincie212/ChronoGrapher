use async_trait::async_trait;
use chronographer::prelude::*;
use chronographer::task::TaskHookContext;

pub struct MyTaskHook;

#[async_trait]
impl<E: TaskHookEvent> TaskHook<E> for MyTaskHook {
    async fn on_event(&self, _ctx: &TaskHookContext, _payload: &E::Payload<'_>) {
        todo!()
    }
}

#[task(schedule = every!(2s))]
pub async fn MyTask(_ctx: &TaskFrameContext) -> Result<(), String> {
    todo!()
}

#[chronographer::main]
pub async fn main(scheduler: DefaultLiveScheduler<String>) {
    let _inst = MyTask::instance();
    todo!()
}
