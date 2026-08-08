pub mod backoff;
pub mod timing_wheel;
pub mod util;

pub fn main() {
    let keep_rt_alive = util::runtime().enter();

    divan::main();
}
