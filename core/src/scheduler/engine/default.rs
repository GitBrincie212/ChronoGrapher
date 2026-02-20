use std::any::{type_name, Any};
use crate::scheduler::SchedulerConfig;
use crate::scheduler::clock::SchedulerClock;
use crate::scheduler::engine::SchedulerEngine;
use crate::scheduler::task_dispatcher::{EngineNotifier, SchedulerTaskDispatcher};
use crate::scheduler::task_store::{RescheduleError, SchedulerTaskStore};
use async_trait::async_trait;
use std::sync::Arc;
use dashmap::DashSet;
use tokio::join;
use crate::prelude::TaskHook;

#[derive(Default, Debug, Clone, Copy)]
pub struct DefaultSchedulerEngine;

type SchedulerHandlePayload = (Arc<dyn Any + Send + Sync>, SchedulerHandleInstructions);

pub enum SchedulerHandleInstructions {
    Reschedule,   // Forces the Task to reschedule (instances may still run)
    Halt,         // Cancels the Task's current execution, if any
    Block,        // Blocks the Task from rescheduling
    Execute       // Spawns a new instance of the Task to run
}

pub(crate) struct SchedulerHandle {
    pub(crate) id: Arc<dyn Any + Send + Sync>,
    pub(crate) channel: tokio::sync::mpsc::Sender<SchedulerHandlePayload>
}

impl SchedulerHandle {
    pub(crate) async fn instruct(&self, instruction: SchedulerHandleInstructions) {
        self.channel.send((self.id.clone(), instruction)).await.expect("Cannot instruct");
    }
}

impl TaskHook<()> for SchedulerHandle {}

#[async_trait]
impl<C: SchedulerConfig> SchedulerEngine<C> for DefaultSchedulerEngine {
    async fn main(
        &self,
        clock: Arc<C::SchedulerClock>,
        store: Arc<C::SchedulerTaskStore>,
        dispatcher: Arc<C::SchedulerTaskDispatcher>,
    ) {
        let (scheduler_send, mut scheduler_receive) =
            tokio::sync::mpsc::channel::<(C::TaskIdentifier, Option<C::TaskError>)>(20480);
        let (instruct_send, mut instruct_receive) =
            tokio::sync::mpsc::channel::<SchedulerHandlePayload>(1024);
        let notifier = tokio::sync::Notify::new();

        let blocked_ids: DashSet<C::TaskIdentifier> = DashSet::default();

        join!(
            // ============================
            // Scheduler Instructions
            // ============================
            async {
                while let Some((id, instruction)) = instruct_receive.recv().await {
                    let id = id.downcast_ref::<C::TaskIdentifier>().expect(
                        &format!(
                            "Cannot downcast to TaskIdentifier of type {:?}",
                            type_name::<C::TaskIdentifier>()
                        )
                    );

                    match instruction {
                        SchedulerHandleInstructions::Reschedule => {
                            match store.reschedule(clock.as_ref(), id).await {
                                RescheduleError::Success => {}
                                RescheduleError::TriggerError(err) => {
                                    eprintln!(
                                        "{}",
                                        format!(
                                            "Failed reschedule via instruction the task(identifier \
                                                    being \"{id:?}\") with error:\n\t{err:?}")
                                    )
                                }
                                RescheduleError::UnknownTask => {}
                            }
                        }

                        SchedulerHandleInstructions::Halt => {

                        }

                        SchedulerHandleInstructions::Block => {
                            store.remove(id).await;
                        }

                        SchedulerHandleInstructions::Execute => {

                        }
                    }
                }
            },

            // ============================
            // Reschedule Logic
            // ============================
            async {
                while let Some((id, err)) = scheduler_receive.recv().await {
                    if blocked_ids.contains(&id) {
                        blocked_ids.remove(&id);
                        continue;
                    }

                    match err {
                        None => {
                            if let Some(_task) = store.get(&id).await {
                                match store.reschedule(&clock, &id).await {
                                    RescheduleError::Success => {}
                                    RescheduleError::TriggerError(_) => {
                                        eprintln!("Failed to reschedule Task with the identifier {id:?}")
                                    }
                                    RescheduleError::UnknownTask => {}
                                }
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

            // ============================
            // Engine Loop
            // ============================
            async {
                loop {
                    let (task, time, id) = store.retrieve().await;
                    tokio::select! {
                        _ = clock.idle_to(time) => {
                            store.pop().await;
                            if !store.exists(&id).await { continue; }
                            let sender = EngineNotifier::new(
                                id.clone(),
                                scheduler_send.clone()
                            );
                            let handle = SchedulerHandle {
                                id: Arc::new(id),
                                channel: instruct_send.clone()
                            };
                            task.attach_hook::<()>(Arc::new(handle)).await;
                            let dispatcher = dispatcher.clone();
                            tokio::spawn(async move {dispatcher.dispatch(task, sender).await;});
                        }

                        _ = notifier.notified() => {
                            continue;
                        }
                    }
                }
            },
        );
    }
}
