use crate::utils::ByteWheel;
use std::time::Duration;

struct Entry<T> {
    value: T,
    precomputed: [u8; 5],
    level: u8
}

pub struct HierarchicalTimingWheel<T> {
    level1: ByteWheel<Entry<T>>,
    level2: ByteWheel<Entry<T>>,
    level3: ByteWheel<Entry<T>>,
    level4: ByteWheel<Entry<T>>,
    level5: ByteWheel<Entry<T>>,
}

impl<T> Default for HierarchicalTimingWheel<T> {
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

impl<T> HierarchicalTimingWheel<T> {
    pub fn insert(&mut self, value: T, delay: Duration) {
        let millis = delay.as_millis();
        let slots = [
            (millis & 0xFF) as u8,
            ((millis >> 8) & 0xFF) as u8,
            ((millis >> 16) & 0xFF) as u8,
            ((millis >> 24) & 0xFF) as u8,
            ((millis >> 32) & 0xFF) as u8,
        ];
        let mut level = 0;

        for i in (0..5).rev() {
            if slots[i] != 0 {
                level = i;
                break;
            }
        }

        let target = match level {
            0 => &mut self.level1,
            1 => &mut self.level2,
            2 => &mut self.level3,
            3 => &mut self.level4,
            4 => &mut self.level5,
            _ => unreachable!(),
        };

        let current = target.current() as u8;
        let slot = current.wrapping_add(slots[level]);

        target.insert(slot, Entry {
            value,
            precomputed: slots,
            level: level as u8,
        });
    }

    pub fn tick(&mut self) -> Vec<T> {
        let mut results = Vec::new();

        let (expired, wrapped0) = self.level1.tick();
        results.extend(expired.into_iter().map(|x| x.value));

        if wrapped0 {
            let mut levels = [
                &mut self.level1,
                &mut self.level2,
                &mut self.level3,
                &mut self.level4,
                &mut self.level5,
            ];

            for idx in 1..5 {
                let level = &mut levels[idx];
                let (expired, wrapped) = level.tick();
                for mut entry in expired {
                    entry.level -= 1;

                    let next_level = entry.level as usize;
                    let target = &mut levels[next_level];

                    let current = target.current() as u8;
                    let slot = current.wrapping_add(entry.precomputed[next_level]);

                    target.insert(slot, entry);
                }

                if !wrapped {
                    break;
                }
            }
        }

        results
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
