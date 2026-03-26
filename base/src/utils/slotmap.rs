use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering};
use crossbeam::queue::SegQueue;
use haphazard::HazardPointer;
use tokio::sync::RwLock;

struct Slot<T: Send + Sync> {
    content: haphazard::AtomicPtr<T>,

    /*
    Possible for the generation counter to wrap around and cause an ABA problem but this would require
    extreme cases which are highly unlikely (around 65k insertions/removals to get to this).
    */
    generation: AtomicU16
}

pub struct SlotMapShard<T: Send + Sync> {
    buffer: RwLock<Vec<Slot<T>>>,

    // TODO: Consider an intrusive linked list over a separate free queue for reduced memory usage
    free: SegQueue<usize>,

    length: AtomicUsize
}

pub struct SlotMapGuard<T: Send + Sync> {
    hazard: HazardPointer<'static>,
    val: *const T
}

unsafe impl<T: Send + Sync> Send for SlotMapGuard<T> {}
unsafe impl<T: Send + Sync> Sync for SlotMapGuard<T> {}

impl<T: Send + Sync> Deref for SlotMapGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.val.as_ref().unwrap_unchecked() }
    }
}

impl<T: Send + Sync> Default for SlotMapShard<T> {
    fn default() -> Self {
        Self {
            buffer: RwLock::new(Vec::new()),
            free: SegQueue::new(),
            length: AtomicUsize::new(0)
        }
    }
}

pub struct SlotKey<T: Send + Sync> {
    idx: usize,
    generation: u16,
    shard_idx: usize,
    shards: Weak<[SlotMapShard<T>]>
}

impl<T: Send + Sync> PartialEq for SlotKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
            && self.generation == other.generation
            && self.shards.ptr_eq(&other.shards)
    }
}

impl<T: Send + Sync> Eq for SlotKey<T> {}

impl<T: Send + Sync> Hash for SlotKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.idx);
        state.write_u16(self.generation);
        state.write_usize(self.shard_idx);
    }
}

impl<T: Send + Sync> Clone for SlotKey<T> {
    fn clone(&self) -> Self {
        Self {
            idx: self.idx,
            generation: self.generation,
            shard_idx: self.shard_idx,
            shards: self.shards.clone()
        }
    }
}

impl<T: Send + Sync> SlotKey<T> {
    pub async fn remove(&self) {
        if let Some(shards) = self.shards.upgrade()
            && let read = shards[self.shard_idx].buffer.read().await
            && let Some(item) = read.get(self.idx)
        {
            let old = unsafe { item.content.swap_ptr(std::ptr::null_mut()) };
            item.generation.fetch_add(1, Ordering::Release);

            if let Some(replaced) = old {
                unsafe { replaced.retire(); }
                shards[self.shard_idx].length.fetch_sub(1, Ordering::Release);
                shards[self.shard_idx].free.push(self.idx);
            }
        }
    }

    #[inline(always)]
    fn internal_read(&self, item: &Slot<T>) -> Option<SlotMapGuard<T>> {
        if item.generation.load(Ordering::Acquire) != self.generation {
            return None;
        }

        let mut hazard = HazardPointer::new();
        let val = item.content.safe_load(&mut hazard)? as *const T;

        Some(SlotMapGuard { val, hazard })
    }

    pub async fn read(&self) -> Option<SlotMapGuard<T>> {
        if let Some(shards) = self.shards.upgrade() {
            let lock = shards[self.shard_idx].buffer.read().await;
            let item = lock.get(self.idx)?;
            return self.internal_read(item);
        }

        None
    }

    pub fn read_blocking(&self) -> Option<SlotMapGuard<T>> {
        if let Some(shards) = self.shards.upgrade() {
            let lock = shards[self.shard_idx].buffer.blocking_read();
            let item = lock.get(self.idx)?;
            return self.internal_read(item);
        }
        None
    }

    pub fn is_valid(&self) -> bool {
        let Some(shards) = self.shards.upgrade() else {
            return false;
        };

        let lock = shards[self.shard_idx].buffer.blocking_read();
        lock[self.idx].generation.load(Ordering::Acquire) == self.generation
    }
}

pub struct ConcurrentSlotMap<T: Send + Sync>(Arc<[SlotMapShard<T>]>);

impl<T: Send + Sync> Default for ConcurrentSlotMap<T> {
    fn default() -> Self {
        let parallelism = std::thread::available_parallelism()
            .unwrap_or(NonZeroUsize::MIN)
            .get();

        let shard_count = (parallelism * 4).next_power_of_two();
        let mut shards = Vec::with_capacity(shard_count);

        for _ in 0..shard_count {
            shards.push(SlotMapShard::default());
        }

        Self(shards.into())
    }
}

impl<T: Send + Sync> ConcurrentSlotMap<T> {
    pub async fn allocate(&self, value: T) -> SlotKey<T> {
        let shard_idx = fastrand::usize(0..self.0.len());
        let shard = &self.0[shard_idx];
        shard.length.fetch_sub(1, Ordering::Release);
        if let Some(idx) = shard.free.pop() {
            let read = shard.buffer.read().await;
            read[idx].content.store(Box::new(value));
            let generation = read[idx].generation.load(Ordering::Acquire);
            return SlotKey::<T> {
                idx,
                generation,
                shard_idx,
                shards: Arc::downgrade(&self.0),
            };
        }

        let mut lock = shard.buffer.write().await;
        let length = lock.len();

        lock.push(Slot {
            content: haphazard::AtomicPtr::from(Box::new(value)),
            generation: AtomicU16::new(0)
        });

        SlotKey::<T> {
            idx: length,
            generation: 0,
            shard_idx,
            shards: Arc::downgrade(&self.0),
        }
    }

    pub async fn clear(&self) {
        for shard in self.0.iter() {
            shard.buffer.write().await.clear();
        }
    }

    pub fn blocking_clear(&self) {
        for shard in self.0.iter() {
            shard.buffer.blocking_write().clear();
        }
    }
}