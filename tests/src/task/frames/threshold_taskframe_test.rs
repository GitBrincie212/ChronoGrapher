use chronographer::task::CollectionTaskFrame;
use chronographer::task::GroupedTaskFramesQuitOnFailure;
use chronographer::task::GroupedTaskFramesSilent;
use chronographer::task::SequentialExecStrategy;
use chronographer::task::Task;
use chronographer::task::TaskFrame;
use chronographer::task::TaskFrameContext;
use chronographer::task::TaskScheduleImmediate;
use chronographer::task::ThresholdErrorsCountLogic;
use chronographer::task::ThresholdSuccessReachBehaviour;
use chronographer::task::ThresholdSuccessesCountLogic;
use chronographer::task::ThresholdTaskFrame;
use std::fmt::Display;
use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crate::impl_counting_frame;

#[allow(dead_code)]
#[derive(Debug)]
struct DummyError(&'static str);

impl Display for DummyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "threshold error")
    }
}

impl_counting_frame!(DummyError);

macro_rules! run_until_threshold {
    ($threshold:expr, $var:ident) => {
        for _ in 0..$threshold {
            assert!($var.run().await.is_ok())
        }
    };
}

#[tokio::test]
async fn threshold_execute_and_count_all_frames() {
    let counter = Arc::new(AtomicUsize::new(0));
    let frame = CollectionTaskFrame::new(
        vec![ok_frame(&counter), ok_frame(&counter)],
        SequentialExecStrategy::new(GroupedTaskFramesQuitOnFailure),
    );

    let threshold = NonZero::new(5).expect("God bless you");

    let frame = ThresholdTaskFrame::builder()
        .threshold_logic(Box::new(ThresholdSuccessesCountLogic))
        .frame(frame)
        .threshold_reach_behaviour(Box::new(ThresholdSuccessReachBehaviour))
        .threshold(threshold)
        .build();
    let task = Task::new(TaskScheduleImmediate, frame);
    let erased = task.into_erased();
    run_until_threshold!(5, erased);
    assert_eq!(
        counter.load(Ordering::SeqCst),
        10,
        "Counter should be equal to threshold * 2 (threshold being 5)"
    );
}

#[tokio::test]
async fn threshold_count_failing_frames() {
    let counter = Arc::new(AtomicUsize::new(0));
    let frame = CollectionTaskFrame::new(
        vec![failing_frame(&counter), failing_frame(&counter)],
        SequentialExecStrategy::new(GroupedTaskFramesSilent),
    );

    let threshold = NonZero::new(2).unwrap();

    let frame = ThresholdTaskFrame::builder()
        .threshold_logic(Box::new(ThresholdErrorsCountLogic))
        .frame(frame)
        .threshold_reach_behaviour(Box::new(ThresholdSuccessReachBehaviour))
        .threshold(threshold)
        .build();
    let task = Task::new(TaskScheduleImmediate, frame);
    let erased = task.into_erased();

    run_until_threshold!(3, erased);
    assert_eq!(
        counter.load(Ordering::SeqCst),
        6,
        "Counter should be equal to threshold * 2 (threshold being 5)"
    );
}

// TODO: Add this this test when `ThresholdErrorReachBehaviour` is implemented
//
// #[tokio::test]
// async fn threshold_success_behavior_error_fails_when_threshold_reached() {
//     let counter = Arc::new(AtomicUsize::new(0));
//     let frame = CollectionTaskFrame::new(
//         vec![ok_frame(&counter)],
//         SequentialExecStrategy::new(GroupedTaskFramesSilent),
//     );
//
//     let threshold = NonZero::new(2).unwrap();
//
//     let frame = ThresholdTaskFrame::builder()
//         .threshold_logic(Box::new(ThresholdSuccessesCountLogic))
//         .frame(frame)
//         .threshold_reach_behaviour(Box::new(ThresholdErrorReachBehaviour))
//         .build();
//      let task = Task::new(TaskScheduleImmediate, frame);
//      let erased = task.into_erased();
//
//      run_until_threshold!(2, erased);
//      assert_eq(
//          counter.load(Ordering::SeqCst),
//          2,
//          "Counter should be equal to threshold * 2 (threshold being 5)"
//      );
// }
