use chronographer::prelude::*;

#[taskframe(__internal_workflow_spec = (
    retry(3),
    timeout(5s)
))]
pub async fn MyTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
    todo!()
}

#[chronographer::main]
pub async fn main(scheduler: DefaultLiveScheduler<String>) {
    todo!()
}
