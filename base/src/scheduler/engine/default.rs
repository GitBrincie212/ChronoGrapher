use crate::scheduler::SchedulerConfig;
use crate::scheduler::clock::SchedulerClock;
use crate::scheduler::engine::SchedulerEngine;
use crate::utils::hierarchical_timing_wheel::HierarchicalTimingWheel;
use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};
use tokio::sync::Notify;

enum WheelCommand<C: SchedulerConfig> {
    Insert(C::TaskIdentifier, Duration),
    Clear,
}

type ResultQueue<C> = (SegQueue<Vec<<C as SchedulerConfig>::TaskIdentifier>>, Notify, AtomicUsize);

pub struct DefaultSchedulerEngine<C: SchedulerConfig> {
    command_batch: Arc<SegQueue<WheelCommand<C>>>,
    get_result_queue: Arc<ResultQueue<C>>,
    clock: Arc<C::SchedulerClock>,
}

impl<C: SchedulerConfig> Default for DefaultSchedulerEngine<C>
where
    C::SchedulerClock: Default,
{
    fn default() -> Self {
        let clock = Arc::new(C::SchedulerClock::default());

        let mut hierarchical_wheel =
            HierarchicalTimingWheel::<C::TaskIdentifier>::default();

        let command_batch = Arc::new(SegQueue::new());
        let get_result_queue = Arc::new((SegQueue::new(), Notify::new(), AtomicUsize::new(0)));

        let clock_clone = clock.clone();
        let batch_clone = command_batch.clone();
        let get_result_queue_clone = get_result_queue.clone();
        tokio::spawn(async move {
            loop {
                clock_clone.tick().await;
                while let Some(command) = batch_clone.pop() {
                    match command {
                        WheelCommand::Insert(val, pos) => {
                            hierarchical_wheel.insert(val, pos);
                        }

                        WheelCommand::Clear => hierarchical_wheel.clear(),
                    }
                }
                get_result_queue_clone.0.push(hierarchical_wheel.tick());
                get_result_queue_clone.2.fetch_add(1, Ordering::Release);
                get_result_queue_clone.1.notify_waiters()
            }
        });

        Self {
            clock,
            command_batch,
            get_result_queue,
        }
    }
}

#[async_trait]
impl<C: SchedulerConfig> SchedulerEngine<C> for DefaultSchedulerEngine<C> {
    async fn retrieve(&self) -> Vec<C::TaskIdentifier> {
        loop {
            if let Some(res) = self.get_result_queue.0.pop() {
                return res;
            }
            self.get_result_queue.1.notified().await;
        }
    }

    fn is_empty(&self) -> bool {
        self.get_result_queue.2.load(Ordering::Relaxed) == 0
    }

    fn clock(&self) -> &C::SchedulerClock {
        self.clock.as_ref()
    }

    async fn schedule(
        &self,
        id: &C::TaskIdentifier,
        time: SystemTime,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let now = self.clock.now();
        self.command_batch.push(WheelCommand::Insert(
            id.clone(),
            time.duration_since(now).unwrap_or(Duration::ZERO),
        ));
        Ok(())
    }

    async fn clear(&self) {
        self.command_batch.push(WheelCommand::Clear);
    }
}