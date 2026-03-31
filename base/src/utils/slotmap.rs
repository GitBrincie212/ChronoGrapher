use crossbeam::queue::SegQueue;
use haphazard::HazardPointer;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

struct Slot<T: Send + Sync> {
    content: haphazard::AtomicPtr<T>,
    generation: AtomicUsize,
}

pub struct SlotMapShard<T: Send + Sync> {
    buffer: RwLock<Vec<Slot<T>>>,

    // TODO: Consider an intrusive linked list over a separate free queue for reduced memory usage
    free: SegQueue<usize>,

    length: AtomicUsize,
}

pub struct SlotMapGuard<T: Send + Sync> {
    hazard: HazardPointer<'static>,
    val: *const T,
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
            length: AtomicUsize::new(0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotKey {
    idx: usize,
    generation: usize,
    shard_idx: u16,
}

pub type RefSlotMap<'a, T> = SlotMap<&'a [SlotMapShard<T>]>;
pub type OwnedSlotMap<T> = SlotMap<Box<[SlotMapShard<T>]>>;
pub type StrongSlotMap<T> = SlotMap<Arc<[SlotMapShard<T>]>>;
pub type WeakSlotMap<T> = SlotMap<Weak<[SlotMapShard<T>]>>;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SlotMap<S>(S);

#[inline(always)]
fn calculate_shards() -> usize {
    let parallelism = std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::MIN)
        .get();

    (parallelism * 4).next_power_of_two()
}

impl<T, S> Default for SlotMap<S>
where
    S: Deref<Target = [SlotMapShard<T>]> + From<Vec<SlotMapShard<T>>>,
    T: Send + Sync
{
    fn default() -> Self {
        let shard_count = calculate_shards();
        let shards: Vec<_> = (0..shard_count)
            .map(|_| SlotMapShard::default())
            .collect();

        Self(shards.into())
    }
}

impl<T: Send + Sync> StrongSlotMap<T> {
    pub fn downgrade(&self) -> WeakSlotMap<T> {
        SlotMap(Arc::downgrade(&self.0))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: Send + Sync> WeakSlotMap<T> {
    pub fn upgrade(&self) -> Option<StrongSlotMap<T>> {
        Some(SlotMap(self.0.upgrade()?))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}


impl<T, S> SlotMap<S>
where
    S: Deref<Target = [SlotMapShard<T>]> + From<Vec<SlotMapShard<T>>>,
    T: Send + Sync
{
    pub fn with_shards(shard_count: usize) -> Self {
        let shards: Vec<_> = (0..shard_count)
            .map(|_| SlotMapShard::default())
            .collect();

        Self(shards.into())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_shads_and_capacity(calculate_shards(), capacity)
    }

    pub fn with_shads_and_capacity(shard_count: usize, capacity: usize) -> Self {
        let base = capacity / shard_count;
        let remainder = capacity % shard_count;

        let shards: Vec<_> = (0..shard_count)
            .map(|x| {
                let size = base + if x < remainder { 1 } else { 0 };
                let mut prealloc = Vec::with_capacity(size);
                let free = SegQueue::new();
                for i in 0..size {
                    prealloc.push(Slot {
                        content: unsafe { haphazard::AtomicPtr::new(std::ptr::null_mut()) },
                        generation: AtomicUsize::new(0),
                    });
                    free.push(i);
                }

                SlotMapShard {
                    free,
                    buffer: RwLock::new(prealloc),
                    length: AtomicUsize::new(size)
                }
            })
            .collect();

        Self(shards.into())
    }

    #[inline(always)]
    fn distribute_to_shard(&self) -> (&SlotMapShard<T>, u16) {
        let shard_idx = fastrand::u16(0..(self.0.len() as u16));
        let shard = &self.0[shard_idx as usize];
        shard.length.fetch_add(1, Ordering::Release);
        (shard, shard_idx)
    }

    #[inline(always)]
    fn reuse_free_slot(
        &self,
        shard_idx: u16,
        idx: usize,
        value: T,
        read: RwLockReadGuard<'_, Vec<Slot<T>>>,
    ) -> SlotKey {
        read[idx].content.store(Box::new(value));
        let generation = read[idx].generation.load(Ordering::Acquire);
        SlotKey {
            idx,
            generation,
            shard_idx,
        }
    }

    #[inline(always)]
    fn allocate_slot(
        &self,
        shard_idx: u16,
        value: T,
        write: &mut RwLockWriteGuard<'_, Vec<Slot<T>>>,
    ) -> SlotKey {
        let length = write.len();
        write.push(Slot {
            content: haphazard::AtomicPtr::from(Box::new(value)),
            generation: AtomicUsize::new(0),
        });
        SlotKey {
            idx: length,
            generation: 0,
            shard_idx,
        }
    }

    pub async fn allocate(&self, value: T) -> SlotKey {
        let (shard, shard_idx) = self.distribute_to_shard();
        if let Some(idx) = shard.free.pop() {
            let read = shard.buffer.read().await;
            return self.reuse_free_slot(shard_idx, idx, value, read);
        }
        let mut lock = shard.buffer.write().await;
        self.allocate_slot(shard_idx, value, &mut lock)
    }

    pub fn blocking_allocate(&self, value: T) -> SlotKey {
        let (shard, shard_idx) = self.distribute_to_shard();
        if let Some(idx) = shard.free.pop() {
            let read = shard.buffer.blocking_read();
            return self.reuse_free_slot(shard_idx, idx, value, read);
        }
        let mut lock = shard.buffer.blocking_write();
        self.allocate_slot(shard_idx, value, &mut lock)
    }

    #[inline(always)]
    fn internal_remove(&self, key: &SlotKey, read: RwLockReadGuard<'_, Vec<Slot<T>>>) {
        if let Some(item) = read.get(key.idx) {
            let old = unsafe { item.content.swap_ptr(std::ptr::null_mut()) };
            item.generation.fetch_sub(1, Ordering::Release);
            if let Some(replaced) = old {
                unsafe {
                    replaced.retire();
                }
                self.0[key.shard_idx as usize]
                    .length
                    .fetch_sub(1, Ordering::Release);
                self.0[key.shard_idx as usize].free.push(key.idx);
            }
        }
    }

    pub async fn remove(&self, key: &SlotKey) {
        let read = self.0[key.shard_idx as usize].buffer.read().await;
        self.internal_remove(key, read);
    }
    pub fn blocking_remove(&self, key: &SlotKey) {
        let read = self.0[key.shard_idx as usize].buffer.blocking_read();
        self.internal_remove(key, read);
    }

    #[inline(always)]
    fn internal_read(&self, key: &SlotKey, item: &Slot<T>) -> Option<SlotMapGuard<T>> {
        if item.generation.load(Ordering::Acquire) != key.generation {
            return None;
        }
        let mut hazard = HazardPointer::new();
        let val = item.content.safe_load(&mut hazard)? as *const T;
        Some(SlotMapGuard { val, hazard })
    }

    pub async fn read(&self, key: &SlotKey) -> Option<SlotMapGuard<T>> {
        let lock = self.0[key.shard_idx as usize].buffer.read().await;
        let item = lock.get(key.idx)?;
        self.internal_read(key, item)
    }

    pub fn blocking_read(&self, key: &SlotKey) -> Option<SlotMapGuard<T>> {
        let lock = self.0[key.shard_idx as usize].buffer.blocking_read();
        let item = lock.get(key.idx)?;
        self.internal_read(key, item)
    }

    pub fn is_valid(&self, key: &SlotKey) -> bool {
        let lock = self.0[key.shard_idx as usize].buffer.blocking_read();
        lock[key.idx].generation.load(Ordering::Acquire) == key.generation
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

    pub fn len(&self) -> usize {
        let mut len: usize = 0;
        for shard in self.0.iter() {
            len += shard.length.load(Ordering::Acquire)
        }
        len
    }

    pub fn capacity(&self) -> usize {
        let mut capacity: usize = 0;
        for shard in self.0.iter() {
            capacity += shard.buffer.blocking_read().len();
        }
        capacity
    }

    pub fn shards(&self) -> usize {
        self.0.len()
    }
}