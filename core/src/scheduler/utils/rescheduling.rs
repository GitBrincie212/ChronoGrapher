use std::sync::Arc;
use dashmap::DashSet;
use crate::prelude::SchedulerConfig;
use crate::scheduler::{assign_to_trigger_worker, ReschedulePayload, TriggerJobWorkers};
use crate::scheduler::task_store::SchedulerTaskStore;

#[inline(always)]
pub fn reschedule_logic<C: SchedulerConfig>(
    store: &Arc<C::SchedulerTaskStore>,
    mut scheduler_receive: tokio::sync::mpsc::Receiver<ReschedulePayload<C>>,
    workers: Arc<TriggerJobWorkers<C>>
) -> impl Future<Output = ()> + 'static {
    let store = store.clone();
    let workers = workers.clone();

    let blocked_ids: DashSet<C::TaskIdentifier> = DashSet::default();

    async move {
        while let Some((id, err)) = scheduler_receive.recv().await {
            if blocked_ids.contains(&id) {
                blocked_ids.remove(&id);
                continue;
            }

            match err {
                None => {
                    if let Some(task) = store.get(&id) {
                        assign_to_trigger_worker::<C>(task.trigger().clone(), &id, workers.as_ref()).await;
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
    }
}