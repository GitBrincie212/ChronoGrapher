pub mod default; // skipcq: RS-D1001

pub use default::*;
use std::ops::Deref;

#[allow(unused_imports)]
use crate::scheduler::Scheduler;
use crate::scheduler::SchedulerConfig;
use crate::task::ErasedTask;
use async_trait::async_trait;

pub struct EngineNotifier<C: SchedulerConfig> {
    id: C::TaskIdentifier,
    notify: tokio::sync::mpsc::Sender<(C::TaskIdentifier, Option<C::TaskError>)>,
}

impl<C: SchedulerConfig> Clone for EngineNotifier<C> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            notify: self.notify.clone(),
        }
    }
}

impl<C: SchedulerConfig> EngineNotifier<C> {
    pub fn new(
        id: C::TaskIdentifier,
        notify: tokio::sync::mpsc::Sender<(C::TaskIdentifier, Option<C::TaskError>)>,
    ) -> Self {
        Self { id, notify }
    }

    pub fn new_id(&mut self, id: C::TaskIdentifier) {
        self.id = id
    }

    pub async fn notify(&self, result: Option<C::TaskError>) {
        self.notify
            .send((self.id.clone(), result))
            .await
            .expect("Failed to send notification via SchedulerTaskDispatcher, could not receive from the SchedulerEngine");
    }
}

#[async_trait]
pub trait SchedulerTaskDispatcher<C: SchedulerConfig>: 'static + Send + Sync {
    async fn init(&self) {}

    async fn dispatch(
        &self,
        task: impl Deref<Target = ErasedTask<C::TaskError>> + Send + Sync + 'static,
        notifier: &EngineNotifier<C>,
    );
}
