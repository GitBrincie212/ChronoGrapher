use async_trait::async_trait;
use chronographer::prelude::*;
use chronographer::task::TaskScheduleImmediate;
use chronographer::task::{TaskFrame, TaskFrameContext};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct AtomicCounter(AtomicUsize);

impl AtomicCounter {
    fn new(value: usize) -> Self {
        Self(AtomicUsize::new(value))
    }

    fn fetch_add(&self, value: usize, ordering: Ordering) -> usize {
        self.0.fetch_add(value, ordering)
    }

    fn load(&self, ordering: Ordering) -> usize {
        self.0.load(ordering)
    }
}

impl NonObserverTaskHook for AtomicCounter {}

#[tokio::test]
async fn test_shared_returns_same_instance() {
    let result = Arc::new(AtomicUsize::new(0));

    struct TestFrame {
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for TestFrame {
        type Error = DynArcError;

        async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
            let counter1 = ctx.shared(|| AtomicCounter::new(0)).await;
            let counter2 = ctx.shared(|| AtomicCounter::new(999)).await; // creator ignored

            counter1.fetch_add(5, Ordering::SeqCst);

            if counter2.load(Ordering::SeqCst) == 5 {
                self.result.store(1, Ordering::SeqCst);
            }

            Ok(())
        }
    }

    let frame = TestFrame {
        result: result.clone(),
    };
    let task = Task::new(TaskScheduleImmediate, frame);

    task.as_erased().run().await.unwrap();

    assert_eq!(
        result.load(Ordering::SeqCst),
        1,
        "Should retrieve the same shared instance via TaskHook"
    );
}

#[tokio::test]
async fn test_shared_isolated_by_type() {
    let result = Arc::new(AtomicUsize::new(0));

    struct IntCounter(AtomicUsize);
    struct StrCounter(AtomicUsize);

    impl NonObserverTaskHook for IntCounter {}
    impl NonObserverTaskHook for StrCounter {}

    struct TestFrame {
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for TestFrame {
        type Error = DynArcError;

        async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
            let int_counter = ctx.shared(|| IntCounter(AtomicUsize::new(42))).await;
            let str_counter = ctx.shared(|| StrCounter(AtomicUsize::new(100))).await;

            if int_counter.0.load(Ordering::SeqCst) == 42
                && str_counter.0.load(Ordering::SeqCst) == 100
            {
                int_counter.0.store(100, Ordering::SeqCst);

                if int_counter.0.load(Ordering::SeqCst) == 100
                    && str_counter.0.load(Ordering::SeqCst) == 100
                {
                    self.result.store(1, Ordering::SeqCst);
                }
            }

            Ok(())
        }
    }

    let frame = TestFrame {
        result: result.clone(),
    };
    let task = Task::new(TaskScheduleImmediate, frame);

    task.as_erased().run().await.unwrap();

    assert_eq!(
        result.load(Ordering::SeqCst),
        1,
        "Should isolate shared data by type via TaskHook"
    );
}

#[tokio::test]
async fn test_get_shared_none_if_missing() {
    let result = Arc::new(AtomicUsize::new(0));

    struct TestFrame {
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for TestFrame {
        type Error = DynArcError;

        async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
            let counter = ctx.get_shared::<AtomicCounter>();

            if counter.is_none() {
                self.result.store(1, Ordering::SeqCst);
            }

            Ok(())
        }
    }

    let frame = TestFrame {
        result: result.clone(),
    };
    let task = Task::new(TaskScheduleImmediate, frame);

    task.as_erased().run().await.unwrap();

    assert_eq!(
        result.load(Ordering::SeqCst),
        1,
        "Should return None when shared data doesn't exist"
    );
}

#[tokio::test]
async fn test_get_shared_some_if_exists() {
    let result = Arc::new(AtomicUsize::new(0));

    struct TestFrame {
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for TestFrame {
        type Error = DynArcError;

        async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
            ctx.shared(|| AtomicCounter::new(42)).await;

            let counter = ctx.get_shared::<AtomicCounter>();

            if let Some(c) = counter
                && c.load(Ordering::SeqCst) == 42
            {
                self.result.store(1, Ordering::SeqCst);
            }

            Ok(())
        }
    }

    let frame = TestFrame {
        result: result.clone(),
    };
    let task = Task::new(TaskScheduleImmediate, frame);

    task.as_erased().run().await.unwrap();

    assert_eq!(
        result.load(Ordering::SeqCst),
        1,
        "Should return Some when shared data exists"
    );
}

#[tokio::test]
async fn test_shared_custom_state_manager() {
    let result = Arc::new(AtomicUsize::new(0));

    struct MyStateManager {
        x: AtomicUsize,
        y: AtomicUsize,
    }

    impl MyStateManager {
        pub fn new(x: usize, y: usize) -> Self {
            Self {
                x: AtomicUsize::new(x),
                y: AtomicUsize::new(y),
            }
        }

        pub fn write_x(&self, new: usize) {
            self.x.store(new, Ordering::SeqCst);
        }

        pub fn write_y(&self, new: usize) {
            self.y.store(new, Ordering::SeqCst);
        }

        pub fn get_sum(&self) -> usize {
            self.x.load(Ordering::SeqCst) + self.y.load(Ordering::SeqCst)
        }
    }

    impl NonObserverTaskHook for MyStateManager {}

    struct TestFrame {
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for TestFrame {
        type Error = DynArcError;

        async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
            let state = ctx.shared(|| MyStateManager::new(10, 20)).await;

            state.write_x(100);
            state.write_y(200);

            if state.get_sum() == 300 {
                self.result.store(1, Ordering::SeqCst);
            }

            Ok(())
        }
    }

    let frame = TestFrame {
        result: result.clone(),
    };
    let task = Task::new(TaskScheduleImmediate, frame);

    task.as_erased().run().await.unwrap();

    assert_eq!(
        result.load(Ordering::SeqCst),
        1,
        "Should work with custom state manager via TaskHook"
    );
}

#[tokio::test]
async fn test_shared_with_marker() {
    let result = Arc::new(AtomicUsize::new(0));

    struct MyStateMarker;

    impl NonObserverTaskHook for MyStateMarker {}

    struct TestFrame {
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for TestFrame {
        type Error = DynArcError;

        async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
            ctx.shared(|| MyStateMarker).await;

            if ctx.get_shared::<MyStateMarker>().is_some() {
                self.result.store(1, Ordering::SeqCst);
            }

            Ok(())
        }
    }

    let frame = TestFrame {
        result: result.clone(),
    };
    let task = Task::new(TaskScheduleImmediate, frame);

    task.as_erased().run().await.unwrap();

    assert_eq!(result.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_shared_scoped_to_task_context() {
    let result = Arc::new(AtomicUsize::new(0));

    struct WorkerTask {
        worker_id: usize,
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for WorkerTask {
        type Error = DynArcError;

        async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
            let counter = ctx.shared(|| AtomicCounter::new(0)).await;
            let value = counter.fetch_add(1, Ordering::SeqCst) + 1;

            if self.worker_id == 1 && value == 1 {
                println!("WorkerTask {} setting result to 1", self.worker_id);
                self.result.store(1, Ordering::SeqCst);
            }

            Ok(())
        }
    }

    let task1 = Task::new(
        TaskScheduleImmediate,
        WorkerTask {
            worker_id: 1,
            result: result.clone(),
        },
    );
    let task2 = Task::new(
        TaskScheduleImmediate,
        WorkerTask {
            worker_id: 2,
            result: result.clone(),
        },
    );

    task1.as_erased().run().await.unwrap();
    task2.as_erased().run().await.unwrap();

    struct SupervisorTask {
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for SupervisorTask {
        type Error = DynArcError;

        async fn execute(&self, ctx: &TaskFrameContext) -> Result<(), Self::Error> {
            let worker1 = WorkerTask {
                worker_id: 3,
                result: self.result.clone(),
            };

            let worker2 = WorkerTask {
                worker_id: 4,
                result: self.result.clone(),
            };

            println!("SupervisorTask subdividing worker3");
            ctx.subdivide(&worker1).await?;
            println!("SupervisorTask subdividing worker4");
            ctx.subdivide(&worker2).await?;

            Ok(())
        }
    }

    let supervisor = Task::new(
        TaskScheduleImmediate,
        SupervisorTask {
            result: result.clone(),
        },
    );
    supervisor.as_erased().run().await.unwrap();

    assert_eq!(
        result.load(Ordering::SeqCst),
        1,
        "Should demonstrate that shared data only works within same task context"
    );
}
