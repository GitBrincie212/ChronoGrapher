use std::sync::Arc;
use crossbeam::queue::SegQueue;
use tokio::sync::Notify;
use crate::scheduler::{assign_triggering_to_worker, ReschedulePayload, SchedulerConfig, SchedulerWorker};

#[inline(always)]
pub fn reschedule_logic<C: SchedulerConfig>(
    reschedule_queue: &Arc<(SegQueue<ReschedulePayload<C>>, Notify)>,
    workers: &Arc<Vec<SchedulerWorker<C>>>
) -> impl Future<Output = ()> + 'static {
    let workers = workers.clone();
    let reschedule_queue = reschedule_queue.clone();
    
    async move {
        loop {
            if let Some((handle, err)) = reschedule_queue.0.pop() {
                match err {
                    None => {
                        assign_triggering_to_worker::<C>(handle, workers.as_ref());
                    }

                    Some(err) => {
                        eprintln!("Scheduler engine received an error:\n\t {err:?}");
                    }
                }

                continue;
            }

            reschedule_queue.1.notified().await;
        }
    }
}