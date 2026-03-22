use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::{DynTaskFrame, ErasedTaskFrame, TaskFrame, TaskFrameContext, TaskHook, TaskHookEvent, TaskRef, TaskTrigger};
use async_trait::async_trait;
use std::error::Error;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::SystemTime;
use crossbeam::queue::SegQueue;
use haphazard::HazardPointer;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

struct EphemeralTaskRecord<C: SchedulerConfig> {
    frame: Box<dyn DynTaskFrame<C::TaskError>>,
    trigger: Box<dyn TaskTrigger>,
    cancel_token: CancellationToken,
}

pub struct EphemeralTaskHandle<C: SchedulerConfig> {
    idx: usize,
    generation: u16,
    shard: Weak<BufferShard<EphemeralTaskRecord<C>>>
}

impl<C: SchedulerConfig> Clone for EphemeralTaskHandle<C> {
    fn clone(&self) -> Self {
        Self {
            shard: self.shard.clone(),
            idx: self.idx,
            generation: self.generation,
        }
    }
}

pub struct TaskFrameGuard<C: SchedulerConfig> {
    guard: BufferGuard<EphemeralTaskRecord<C>>,
}

#[async_trait]
impl<C: SchedulerConfig> DynTaskFrame<C::TaskError> for TaskFrameGuard<C> {
    async fn erased_execute(&self, ctx: &TaskFrameContext) -> Result<(), C::TaskError> {
        self.guard.frame.erased_execute(ctx).await
    }

    fn erased(&self) -> &dyn ErasedTaskFrame {
        self.guard.frame.erased()
    }
}

impl<C: SchedulerConfig> Deref for TaskFrameGuard<C> {
    type Target = dyn DynTaskFrame<C::TaskError>;

    fn deref(&self) -> &Self::Target {
        self.guard.frame.as_ref()
    }
}

pub struct TaskTriggerGuard<C: SchedulerConfig> {
    guard: BufferGuard<EphemeralTaskRecord<C>>,
}

#[async_trait]
impl<C: SchedulerConfig> TaskTrigger for TaskTriggerGuard<C> {
    async fn trigger(&self, now: SystemTime) -> Result<SystemTime, Box<dyn Error + Send + Sync>> {
        self.guard.trigger.trigger(now).await
    }
}

impl<C: SchedulerConfig> Deref for TaskTriggerGuard<C> {
    type Target = dyn TaskTrigger;

    fn deref(&self) -> &Self::Target {
        self.guard.trigger.as_ref()
    }
}


#[async_trait]
impl<C: SchedulerConfig> TaskRef<C> for EphemeralTaskHandle<C> {
    type TaskFrame<'a> = TaskFrameGuard<C>;
    type TaskTrigger<'a> = TaskTriggerGuard<C>;

    async fn frame(&self) -> Option<Self::TaskFrame<'_>> {
        let shard = self.shard.upgrade()?;
        let value = shard.read(self.idx, self.generation).await?;
        Some(TaskFrameGuard { guard: value })
    }

    async fn trigger(&self) -> Option<Self::TaskTrigger<'_>> {
        let shard = self.shard.upgrade()?;
        let value = shard.read(self.idx, self.generation).await?;
        Some(TaskTriggerGuard { guard: value })
    }

    async fn attach_hook<EV: TaskHookEvent>(&self, hook: Arc<impl TaskHook<EV>>) {
        todo!()
    }

    async fn get_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) -> Option<Arc<T>> {
        todo!()
    }

    async fn emit_event<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) {
        todo!()
    }

    async fn detach_hook<EV: TaskHookEvent, T: TaskHook<EV>>(&self) {
        todo!()
    }

    fn is_invalid(&self) -> bool {
        if let Some(shard) = self.shard.upgrade()
            && let Some(_) = shard.read_blocking(self.idx, self.generation)
        {
            return false;
        }
        true
    }

    async fn cancelled(&self) {
        if let Some(shard) = self.shard.upgrade()
            && let Some(value) = shard.read(self.idx, self.generation).await
        {
            value.cancel_token.cancelled().await
        }
    }

    async fn cancel(&self) {
        if let Some(shard) = self.shard.upgrade()
            && let Some(value) = shard.read(self.idx, self.generation).await
        {
            value.cancel_token.cancel()
        }
    }
}

struct BufferItem<T: Send + Sync> {
    content: haphazard::AtomicPtr<T>,

    /*
    Possible for the generation counter to wrap around and cause an ABA problem but this would require
    extreme cases which are highly unlikely (around 65k insertions/removals to get to this).
    */
    generation: AtomicU16
}

struct BufferShard<T: Send + Sync> {
    buffer: RwLock<Vec<BufferItem<T>>>,
    free: SegQueue<usize>
}

struct BufferGuard<T: Send + Sync> {
    hazard: HazardPointer<'static>,
    val: *const T
}

unsafe impl<T: Send + Sync> Send for BufferGuard<T> {}
unsafe impl<T: Send + Sync> Sync for BufferGuard<T> {}

impl<T: Send + Sync> Deref for BufferGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.val.as_ref().unwrap_unchecked() }
    }
}

impl<T: Send + Sync> Default for BufferShard<T> {
    fn default() -> Self {
        Self {
            buffer: RwLock::new(Vec::new()),
            free: SegQueue::new(),
        }
    }
}

impl<T: Send +  Sync> BufferShard<T> {
    pub async fn push(&self, value: T) -> (usize, u16) {
        if let Some(idx) = self.free.pop() {
            let read = self.buffer.read().await;
            read[idx].content.store(Box::new(value));
            let generation = read[idx].generation.load(Ordering::Acquire);
            return (idx, generation);
        }

        let mut lock = self.buffer.write().await;
        let length = lock.len();

        lock.push(BufferItem {
            content: haphazard::AtomicPtr::from(Box::new(value)),
            generation: AtomicU16::new(0)
        });

        (length, 0)
    }

    pub async fn remove(&self, idx: usize) {
        let read = self.buffer.read().await;
        if let Some(item) = read.get(idx)  {
            let old = unsafe { item.content.swap_ptr(std::ptr::null_mut()) };
            item.generation.fetch_add(1, Ordering::Release);

            if let Some(replaced) = old {
                unsafe { replaced.retire(); }
                self.free.push(idx);
            }
        }
    }

    pub(crate) fn internal_read(&self, item: &BufferItem<T>, generation: u16) -> Option<BufferGuard<T>> {
        if item.generation.load(Ordering::Acquire) != generation {
            return None;
        }

        let mut hazard = HazardPointer::new();
        let val = item.content.safe_load(&mut hazard)? as *const T;

        Some(BufferGuard { val, hazard })
    }

    pub async fn read(&self, idx: usize, generation: u16) -> Option<BufferGuard<T>> {
        let lock = self.buffer.read().await;
        let item = lock.get(idx)?;
        self.internal_read(item, generation)
    }

    pub fn read_blocking(&self, idx: usize, generation: u16) -> Option<BufferGuard<T>> {
        let lock = self.buffer.blocking_read();
        let item = lock.get(idx)?;
        self.internal_read(item, generation)
    }
}

pub struct EphemeralSchedulerTaskStore<C: SchedulerConfig> {
    tasks: Box<[Arc<BufferShard<EphemeralTaskRecord<C>>>]>,
}

impl<C: SchedulerConfig> Default for EphemeralSchedulerTaskStore<C> {
    fn default() -> Self {
        let workers = std::thread::available_parallelism()
            .unwrap_or(NonZeroUsize::MIN)
            .get();

        let shard_count = (workers * 4).next_power_of_two();
        let mut tasks = Vec::with_capacity(shard_count);

        for _ in 0..shard_count {
            tasks.push(Arc::new(BufferShard::default()));
        }

        Self {
            tasks: tasks.into_boxed_slice(),
        }
    }
}

#[async_trait]
impl<C: SchedulerConfig> SchedulerTaskStore<C> for EphemeralSchedulerTaskStore<C> {
    type TaskRef = EphemeralTaskHandle<C>;

    async fn allocate(
        &self,
        trigger: impl TaskTrigger,
        frame: impl TaskFrame<Error=C::TaskError>
    ) -> Result<Self::TaskRef, Box<dyn Error + Send + Sync>> {
        let shard_idx = fastrand::usize(0..self.tasks.len());
        let (idx, generation) = self.tasks[shard_idx].push(EphemeralTaskRecord {
            trigger: Box::new(trigger),
            frame: Box::new(frame),
            cancel_token: CancellationToken::new(),
        }).await;

        Ok(EphemeralTaskHandle {
            idx,
            generation,
            shard: Arc::downgrade(&self.tasks[shard_idx]),
        })
    }

    async fn deallocate(&self, handle: &Self::TaskRef) {
        if let Some(shard) = handle.shard.upgrade() {
            shard.remove(handle.idx).await;
        }
    }

    async fn clear(&self) {
        todo!()
    }
}
