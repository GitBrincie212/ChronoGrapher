use crate::utils::ByteWheel;
use std::time::Duration;
use std::vec::Drain;

pub struct HierarchicalTimingWheel<T: 'static + Send> {
    level1: ByteWheel<T>,
    level2: ByteWheel<T>,
    level3: ByteWheel<T>,
    level4: ByteWheel<T>,
    level5: ByteWheel<T>,
}

const END_RANGE0: u128 = 256u128;
const END_RANGE1: u128 = 256u128.pow(2);
const END_RANGE2: u128 = 256u128.pow(3);
const END_RANGE3: u128 = 256u128.pow(4);
const END_RANGE4: u128 = 256u128.pow(5);

impl<T: 'static + Send> Default for HierarchicalTimingWheel<T> {
    fn default() -> Self {
        Self {
            level1: ByteWheel::default(),
            level2: ByteWheel::default(),
            level3: ByteWheel::default(),
            level4: ByteWheel::default(),
            level5: ByteWheel::default(),
        }
    }
}

pub struct TickIter<'a, T> {
    shards: std::array::IntoIter<&'a mut ByteWheel<T>, 5>,
    current_iter: Option<Drain<'a, T>>,
    stop: bool,
}

impl<T: 'static + Send> Iterator for TickIter<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(iter) = &mut self.current_iter {
                if let Some(item) = iter.next() {
                    return Some(item);
                }

                self.current_iter = None;
            }

            if self.stop {
                return None;
            }

            let shard = self.shards.next()?;
            let (result, curr) = shard.tick();

            if curr != 0 {
                self.stop = true;
            }

            self.current_iter = Some(result);
        }
    }
}

impl<T: 'static + Send> HierarchicalTimingWheel<T> {
    pub fn insert(&mut self, value: T, delay: Duration) {
        let millis = delay.as_millis();

        let (target, level_index) = match millis {
            0..END_RANGE0 => (&mut self.level1, 0),
            END_RANGE0..END_RANGE1 => (&mut self.level2, 1),
            END_RANGE1..END_RANGE2 => (&mut self.level3, 2),
            END_RANGE2..END_RANGE3 => (&mut self.level4, 3),
            END_RANGE3..END_RANGE4 => (&mut self.level5, 4),
            _ => panic!("value out of supported range"),
        };

        let shift: u8 = level_index * 8;
        let slot = ((millis >> shift) & 0xFF) as u8;

        target.insert(slot, value);
    }

    /*
    pub async fn skip(&self, delay: Duration) {
        let mut millis = delay.as_millis();

        // TODO: Heap-allocation cost from Arc<T> might be slightly cheaper for reading than mpsc channels but not 100% sure
        for level in [&self.level1, &self.level2, &self.level3, &self.level4] {
            for shard in level {
                let wrapped = (millis & 31) as u8;
                millis = millis >> 6;
                if millis == 0 {
                    return;
                }

                shard
                    .send(WheelShardCommand::Skip(wrapped, self.skip_tx.clone()))
                    .await
                    .expect("Cannot send message to corresponding shard");
            }
        }
    }
    */

    pub fn tick(&mut self) -> TickIter<'_, T> {
        TickIter {
            shards: [
                &mut self.level1,
                &mut self.level2,
                &mut self.level3,
                &mut self.level4,
                &mut self.level5,
            ].into_iter(),
            current_iter: None,
            stop: false,
        }
    }

    pub fn clear(&mut self) {
        for shard in [
            &mut self.level1,
            &mut self.level2,
            &mut self.level3,
            &mut self.level4,
            &mut self.level5,
        ]
        .into_iter()
        {
            shard.clear()
        }
    }
}
