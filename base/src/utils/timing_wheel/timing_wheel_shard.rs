/*
   The following code for the data structure isn't fully created by the ChronoGrapher Team,
   only adapted for potentially better performance use case (credit is where due). Please refer to the
   linked repository section for the original sync code:

   https://github.com/Bathtor/rust-hash-wheel-timer/blob/master/src/wheels/byte_wheel.rs
*/
use std::vec::Drain;

#[derive(Clone)]
pub struct ByteWheel<T> {
    slots: [Vec<T>; 256],
    current: usize,
}

impl<T> Default for ByteWheel<T> {
    fn default() -> Self {
        Self {
            slots: [const { Vec::new() }; 256],
            current: 0,
        }
    }
}

impl<T> ByteWheel<T> {
    pub fn current(&self) -> usize {
        self.current
    }

    pub fn skip(&mut self, to: u8) {
        self.current = (to as usize) & 255; // Same as using modulo but faster
    }

    pub fn insert(&mut self, pos: u8, value: T) {
        let index = pos as usize;
        self.slots[index].push(value);
    }

    pub fn tick(&mut self) -> (Drain<T>, usize) {
        self.current = (self.current + 1) & 255; // Same as wrapping_add but faster
        (self.slots[self.current].drain(..), self.current)
    }

    pub fn clear(&mut self) {
        for slot in self.slots.iter_mut() {
            slot.clear()
        }
        self.current = 0;
    }
}
