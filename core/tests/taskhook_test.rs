use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use chronographer::errors::StandardCoreErrorsCG;
use chronographer::prelude::*;
use chronographer::task::{TaskFrame, TaskFrameContext, TaskHookContext, TaskScheduleImmediate};

type OnTaskStartPayload<'a> = <OnTaskStart as TaskHookEvent>::Payload<'a>;
type OnTaskEndPayload<'a> = <OnTaskEnd as TaskHookEvent>::Payload<'a>;

struct OnStartCountingHook {
    count: Arc<AtomicUsize>,
}

#[async_trait]
impl TaskHook<OnTaskStart> for OnStartCountingHook {
    async fn on_event(
        &self,
        _ctx: &TaskHookContext,
        _payload: &OnTaskStartPayload<'_>,
    ) {
        self.count.fetch_add(1, Ordering::SeqCst);
    }
}

struct OnEndCountingHook {
    count: Arc<AtomicUsize>,
}

#[async_trait]
impl TaskHook<OnTaskEnd> for OnEndCountingHook {
    async fn on_event(
        &self,
        _ctx: &TaskHookContext,
        _payload: &OnTaskEndPayload<'_>,
    ) {
        self.count.fetch_add(1, Ordering::SeqCst);
    }
}

struct SimpleTaskFrame {
    should_succeed: Arc<AtomicBool>,
}

#[async_trait]
impl TaskFrame for SimpleTaskFrame {
    type Error = DynArcError;

    async fn execute(&self, _ctx: &TaskFrameContext) -> Result<(), Self::Error> {
        if self.should_succeed.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(Arc::new(StandardCoreErrorsCG::TaskDependenciesUnresolved))
        }
    }
}

#[tokio::test]
async fn test_attach_and_trigger_hooks() {
    let on_start_count = Arc::new(AtomicUsize::new(0));
    let on_end_count = Arc::new(AtomicUsize::new(0));

    let hook_start = Arc::new(OnStartCountingHook {
        count: on_start_count.clone(),
    });

    let hook_end = Arc::new(OnEndCountingHook {
        count: on_end_count.clone(),
    });

    let should_succeed = Arc::new(AtomicBool::new(true));
    let frame = SimpleTaskFrame {
        should_succeed: should_succeed.clone(),
    };

    let task = Task::new(TaskScheduleImmediate, frame);

    task.attach_hook::<OnTaskStart>(hook_start).await;
    task.attach_hook::<OnTaskEnd>(hook_end).await;

    task.emit_hook_event::<OnTaskStart>(&()).await;
    assert_eq!(
        on_start_count.load(Ordering::SeqCst),
        1,
        "OnTaskStart hook should fire once"
    );

    let no_error: Option<DynArcError> = None;
    task.emit_hook_event::<OnTaskEnd>(&no_error.as_ref().map(|x| x.as_ref())).await;
    assert_eq!(
        on_end_count.load(Ordering::SeqCst),
        1,
        "OnTaskEnd hook should fire once"
    );
}

#[tokio::test]
async fn test_detach_hooks() {
    let on_start_count = Arc::new(AtomicUsize::new(0));

    let hook = Arc::new(OnStartCountingHook {
        count: on_start_count.clone(),
    });

    let should_succeed = Arc::new(AtomicBool::new(true));
    let frame = SimpleTaskFrame { should_succeed };

    let task = Task::new(TaskScheduleImmediate, frame);

    task.attach_hook::<OnTaskStart>(hook.clone()).await;

    task.emit_hook_event::<OnTaskStart>(&()).await;
    assert_eq!(
        on_start_count.load(Ordering::SeqCst),
        1,
        "Hook should fire once before detach"
    );

    task.detach_hook::<OnTaskStart, OnStartCountingHook>().await;

    task.emit_hook_event::<OnTaskStart>(&()).await;

    assert_eq!(
        on_start_count.load(Ordering::SeqCst),
        1,
        "Hook should be detached, count should not increment"
    );
}

#[tokio::test]
async fn test_get_hook() {
    let on_start_count = Arc::new(AtomicUsize::new(0));

    let hook = Arc::new(OnStartCountingHook {
        count: on_start_count.clone(),
    });

    let should_succeed = Arc::new(AtomicBool::new(true));
    let frame = SimpleTaskFrame { should_succeed };

    let task = Task::new(TaskScheduleImmediate, frame);

    assert!(
        task.get_hook::<OnTaskStart, OnStartCountingHook>()
            .is_none(),
        "Hook should not exist before attach"
    );

    task.attach_hook::<OnTaskStart>(hook).await;

    let retrieved_hook = task.get_hook::<OnTaskStart, OnStartCountingHook>();
    assert!(retrieved_hook.is_some(), "Hook should exist after attach");

    task.emit_hook_event::<OnTaskStart>(&()).await;
    assert_eq!(
        on_start_count.load(Ordering::SeqCst),
        1,
        "Retrieved hook should work"
    );
}
