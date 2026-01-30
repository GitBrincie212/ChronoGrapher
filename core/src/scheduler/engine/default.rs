use crate::scheduler::SchedulerConfig;
use crate::scheduler::clock::SchedulerClock;
use crate::scheduler::engine::SchedulerEngine;
use crate::scheduler::task_dispatcher::{EngineNotifier, SchedulerTaskDispatcher};
use crate::scheduler::task_store::SchedulerTaskStore;
use crate::task::TaskError;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;
use tokio::join;

#[derive(Default, Debug, Clone, Copy)]
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
            tokio::sync::mpsc::channel::<(Box<dyn Any + Send + Sync>, Option<TaskError>)>(1024);
        let notifier = tokio::sync::Notify::new();
        join!(
            async {
                while let Some((id, err)) = scheduler_receive.recv().await {
                    let id = id.downcast_ref::<C::TaskIdentifier>()
                        .expect("Different type was used on EngineNotifier, which was meant as for an identifier");
                    match err {
                        None => {
                            if let Some(_task) = store.get(id).await {
                                /*
                                if let Some(max_runs) = task.max_runs()
                                    && task.runs() >= max_runs.get()
                                {
                                    continue;
                                }
                                 */
                                store.reschedule(&clock, id).await.unwrap_or_else(|_| {
                                    panic!("Failed to reschedule Task with the identifier {id:?}")
                                });
                                notifier.notify_waiters();
                            }
                        }

                        Some(err) => {
                            eprintln!(
                                "Scheduler engine received an error for Task with identifier ({:?}):\n\t {:?}",
                                id, err
                            );
                        }
                    }
                }
            },
            async {
                loop {
                    if let Some((task, time, id)) = store.retrieve().await {
                        tokio::select! {
                            _ = clock.idle_to(time) => {
                                store.pop().await;
                                if !store.exists(&id).await { continue; }
                                let sender = EngineNotifier::new::<C>(
                                    id,
                                    scheduler_send.clone()
                                );
                                dispatcher.dispatch(task, sender).await;
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
