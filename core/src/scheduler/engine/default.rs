use crate::scheduler::SchedulerConfig;
use crate::scheduler::clock::SchedulerClock;
use crate::scheduler::engine::SchedulerEngine;
use crate::scheduler::task_dispatcher::SchedulerTaskDispatcher;
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::utils::RescheduleAlerter;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::join;

pub(crate) struct DefaultRescheduleAlerter<C: SchedulerConfig> {
    value: C::TaskIdentifier,
    sender: tokio::sync::mpsc::Sender<C::TaskIdentifier>,
}

#[async_trait]
impl<C: SchedulerConfig> RescheduleAlerter for DefaultRescheduleAlerter<C> {
    async fn notify_task_finish(&self) {
        self.sender
            .send(self.value.clone())
            .await
            .expect("Failed to send task finish");
    }
}

pub struct DefaultSchedulerEngine;

#[async_trait]
impl<C: SchedulerConfig> SchedulerEngine<C> for DefaultSchedulerEngine {
    async fn main(
        &self,
        clock: Arc<C::SchedulerClock>,
        store: Arc<C::SchedulerTaskStore>,
        dispatcher: Arc<C::SchedulerTaskDispatcher>,
    ) {
        let (scheduler_send, mut scheduler_receive) =
            tokio::sync::mpsc::channel::<C::TaskIdentifier>(1028);
        let notifier = tokio::sync::Notify::new();
        join!(
            async {
                while let Some(idx) = scheduler_receive.recv().await {
                    if let Some(_task) = store.get(&idx).await {
                        /*
                        if let Some(max_runs) = task.max_runs()
                            && task.runs() >= max_runs.get()
                        {
                            continue;
                        }
                         */
                        store.reschedule(&clock, &idx).await.expect(&format!(
                            "Failed to reschedule Task with the identifier {idx:?}"
                        ));
                        notifier.notify_waiters();
                    }
                }
            },
            async {
                loop {
                    if let Some((task, time, idx)) = store.retrieve().await {
                        tokio::select! {
                            _ = clock.idle_to(time) => {
                                store.pop().await;
                                if !store.exists(&idx).await { continue; }
                                let sender: DefaultRescheduleAlerter<C> = DefaultRescheduleAlerter {
                                    value: idx.clone(),
                                    sender: scheduler_send.clone()
                                };
                                dispatcher.dispatch(task, &sender).await;
                                continue;
                            }

                            _ = notifier.notified() => {
                                continue;
                            }
                        }
                    }
                }
            }
        );
    }
}
