use crate::util::{noop_erased, runtime, BenchConfig};
use chronographer::scheduler::task_dispatcher::{DefaultTaskDispatcher, SchedulerTaskDispatcher};
use chronographer::scheduler::task_store::{EphemeralSchedulerTaskStore, SchedulerTaskStore};

type Dispatcher = DefaultTaskDispatcher<BenchConfig>;
type Store = EphemeralSchedulerTaskStore<BenchConfig>;

#[divan::bench]
fn dispatch(bencher: divan::Bencher) {
    let dispatcher = Dispatcher::default();
    let store = Store::default();
    let key = store.store(noop_erased()).unwrap();
    let task = noop_erased();
    bencher.bench(|| {
        runtime()
            .block_on(dispatcher.dispatch(&key, task.clone()))
            .unwrap();
    });
}

#[divan::bench]
fn dispatch_cancel(bencher: divan::Bencher) {
    let dispatcher = Dispatcher::default();
    let store = Store::default();
    let key = store.store(noop_erased()).unwrap();
    bencher.bench(|| {
        runtime().block_on(async {
            dispatcher.dispatch(&key, noop_erased()).await.unwrap();
            dispatcher.cancel(&key).await;
        });
    });
}
