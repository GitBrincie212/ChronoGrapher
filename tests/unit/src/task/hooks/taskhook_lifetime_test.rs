use async_trait::async_trait;
use chronographer::task::{
    NoOperationTaskFrame, OnTaskStart, Task, TaskHook, TaskScheduleImmediate,
};
use chronographer::task::{OnTaskEnd, TaskHookContext, TaskHookEvent};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Notify;
use tokio::time::{Duration, timeout};

struct LifetimeHook;

#[async_trait]
impl TaskHook<OnTaskStart> for LifetimeHook {}

#[async_trait]
impl TaskHook<OnTaskEnd> for LifetimeHook {}

fn task() -> Task<NoOperationTaskFrame<String>> {
    Task::new(NoOperationTaskFrame::default(), TaskScheduleImmediate)
}

#[tokio::test]
async fn dropping_task_releases_its_hook() {
    let task = task();
    let hook = Arc::new(LifetimeHook);
    let weak = Arc::downgrade(&hook);

    task.attach_hook::<OnTaskStart>(hook).await;
    assert!(weak.upgrade().is_some(), "the task must retain its hook");

    drop(task);

    assert!(
        weak.upgrade().is_none(),
        "issue #140: the global registry must release hooks when their task drops"
    );
}

#[tokio::test]
async fn erasing_task_preserves_hooks_until_last_owner_drops() {
    let task = task();
    let hook = Arc::new(LifetimeHook);
    let weak = Arc::downgrade(&hook);
    task.attach_hook::<OnTaskStart>(hook).await;

    let erased = Arc::new(task.into_erased());
    let other_owner = erased.clone();
    assert!(
        erased.get_hook::<OnTaskStart, LifetimeHook>().is_some(),
        "type erasure must preserve hook registration"
    );

    drop(erased);
    assert!(
        weak.upgrade().is_some(),
        "another task owner is still alive"
    );

    drop(other_owner);
    assert!(weak.upgrade().is_none(), "the last task owner has dropped");
}

#[tokio::test]
async fn dropping_one_task_preserves_a_hook_shared_with_another_task() {
    let first = task();
    let second = task();
    let hook = Arc::new(LifetimeHook);
    let weak = Arc::downgrade(&hook);
    first.attach_hook::<OnTaskStart>(hook.clone()).await;
    second.attach_hook::<OnTaskStart>(hook).await;

    drop(first);
    assert!(
        weak.upgrade().is_some(),
        "the second task still owns the hook"
    );
    assert!(second.get_hook::<OnTaskStart, LifetimeHook>().is_some());

    drop(second);
    assert!(weak.upgrade().is_none(), "both tasks have dropped");
}

#[tokio::test]
async fn dropping_task_releases_all_events_and_stacked_instances() {
    let task = task();
    let first = Arc::new(LifetimeHook);
    let second = Arc::new(LifetimeHook);
    let first_weak = Arc::downgrade(&first);
    let second_weak = Arc::downgrade(&second);
    task.attach_hook::<OnTaskStart>(first.clone()).await;
    task.attach_hook::<OnTaskEnd>(first).await;
    task.attach_hook::<OnTaskStart>(second).await;

    drop(task);
    assert!(first_weak.upgrade().is_none());
    assert!(second_weak.upgrade().is_none());
}

#[tokio::test]
async fn dropping_task_does_not_destroy_a_user_owned_hook() {
    let task = task();
    let hook = Arc::new(LifetimeHook);
    let weak = Arc::downgrade(&hook);
    task.attach_hook::<OnTaskStart>(hook.clone()).await;
    drop(task);
    assert_eq!(
        Arc::strong_count(&hook),
        1,
        "only the user should retain the hook"
    );
    drop(hook);
    assert!(weak.upgrade().is_none());
}

#[derive(Default)]
struct BorrowedEvent;

impl TaskHookEvent for BorrowedEvent {
    type Payload<'a> = &'a str;
}

struct BorrowingHook {
    owner: std::sync::Weak<Task<NoOperationTaskFrame<String>>>,
    seen: Mutex<String>,
}

#[async_trait]
impl TaskHook<BorrowedEvent> for BorrowingHook {
    async fn on_event(&self, _: &TaskHookContext, payload: &&str) {
        tokio::task::yield_now().await;
        assert!(self.owner.upgrade().is_some());
        self.seen.lock().unwrap().push_str(payload);
    }
}

#[tokio::test]
async fn borrowed_event_survives_await_and_weak_back_reference_allows_cleanup() {
    let task = Arc::new(task());
    let hook = Arc::new(BorrowingHook {
        owner: Arc::downgrade(&task),
        seen: Mutex::new(String::new()),
    });
    task.attach_hook::<BorrowedEvent>(hook.clone()).await;
    let payload = String::from("borrowed from this stack");
    task.emit_hook_event::<BorrowedEvent>(&payload.as_str())
        .await;
    assert_eq!(*hook.seen.lock().unwrap(), payload);

    drop(task);
    assert!(hook.owner.upgrade().is_none());
    assert_eq!(Arc::strong_count(&hook), 1);
}

struct CaptureContext(Arc<Mutex<Option<TaskHookContext>>>);

#[async_trait]
impl TaskHook<OnTaskStart> for CaptureContext {
    async fn on_event(
        &self,
        ctx: &TaskHookContext,
        _payload: &<OnTaskStart as TaskHookEvent>::Payload<'_>,
    ) {
        *self.0.lock().unwrap() = Some(*ctx);
    }
}

async fn context(task: &Task<NoOperationTaskFrame<String>>) -> TaskHookContext {
    let output = Arc::new(Mutex::new(None));
    task.attach_hook::<OnTaskStart>(Arc::new(CaptureContext(output.clone())))
        .await;
    task.emit_hook_event::<OnTaskStart>(&()).await;
    output.lock().unwrap().unwrap()
}

struct OwningHook {
    _owner: Arc<Task<NoOperationTaskFrame<String>>>,
}

impl TaskHook<OnTaskEnd> for OwningHook {}

#[tokio::test]
async fn retained_context_can_detach_and_break_a_strong_cycle() {
    let task = Arc::new(task());
    let ctx = context(&task).await;
    let weak = Arc::downgrade(&task);
    task.attach_hook::<OnTaskEnd>(Arc::new(OwningHook {
        _owner: task.clone(),
    }))
    .await;
    drop(task);
    assert!(weak.upgrade().is_some());

    ctx.detach_hook::<OnTaskEnd, OwningHook>().await;
    assert!(weak.upgrade().is_none());
}

#[tokio::test]
async fn retained_context_cannot_register_hooks_after_task_drop() {
    let task = task();
    let ctx = context(&task).await;
    drop(task);
    let hook = Arc::new(LifetimeHook);
    let weak = Arc::downgrade(&hook);
    ctx.attach_hook::<OnTaskStart>(hook).await;
    assert!(ctx.get_hook::<OnTaskStart, LifetimeHook>().is_none());
    assert!(
        weak.upgrade().is_none(),
        "a stale context must not recreate a registry entry"
    );
    ctx.emit::<OnTaskStart>(&()).await;
    ctx.detach_hook::<OnTaskStart, LifetimeHook>().await;
}

struct BlockingHook {
    entered: Arc<Notify>,
    resume: Arc<Notify>,
}

#[async_trait]
impl TaskHook<OnTaskStart> for BlockingHook {
    async fn on_event(
        &self,
        _ctx: &TaskHookContext,
        _payload: &<OnTaskStart as TaskHookEvent>::Payload<'_>,
    ) {
        self.entered.notify_one();
        self.resume.notified().await;
    }
}

struct CountingHook(Arc<AtomicUsize>);

#[async_trait]
impl TaskHook<OnTaskStart> for CountingHook {
    async fn on_event(
        &self,
        _ctx: &TaskHookContext,
        _payload: &<OnTaskStart as TaskHookEvent>::Payload<'_>,
    ) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn in_flight_emission_retains_hooks_until_completion_after_task_drop() {
    timeout(Duration::from_secs(5), async {
        let task = task();
        let ctx = context(&task).await;
        let entered = Arc::new(Notify::new());
        let resume = Arc::new(Notify::new());
        let blocking = Arc::new(BlockingHook {
            entered: entered.clone(),
            resume: resume.clone(),
        });
        let count = Arc::new(AtomicUsize::new(0));
        let counting = Arc::new(CountingHook(count.clone()));
        let weak = Arc::downgrade(&counting);
        task.attach_hook::<OnTaskStart>(blocking).await;
        task.attach_hook::<OnTaskStart>(counting).await;

        let emission = tokio::spawn(async move {
            ctx.emit::<OnTaskStart>(&()).await;
        });
        entered.notified().await;
        drop(task);
        assert!(ctx.get_hook::<OnTaskStart, CountingHook>().is_none());
        assert!(
            weak.upgrade().is_some(),
            "the in-flight snapshot must own its hooks"
        );

        resume.notify_one();
        emission.await.unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert!(
            weak.upgrade().is_none(),
            "the completed emission must release its snapshot"
        );
    })
    .await
    .expect("emission must complete without holding registry locks across callbacks");
}
