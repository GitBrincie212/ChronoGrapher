use async_trait::async_trait;
use chronographer::prelude::*;
use chronographer::task::SharedHandle;
use chronographer::task::TaskFrame;
use chronographer::task::TaskScheduleImmediate;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test]
async fn test_shared_creates_and_retrieves_same_instance() {
    let result = Arc::new(AtomicUsize::new(0));

    struct TestFrame {
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for TestFrame {
        async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
            let handle1 = ctx.shared(|| 42i32);
            let handle2 = ctx.shared(|| 100i32);

            *handle1.write().unwrap() = 10;

            if *handle2.read().unwrap() == 10 {
                self.result.store(1, Ordering::SeqCst);
            }

            Ok(())
        }
    }

    let frame = TestFrame {
        result: result.clone(),
    };
    let task = Task::simple(TaskScheduleImmediate, frame);

    task.as_erased().run().await.unwrap();

    assert_eq!(
        result.load(Ordering::SeqCst),
        1,
        "Should retrieve the same shared instance"
    );
}

#[tokio::test]
async fn test_shared_read_and_write() {
    let result = Arc::new(AtomicUsize::new(0));

    struct TestFrame {
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for TestFrame {
        async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
            let handle = ctx.shared(|| AtomicUsize::new(0));

            handle.write().unwrap().fetch_add(5, Ordering::SeqCst);
            if handle.read().unwrap().load(Ordering::SeqCst) == 5 {
                handle.write().unwrap().fetch_add(3, Ordering::SeqCst);
                if handle.read().unwrap().load(Ordering::SeqCst) == 8 {
                    self.result.store(1, Ordering::SeqCst);
                }
            }

            Ok(())
        }
    }

    let frame = TestFrame {
        result: result.clone(),
    };
    let task = Task::simple(TaskScheduleImmediate, frame);

    task.as_erased().run().await.unwrap();

    assert_eq!(
        result.load(Ordering::SeqCst),
        1,
        "Should read and write correctly"
    );
}

#[tokio::test]
async fn test_shared_async_creates_and_retrieves_same_instance() {
    let result = Arc::new(AtomicUsize::new(0));

    struct TestFrame {
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for TestFrame {
        async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
            let handle1 = ctx.shared_async(|| async { 42i32 }).await;
            let handle2 = ctx.shared_async(|| async { 100i32 }).await;

            *handle1.write().unwrap() = 10;

            if *handle2.read().unwrap() == 10 {
                self.result.store(1, Ordering::SeqCst);
            }

            Ok(())
        }
    }

    let frame = TestFrame {
        result: result.clone(),
    };
    let task = Task::simple(TaskScheduleImmediate, frame);

    task.as_erased().run().await.unwrap();

    assert_eq!(
        result.load(Ordering::SeqCst),
        1,
        "Should retrieve the same async shared instance"
    );
}

#[tokio::test]
async fn test_get_shared_returns_none_when_not_exists() {
    let result = Arc::new(AtomicUsize::new(0));

    struct TestFrame {
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for TestFrame {
        async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
            let handle: Option<SharedHandle<i32>> = ctx.get_shared();

            if handle.is_none() {
                self.result.store(1, Ordering::SeqCst);
            }

            Ok(())
        }
    }

    let frame = TestFrame {
        result: result.clone(),
    };
    let task = Task::simple(TaskScheduleImmediate, frame);

    task.as_erased().run().await.unwrap();

    assert_eq!(
        result.load(Ordering::SeqCst),
        1,
        "Should return None when shared data doesn't exist"
    );
}

#[tokio::test]
async fn test_get_shared_returns_some_when_exists() {
    let result = Arc::new(AtomicUsize::new(0));

    struct TestFrame {
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for TestFrame {
        async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
            ctx.shared(|| 42i32);

            let handle: Option<SharedHandle<i32>> = ctx.get_shared();

            if let Some(h) = handle
                && *h.read().unwrap() == 42
            {
                self.result.store(1, Ordering::SeqCst);
            }

            Ok(())
        }
    }

    let frame = TestFrame {
        result: result.clone(),
    };
    let task = Task::simple(TaskScheduleImmediate, frame);

    task.as_erased().run().await.unwrap();

    assert_eq!(
        result.load(Ordering::SeqCst),
        1,
        "Should return Some when shared data exists"
    );
}

#[tokio::test]
async fn test_shared_data_isolated_by_type() {
    let result = Arc::new(AtomicUsize::new(0));

    struct TestFrame {
        result: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TaskFrame for TestFrame {
        async fn execute(&self, ctx: &TaskContext) -> Result<(), TaskError> {
            let int_handle = ctx.shared(|| 42i32);
            let str_handle = ctx.shared(|| String::from("hello"));

            if *int_handle.read().unwrap() == 42 && *str_handle.read().unwrap() == "hello" {
                *int_handle.write().unwrap() = 100;

                if *int_handle.read().unwrap() == 100 && *str_handle.read().unwrap() == "hello" {
                    self.result.store(1, Ordering::SeqCst);
                }
            }

            Ok(())
        }
    }

    let frame = TestFrame {
        result: result.clone(),
    };
    let task = Task::simple(TaskScheduleImmediate, frame);

    task.as_erased().run().await.unwrap();

    assert_eq!(
        result.load(Ordering::SeqCst),
        1,
        "Should isolate shared data by type"
    );
}
