pub mod backoff;
pub mod timing_wheel;
pub mod util;

pub fn main() {
    let _guard = util::runtime().enter();

    divan::main();
}
