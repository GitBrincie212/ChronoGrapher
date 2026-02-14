pub mod default; // skipcq: RS-D1001

use std::ops::Deref;
pub use default::*;

#[allow(unused_imports)]
use crate::scheduler::Scheduler;
use crate::scheduler::SchedulerConfig;
use crate::task::ErasedTask;
use async_trait::async_trait;

pub struct EngineNotifier<C: SchedulerConfig> {
    id: C::TaskIdentifier,
    notify: tokio::sync::mpsc::Sender<(C::TaskIdentifier, Option<C::Error>)>,
}

impl<C: SchedulerConfig> EngineNotifier<C> {
    pub fn new(
        id: C::TaskIdentifier,
        notify: tokio::sync::mpsc::Sender<(C::TaskIdentifier, Option<C::Error>)>,
    ) -> Self {
        Self {
            id,
            notify,
        }
    }

    pub async fn notify(self, result: Option<C::Error>) {
        self.notify
            .send((self.id, result))
            .await
            .expect("Failed to send notification via SchedulerTaskDispatcher, could not receive from the SchedulerEngine");
    }
}

#[async_trait]
pub trait SchedulerTaskDispatcher<C: SchedulerConfig>: 'static + Send + Sync {
    async fn init(&self) {}

    async fn dispatch(
        &self,
        task: impl Deref<Target = ErasedTask<C::Error>> + Send + Sync + 'static,
        notifier: EngineNotifier<C>
    );
}
