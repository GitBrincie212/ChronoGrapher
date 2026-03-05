pub mod default; // skipcq: RS-D1001

pub use default::*;
use std::ops::Deref;
use std::sync::Arc;
#[allow(unused_imports)]
use crate::scheduler::Scheduler;
use crate::scheduler::{ReschedulePayload, SchedulerConfig};
use crate::task::ErasedTask;
use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use tokio::sync::Notify;

pub struct EngineNotifier<C: SchedulerConfig> {
    id: C::TaskIdentifier,
    reschedule_queue: Arc<(SegQueue<ReschedulePayload<C>>, Notify)>,
}

impl<C: SchedulerConfig> Clone for EngineNotifier<C> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            reschedule_queue: self.reschedule_queue.clone(),
        }
    }
}

impl<C: SchedulerConfig> EngineNotifier<C> {
    pub fn new(
        id: C::TaskIdentifier,
        reschedule_queue: Arc<(SegQueue<ReschedulePayload<C>>, Notify)>
    ) -> Self {
        Self { id, reschedule_queue }
    }

    pub fn id(&self) -> &C::TaskIdentifier {
        &self.id
    }

    pub fn new_id(&mut self, id: C::TaskIdentifier) {
        self.id = id
    }

    pub fn notify(&self, result: Option<C::TaskError>) {
        self.reschedule_queue.0.push((self.id.clone(), result));
        self.reschedule_queue.1.notify_waiters()
    }
}

#[async_trait]
pub trait SchedulerTaskDispatcher<C: SchedulerConfig>: 'static + Send + Sync {
    async fn init(&self) {}

    async fn dispatch(
        &self,
        task: impl Deref<Target = ErasedTask<C::TaskError>> + Send + Sync + 'static,
        engine_notifier: &EngineNotifier<C>,
    );
    
    async fn cancel(&self, id: &C::TaskIdentifier);
}
