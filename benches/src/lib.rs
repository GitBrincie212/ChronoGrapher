pub mod backoff;
pub mod clocks;
pub mod dispatcher;
pub mod engine;
pub mod hooks;
pub mod schedule;
pub mod task_construction;
pub mod task_store;
pub mod timing_wheel;
pub mod util;

pub fn main() {
    let _guard = util::runtime().enter();

    divan::main();
}
