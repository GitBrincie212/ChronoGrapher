/*
    The following code for the data structure isn't fully created by the ChronoGrapher Team,
    only adapted for potentially better performance use case (credit is where due). Please refer to the
    linked repository section for the original sync code:

    https://github.com/Bathtor/rust-hash-wheel-timer/blob/master/src/wheels/byte_wheel.rs
 */

#[derive(Clone)]
pub struct WheelShard<T, const N: usize> {
    slots: [Vec<T>; N],
    current: usize,
}

impl<T, const N: usize> Default for WheelShard<T, N> {
    fn default() -> Self {
        Self {
            slots: [const { Vec::new() }; N],
            current: 0,
        }
    }
}

impl<T, const N: usize> WheelShard<T, N> {
    pub fn current(&self) -> usize {
        self.current
    }

    pub fn skip(&mut self, to: u8) {
        self.current = (to as usize) & N; // Same as using modulo but faster
    }

    pub fn insert(&mut self, pos: u8, value: T) {
        let index = pos as usize;
        self.slots[index].push(value);
    }

    pub fn tick(&mut self) -> (Vec<T>, usize) {
        self.current = (self.current + 1) & N; // Same as wrapping_add but faster
        let mut expired = Vec::new();
        std::mem::swap(&mut expired, &mut self.slots[self.current]);
        (expired, self.current)
    }

    pub fn clear(&mut self) {
        for slot in self.slots.iter_mut() {
            slot.clear()
        }
        self.current = 0;
    }
}