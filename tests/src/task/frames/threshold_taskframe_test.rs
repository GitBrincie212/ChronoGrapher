use chronographer::task::CollectionTaskFrame;
use chronographer::task::GroupedTaskFramesQuitOnFailure;
use chronographer::task::GroupedTaskFramesSilent;
use chronographer::task::SequentialExecStrategy;
use chronographer::task::Task;
use chronographer::task::TaskScheduleImmediate;
use chronographer::task::ThresholdErrorsCountLogic;
use chronographer::task::ThresholdSuccessReachBehaviour;
use chronographer::task::ThresholdSuccessesCountLogic;
use chronographer::task::ThresholdTaskFrame;
use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use crate::task::frames::{failing_frame, ok_frame};

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

#[tokio::test]
async fn threshold_of_one_reached_after_single_run() {
    let counter = Arc::new(AtomicUsize::new(0));
    let frame = CollectionTaskFrame::new(
        vec![ok_frame(&counter)],
        SequentialExecStrategy::new(GroupedTaskFramesQuitOnFailure),
    );

    let threshold = NonZero::new(1).unwrap();

    let frame = ThresholdTaskFrame::builder()
        .threshold_logic(Box::new(ThresholdSuccessesCountLogic))
        .frame(frame)
        .threshold_reach_behaviour(Box::new(ThresholdSuccessReachBehaviour))
        .threshold(threshold)
        .build();

    let task = Task::new(TaskScheduleImmediate, frame);
    let erased = task.into_erased();

    erased.run().await.unwrap();

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "A threshold of 1 should be reached after exactly one successful run"
    );
}

#[tokio::test]
async fn sub_threshold_runs_do_not_trigger_reach_behaviour() {
    let counter = Arc::new(AtomicUsize::new(0));
    let frame = CollectionTaskFrame::new(
        vec![ok_frame(&counter)],
        SequentialExecStrategy::new(GroupedTaskFramesQuitOnFailure),
    );

    let threshold = NonZero::new(5).unwrap();

    let frame = ThresholdTaskFrame::builder()
        .threshold_logic(Box::new(ThresholdSuccessesCountLogic))
        .frame(frame)
        .threshold_reach_behaviour(Box::new(ThresholdSuccessReachBehaviour))
        .threshold(threshold)
        .build();

    let task = Task::new(TaskScheduleImmediate, frame);
    let erased = task.into_erased();

    // Run only 3 times out of required 5
    for _ in 0..3 {
        erased.run().await.unwrap();
    }

    assert_eq!(
        counter.load(Ordering::SeqCst),
        3,
        "Only 3 runs should have occurred, threshold not yet reached"
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
