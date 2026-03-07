use std::sync::Arc;
use crossbeam::queue::SegQueue;
use dashmap::DashSet;
use tokio::sync::Notify;
use crate::prelude::SchedulerConfig;
use crate::scheduler::{assign_to_trigger_worker, ReschedulePayload, SchedulerWorker};

#[inline(always)]
pub fn reschedule_logic<C: SchedulerConfig>(
    reschedule_queue: &Arc<(SegQueue<ReschedulePayload<C>>, Notify)>,
    workers: &Arc<Vec<SchedulerWorker<C>>>
) -> impl Future<Output = ()> + 'static {
    let workers = workers.clone();
    let reschedule_queue = reschedule_queue.clone();

    let blocked_ids: DashSet<C::TaskIdentifier> = DashSet::default();

    async move {
        loop {
            if let Some((id, err)) = reschedule_queue.0.pop() {
                if blocked_ids.contains(&id) {
                    blocked_ids.remove(&id);
                    continue;
                }

                match err {
                    None => {
                        assign_to_trigger_worker::<C>(id.clone(), workers.as_ref());
                    }

                    Some(err) => {
                        eprintln!(
                            "Scheduler engine received an error for Task with identifier ({:?}):\n\t {:?}",
                            id, err
                        );
                    }
                }

                continue;
            }

            reschedule_queue.1.notified().await;
        }
    }
}