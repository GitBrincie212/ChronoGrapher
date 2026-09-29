use crate::util::{runtime, SharedVirtualClock};
use chronographer::scheduler::clock::{
    AdvanceableSchedulerClock, ProgressiveClock, SchedulerClock, VirtualClock,
};
use std::time::{Duration, SystemTime};

fn fixed_time() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_millis(1_700_000_000_000)
}

#[divan::bench]
fn now_virtual_clock(bencher: divan::Bencher) {
    let clock = VirtualClock::from_value(1_700_000_000_000);
    bencher.bench(|| {
        divan::black_box(clock.now());
    });
}

#[divan::bench(args = [0u64, 1, 1_000, 1_000_000, 86_400_000, 604_800_000])]
fn advance_virtual_clock(bencher: divan::Bencher, ms: u64) {
    let clock = VirtualClock::from_value(1_700_000_000_000);
    bencher.bench(|| {
        clock.advance(Duration::from_millis(divan::black_box(ms)));
    });
}

#[divan::bench]
fn advance_to_virtual_clock(bencher: divan::Bencher) {
    let clock = VirtualClock::from_value(1_700_000_000_000);
    let to = fixed_time();
    bencher.bench(|| {
        clock.advance_to(divan::black_box(to));
    });
}

#[divan::bench]
fn idle_to_virtual_clock(bencher: divan::Bencher) {
    let clock = VirtualClock::from_value(1_700_000_000_000);
    let to = fixed_time();
    bencher.bench(|| {
        runtime().block_on(clock.idle_to(divan::black_box(to)));
    });
}

#[divan::bench]
fn from_value_virtual_clock(bencher: divan::Bencher) {
    bencher.bench(|| {
        divan::black_box(VirtualClock::from_value(1_700_000_000_000));
    });
}

#[divan::bench]
fn now_progressive_clock(bencher: divan::Bencher) {
    let clock = ProgressiveClock::default();
    bencher.bench(|| {
        divan::black_box(clock.now());
    });
}

#[divan::bench]
fn now_shared_virtual_clock(bencher: divan::Bencher) {
    let clock = SharedVirtualClock::default();
    bencher.bench(|| {
        divan::black_box(clock.now());
    });
}
