use crate::util::{noop_task, runtime};
use chronographer::task::{
    OnTaskEnd, OnTaskStart, TaskHook, TaskHookContext, TaskHookEvent, TaskScheduleImmediate,
};
use async_trait::async_trait;
use std::sync::Arc;

struct NoopStartHook;

#[async_trait]
impl TaskHook<OnTaskStart> for NoopStartHook {
    async fn on_event(
        &self,
        _ctx: &TaskHookContext,
        _payload: &<OnTaskStart as TaskHookEvent>::Payload<'_>,
    ) {
    }
}

struct NoopEndHook;

#[async_trait]
impl TaskHook<OnTaskEnd> for NoopEndHook {
    async fn on_event(
        &self,
        _ctx: &TaskHookContext,
        _payload: &<OnTaskEnd as TaskHookEvent>::Payload<'_>,
    ) {
    }
}

#[divan::bench]
fn run_noop(bencher: divan::Bencher) {
    let task = noop_task(TaskScheduleImmediate).into_erased();
    bencher.bench(|| {
        runtime().block_on(task.run()).unwrap();
    });
}

#[divan::bench(args = [0usize, 1, 2, 3])]
fn run_with_hooks(bencher: divan::Bencher, hooks: usize) {
    let task = noop_task(TaskScheduleImmediate).into_erased();
    runtime().block_on(async {
        for _ in 0..hooks {
            task.attach_hook::<OnTaskStart>(Arc::new(NoopStartHook)).await;
            task.attach_hook::<OnTaskEnd>(Arc::new(NoopEndHook)).await;
        }
    });
    bencher.bench(|| {
        runtime().block_on(task.run()).unwrap();
    });
}

#[divan::bench]
fn emit_empty(bencher: divan::Bencher) {
    let task = noop_task(TaskScheduleImmediate);
    bencher.bench(|| {
        runtime().block_on(task.emit_hook_event::<OnTaskStart>(&()));
    });
}

#[divan::bench]
fn emit_single(bencher: divan::Bencher) {
    let task = noop_task(TaskScheduleImmediate);
    runtime().block_on(task.attach_hook::<OnTaskStart>(Arc::new(NoopStartHook)));
    bencher.bench(|| {
        runtime().block_on(task.emit_hook_event::<OnTaskStart>(&()));
    });
}

#[divan::bench(args = [1usize, 2, 3, 10])]
fn emit_many(bencher: divan::Bencher, hooks: usize) {
    let task = noop_task(TaskScheduleImmediate);
    runtime().block_on(async {
        for _ in 0..hooks {
            task.attach_hook::<OnTaskStart>(Arc::new(NoopStartHook)).await;
        }
    });
    bencher.bench(|| {
        runtime().block_on(task.emit_hook_event::<OnTaskStart>(&()));
    });
}

#[divan::bench]
fn attach(bencher: divan::Bencher) {
    bencher.bench(|| {
        let task = noop_task(TaskScheduleImmediate);
        runtime().block_on(task.attach_hook::<OnTaskStart>(Arc::new(NoopStartHook)));
    });
}

#[divan::bench]
fn attach_detach(bencher: divan::Bencher) {
    bencher.bench(|| {
        let task = noop_task(TaskScheduleImmediate);
        runtime().block_on(async {
            task.attach_hook::<OnTaskStart>(Arc::new(NoopStartHook))
                .await;
            task.detach_hook::<OnTaskStart, NoopStartHook>().await;
        });
    });
}

#[divan::bench]
fn get(bencher: divan::Bencher) {
    let task = noop_task(TaskScheduleImmediate);
    runtime().block_on(task.attach_hook::<OnTaskStart>(Arc::new(NoopStartHook)));
    bencher.bench(|| {
        divan::black_box(task.get_hook::<OnTaskStart, NoopStartHook>().is_some());
    });
}
