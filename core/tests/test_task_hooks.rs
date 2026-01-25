use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use chronographer::prelude::*;
use chronographer::task::{TaskFrame, TaskHookContext, TaskScheduleImmediate};
use chronographer::errors::ChronographerErrors;

type OnTaskStartPayload = <OnTaskStart as TaskHookEvent>::Payload;
type OnTaskEndPayload = <OnTaskEnd as TaskHookEvent>::Payload;

struct OnStartCountingHook {
    count: Arc<AtomicUsize>,
}

#[async_trait]
impl TaskHook<OnTaskStart> for OnStartCountingHook {
    async fn on_event(&self, _event: OnTaskStart, _ctx: &TaskHookContext, _payload: &OnTaskStartPayload) {
        self.count.fetch_add(1, Ordering::SeqCst);
    }
}

struct OnEndCountingHook {
    count: Arc<AtomicUsize>,
}

#[async_trait]
impl TaskHook<OnTaskEnd> for OnEndCountingHook {
    async fn on_event(&self, _event: OnTaskEnd, _ctx: &TaskHookContext, _payload: &OnTaskEndPayload) {
        self.count.fetch_add(1, Ordering::SeqCst);
    }
}

struct SimpleTaskFrame {
    should_succeed: Arc<AtomicBool>,
}

#[async_trait]
impl TaskFrame for SimpleTaskFrame {
    async fn execute(&self, _ctx: &TaskContext) -> Result<(), TaskError> {
        if self.should_succeed.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(Arc::new(ChronographerErrors::TaskDependenciesUnresolved))
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

    let task = Task::simple(TaskScheduleImmediate, frame);

    task.attach_hook::<OnTaskStart>(hook_start).await;
    task.attach_hook::<OnTaskEnd>(hook_end).await;

    task.emit_hook_event::<OnTaskStart>(&()).await;
    assert_eq!(on_start_count.load(Ordering::SeqCst), 1, "OnTaskStart hook should fire once");

    let no_error: Option<TaskError> = None;
    task.emit_hook_event::<OnTaskEnd>(&no_error).await;
    assert_eq!(on_end_count.load(Ordering::SeqCst), 1, "OnTaskEnd hook should fire once");
}

#[tokio::test]
async fn test_detach_hooks() {
    let on_start_count = Arc::new(AtomicUsize::new(0));

    let hook = Arc::new(OnStartCountingHook {
        count: on_start_count.clone(),
    });

    let should_succeed = Arc::new(AtomicBool::new(true));
    let frame = SimpleTaskFrame {
        should_succeed,
    };

    let task = Task::simple(TaskScheduleImmediate, frame);

    task.attach_hook::<OnTaskStart>(hook.clone()).await;
    
    task.emit_hook_event::<OnTaskStart>(&()).await;
    assert_eq!(on_start_count.load(Ordering::SeqCst), 1, "Hook should fire once before detach");

    // Try to detach - this will panic due to type erasure bug
    // The hook is stored as Arc<ErasedTaskHookWrapper<OnTaskStart>>
    // But detach() tries to downcast to Arc<OnStartCountingHook>
    // These have different TypeIds, so the downcast fails
    task.detach_hook::<OnTaskStart, OnStartCountingHook>().await;
    
    task.emit_hook_event::<OnTaskStart>(&()).await;
    
    assert_eq!(on_start_count.load(Ordering::SeqCst), 1, "Hook should be detached, count should not increment");
}

#[tokio::test]
async fn test_get_hook() {
    let on_start_count = Arc::new(AtomicUsize::new(0));

    let hook = Arc::new(OnStartCountingHook {
        count: on_start_count.clone(),
    });

    let should_succeed = Arc::new(AtomicBool::new(true));
    let frame = SimpleTaskFrame {
        should_succeed,
    };

    let task = Task::simple(TaskScheduleImmediate, frame);

    assert!(task.get_hook::<OnTaskStart, OnStartCountingHook>().is_none(), "Hook should not exist before attach");

    task.attach_hook::<OnTaskStart>(hook).await;

    let retrieved_hook = task.get_hook::<OnTaskStart, OnStartCountingHook>();
    // This will fail due to the same type erasure issue as detach
    // The hook is stored as ErasedTaskHookWrapper<OnTaskStart>, not as OnStartCountingHook
    // So the downcast in get() fails and returns None
    // However, the hook still works when emitting events because emit() doesn't need the concrete type
    assert!(retrieved_hook.is_some(), "Hook should exist after attach");

    task.emit_hook_event::<OnTaskStart>(&()).await;
    assert_eq!(on_start_count.load(Ordering::SeqCst), 1, "Retrieved hook should work");
}