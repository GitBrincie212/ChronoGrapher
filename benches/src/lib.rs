pub mod backoff;
pub mod clocks;
pub mod timing_wheel;
pub mod util;

pub fn main() {
    let _guard = util::runtime().enter();

    divan::main();
}
