use crate::util::{BenchError, NoopFrame};
use chronographer::task::dependency::FrameDependency;
use chronographer::task::{
    CollectionTaskFrame, ConditionalTaskFrame, DelayTaskFrame, DependencyTaskFrame,
    ErasedTaskFrame, FallbackTaskFrame, GroupedTaskFramesQuitOnFailure, NoOperationTaskFrame,
    RetriableTaskFrame, Task, TaskFrameBuilder, TaskScheduleImmediate, TimeoutTaskFrame,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

fn noop_frame() -> NoopFrame {
    NoOperationTaskFrame::<BenchError>::default()
}

#[divan::bench]
fn new_noop_frame_default(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(noop_frame());
    });
}

#[divan::bench]
fn new_task(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(Task::new(noop_frame(), TaskScheduleImmediate));
    });
}

#[divan::bench]
fn new_erased_task(bencher: divan::Bencher) {
    bencher.bench(|| {
        let task = Task::new(noop_frame(), TaskScheduleImmediate);
        divan::black_box(task.into_erased());
    });
}

#[divan::bench(args = [1usize, 2, 4, 8])]
fn new_retriable_instant(bencher: divan::Bencher, retries: usize) {
    bencher.bench(|| {
        divan::black_box(
            RetriableTaskFrame::builder()
                .frame(noop_frame())
                .retries(NonZeroU32::new(divan::black_box(retries) as u32).unwrap())
                .build(),
        );
    });
}

#[divan::bench(args = [1usize, 2, 4, 8])]
fn new_retriable_constant(bencher: divan::Bencher, retries: usize) {
    bencher.bench(|| {
        divan::black_box(
            RetriableTaskFrame::builder()
                .frame(noop_frame())
                .retries(NonZeroU32::new(divan::black_box(retries) as u32).unwrap())
                .constant(Duration::from_millis(100))
                .build(),
        );
    });
}

#[divan::bench]
fn new_retriable_linear(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(
            RetriableTaskFrame::builder()
                .frame(noop_frame())
                .retries(NonZeroU32::new(3).unwrap())
                .linear(Duration::from_millis(100))
                .build(),
        );
    });
}

#[divan::bench]
fn new_retriable_exponential(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(
            RetriableTaskFrame::builder()
                .frame(noop_frame())
                .retries(NonZeroU32::new(3).unwrap())
                .exponential(2.0)
                .build(),
        );
    });
}

#[divan::bench]
fn new_retriable_jitter(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(
            RetriableTaskFrame::builder()
                .frame(noop_frame())
                .retries(NonZeroU32::new(3).unwrap())
                .full_jitter(chronographer::task::ConstantBackoffStrategy::new(Duration::from_millis(100)), 2.0)
                .build(),
        );
    });
}

#[divan::bench]
fn new_conditional_frame(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(
            ConditionalTaskFrame::builder()
                .predicate(|_: &chronographer::task::RestrictTaskFrameContext| async { true })
                .frame(noop_frame())
                .build(),
        );
    });
}

#[divan::bench]
fn new_dependency_frame(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(
            DependencyTaskFrame::builder()
                .frame(noop_frame())
                .dependency(FrameDependency::external(|| std::future::ready(true)))
                .build(),
        );
    });
}

#[divan::bench]
fn new_timeout_frame(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(
            TimeoutTaskFrame::builder()
                .frame(noop_frame())
                .duration(Duration::from_secs(1))
                .build(),
        );
    });
}

#[divan::bench]
fn new_fallback_frame(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(FallbackTaskFrame::singular(
            noop_frame(),
            NoOperationTaskFrame::<BenchError, BenchError>::default(),
        ));
    });
}

#[divan::bench]
fn new_delay_frame(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(DelayTaskFrame::new(noop_frame(), Duration::from_secs(1)));
    });
}

#[divan::bench(args = [1usize, 10, 100, 1_000])]
fn new_collection_frame(bencher: divan::Bencher, jobs: usize) {
    bencher.counter(jobs).bench(|| {
        let frames: Vec<Arc<dyn ErasedTaskFrame<()>>> = (0..jobs)
            .map(|_| Arc::new(noop_frame()) as Arc<dyn ErasedTaskFrame<()>>)
            .collect();
        divan::black_box(CollectionTaskFrame::sequential(frames));
    });
}

#[divan::bench(args = [1usize, 10, 100, 1_000])]
fn new_collection_parallel(bencher: divan::Bencher, jobs: usize) {
    bencher.counter(jobs).bench(|| {
        let frames: Vec<Arc<dyn ErasedTaskFrame<()>>> = (0..jobs)
            .map(|_| Arc::new(noop_frame()) as Arc<dyn ErasedTaskFrame<()>>)
            .collect();
        divan::black_box(CollectionTaskFrame::parallel(
            frames,
            GroupedTaskFramesQuitOnFailure,
        ));
    });
}

#[divan::bench]
fn new_collection_empty(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(CollectionTaskFrame::sequential(vec![]));
    });
}

#[divan::bench]
fn new_builder_instant_retry(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(
            TaskFrameBuilder::builder(noop_frame())
                .with_instant_retry(NonZeroU32::new(3).unwrap())
                .build(),
        );
    });
}

#[divan::bench]
fn new_builder_retry(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(
            TaskFrameBuilder::builder(noop_frame())
                .with_retry(NonZeroU32::new(3).unwrap(), Duration::from_millis(100))
                .build(),
        );
    });
}

#[divan::bench]
fn new_builder_timeout(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(
            TaskFrameBuilder::builder(noop_frame())
                .with_timeout(Duration::from_secs(1))
                .build(),
        );
    });
}

#[divan::bench]
fn new_builder_fallback(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(
            TaskFrameBuilder::builder(noop_frame())
                .with_fallback(NoOperationTaskFrame::<BenchError, BenchError>::default())
                .build(),
        );
    });
}

#[divan::bench]
fn new_builder_condition(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(
            TaskFrameBuilder::builder(noop_frame())
                .with_condition(|_: &chronographer::task::RestrictTaskFrameContext| async { true })
                .build(),
        );
    });
}

#[divan::bench]
fn new_builder_dependency(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(
            TaskFrameBuilder::builder(noop_frame())
                .with_dependency(FrameDependency::external(|| std::future::ready(true)))
                .build(),
        );
    });
}

#[divan::bench(args = [1usize, 10, 100])]
fn combine_dependencies(bencher: divan::Bencher, count: usize) {
    bencher.bench(|| {
        let mut dep = FrameDependency::external(|| std::future::ready(true));
        for _ in 0..count {
            dep = dep & FrameDependency::external(|| std::future::ready(true));
        }
        divan::black_box(dep);
    });
}
