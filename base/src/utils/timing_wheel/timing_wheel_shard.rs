/*
   The following code for the data structure isn't fully created by the ChronoGrapher Team,
   only adapted for potentially better performance use case (credit is where due). Please refer to the
   linked repository section for the original sync code:

   https://github.com/Bathtor/rust-hash-wheel-timer/blob/master/src/wheels/byte_wheel.rs
*/

#[derive(Clone)]
pub struct ByteWheel<T> {
    slots: [Vec<T>; 256],
    current: u8,
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
        self.current as usize
    }

    pub fn skip(&mut self, to: u8) {
        self.current = to;
    }

    pub fn insert(&mut self, pos: u8, value: T) {
        let index = pos as usize;
        self.slots[index].push(value);
    }

    pub fn tick(&mut self) -> (Vec<T>, bool) {
        self.current = self.current.wrapping_add(1); // Same as wrapping_add but faster
        let mut expired = Vec::new();
        std::mem::swap(&mut expired, &mut self.slots[self.current as usize]);
        (expired, self.current == 0)
    }

    pub fn clear(&mut self) {
        for slot in self.slots.iter_mut() {
            slot.clear()
        }
        self.current = 0;
    }
}
