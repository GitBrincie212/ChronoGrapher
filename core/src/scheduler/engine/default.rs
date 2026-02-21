use std::any::type_name;
use crate::scheduler::SchedulerConfig;
use crate::scheduler::clock::SchedulerClock;
use crate::scheduler::engine::{SchedulerEngine, SchedulerHandlePayload};
use crate::scheduler::task_dispatcher::{EngineNotifier, SchedulerTaskDispatcher};
use crate::scheduler::task_store::{RescheduleError, SchedulerTaskStore};
use async_trait::async_trait;
use std::sync::Arc;
use dashmap::DashSet;
use tokio::join;

#[derive(Default, Debug, Clone, Copy)]
pub struct DefaultSchedulerEngine;

pub enum SchedulerHandleInstructions {
    Reschedule,   // Forces the Task to reschedule (instances may still run)
    Halt,         // Cancels the Task's current execution, if any
    Block,        // Blocks the Task from rescheduling
    Execute       // Spawns a new instance of the Task to run
}

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
        let notifier = tokio::sync::Notify::new();

        let blocked_ids: DashSet<C::TaskIdentifier> = DashSet::default();

        join!(
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
                            if let Some(_task) = store.get(&id) {
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
                            if !store.exists(&id) { continue; }

                            let sender = EngineNotifier::new(
                                id.clone(),
                                scheduler_send.clone(),
                            );

                            let dispatcher = dispatcher.clone();
                            tokio::spawn(async move {
                                dispatcher.dispatch(task, &sender).await;
                            });
                        }

                        _ = notifier.notified() => {
                            continue;
                        }
                    }
                }
            },
        );
    }

    async fn create_instruction_channel(
        &self,
        clock: &Arc<C::SchedulerClock>,
        store: &Arc<C::SchedulerTaskStore>,
        dispatcher: &Arc<C::SchedulerTaskDispatcher>,
    ) -> tokio::sync::mpsc::Sender<SchedulerHandlePayload> {
        let (instruct_send, mut instruct_receive) =
            tokio::sync::mpsc::channel::<SchedulerHandlePayload>(1024);

        let clock = clock.clone();
        let store = store.clone();
        let _dispatcher = dispatcher.clone();

        tokio::spawn(
            async move {
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
        );

        instruct_send
    }
}
