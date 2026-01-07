use dashmap::DashMap;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tokio::time::Instant;

#[derive(Debug, Clone)]
struct CoreTask<Id, P>
where
    Id: Clone,
    P: Clone,
{
    id: Id,
    payload: Arc<P>,
    execution_time: SystemTime,
    circle: u32,
    diff: u32,
    removed: bool,
}

#[derive(Debug, Clone)]
struct PositionEntry<Id, P>
where
    Id: Clone,
    P: Clone,
{
    task: CoreTask<Id, P>,
    position: usize,
}

/// Generic timing wheel core that is independent from SchedulerConfig.
/// It manages insertion, movement and expiration of payloads keyed by Id.
pub struct TimingWheelCore<Id, P>
where
    Id: Eq + Hash + Clone + Send + Sync + 'static,
    P: Clone + Send + Sync + 'static,
{
    interval: Duration,
    num_slots: usize,
    slots: Vec<RwLock<VecDeque<CoreTask<Id, P>>>>,
    task_positions: DashMap<Id, PositionEntry<Id, P>>,
    ticked_pos: RwLock<usize>,
    start_time: RwLock<Instant>,
}

impl<Id, P> TimingWheelCore<Id, P>
where
    Id: Eq + Hash + Clone + Send + Sync + 'static,
    P: Clone + Send + Sync + 'static,
{
    pub fn new(interval: Duration, num_slots: usize) -> Self {
        assert!(num_slots > 0, "Number of slots must be greater than 0");
        let mut slots = Vec::with_capacity(num_slots);
        for _ in 0..num_slots {
            slots.push(RwLock::new(VecDeque::new()));
        }
        Self {
            interval,
            num_slots,
            slots,
            task_positions: DashMap::new(),
            ticked_pos: RwLock::new(num_slots - 1),
            start_time: RwLock::new(Instant::now()),
        }
    }

    async fn get_position_and_circle(&self, delay: Duration) -> (usize, u32) {
        let steps = (delay.as_millis() / self.interval.as_millis()).max(1) as u32;
        let current_pos = *self.ticked_pos.read().await;
        let pos = (current_pos + steps as usize) % self.num_slots;
        let circle = (steps - 1) / self.num_slots as u32;
        (pos, circle)
    }

    pub async fn insert(&self, id: Id, payload: Arc<P>, execution_time: SystemTime) -> Id {
        let start_time = *self.start_time.read().await;
        let now = Instant::now();
        let elapsed = now.duration_since(start_time);
        let delay = execution_time
            .duration_since(SystemTime::now())
            .unwrap_or_else(|_| Duration::from_millis(0))
            .saturating_add(elapsed);
        let (pos, circle) = self.get_position_and_circle(delay).await;
        let timing_task = CoreTask {
            id: id.clone(),
            payload,
            execution_time,
            circle,
            diff: 0,
            removed: false,
        };
        {
            let mut slot = self.slots[pos].write().await;
            slot.push_back(timing_task.clone());
        }
        self.task_positions.insert(
            id.clone(),
            PositionEntry {
                task: timing_task,
                position: pos,
            },
        );
        id
    }

    pub async fn move_task(&self, id: Id, new_execution_time: SystemTime) -> Result<(), ()> {
        let (task, old_pos) = match self.task_positions.get(&id) {
            Some(e) => {
                let task = e.task.clone();
                let old_pos = e.position;
                drop(e);
                (task, old_pos)
            }
            None => return Err(()),
        };
        let mut task = task;

        let start_time = *self.start_time.read().await;
        let now = Instant::now();
        let elapsed = now.duration_since(start_time);
        let delay = new_execution_time
            .duration_since(SystemTime::now())
            .map_err(|_| ())?
            .saturating_add(elapsed);

        if delay < self.interval {
            task.removed = true;
            self.task_positions.remove(&id);
            return Ok(());
        }

        let (new_pos, new_circle) = self.get_position_and_circle(delay).await;
        {
            let mut old_slot = self.slots[old_pos].write().await;
            old_slot.retain(|t| t.id != id);
        }
        task.execution_time = new_execution_time;
        task.circle = new_circle;

        if new_pos >= old_pos {
            task.diff = (new_pos - old_pos) as u32;
        } else if new_circle > 0 {
            task.circle = new_circle - 1;
            task.diff = (self.num_slots + new_pos - old_pos) as u32;
        } else {
            task.removed = true;
            let mut new_task = task.clone();
            new_task.removed = false;
            new_task.circle = 0;
            new_task.diff = 0;
            {
                let mut new_slot = self.slots[new_pos].write().await;
                new_slot.push_back(new_task.clone());
            }
            self.task_positions.insert(
                id,
                PositionEntry {
                    task: new_task,
                    position: new_pos,
                },
            );
            return Ok(());
        }
        {
            let mut new_slot = self.slots[new_pos].write().await;
            new_slot.push_back(task.clone());
        }
        self.task_positions.insert(
            id,
            PositionEntry {
                task,
                position: new_pos,
            },
        );
        Ok(())
    }

    pub async fn peek_ready(&self) -> Option<(Arc<P>, SystemTime, Id)> {
        let current_pos = *self.ticked_pos.read().await;
        for offset in 0..self.num_slots {
            let check_pos = (current_pos + offset) % self.num_slots;
            let slot = self.slots[check_pos].read().await;
            for task in slot.iter() {
                if !task.removed && task.circle == 0 && task.diff == 0 {
                    return Some((task.payload.clone(), task.execution_time, task.id.clone()));
                }
            }
        }
        None
    }

    pub async fn tick(&self) -> Vec<(Arc<P>, SystemTime, Id)> {
        {
            let mut start_time = self.start_time.write().await;
            *start_time += self.interval;
        }

        let mut ticked_pos = self.ticked_pos.write().await;
        *ticked_pos = (*ticked_pos + 1) % self.num_slots;
        let current_pos = *ticked_pos;
        drop(ticked_pos);

        let mut ready = Vec::new();
        let mut to_reschedule: Vec<(CoreTask<Id, P>, usize)> = Vec::new();
        {
            let mut slot = self.slots[current_pos].write().await;
            let mut i = 0;
            while i < slot.len() {
                let mut task = slot[i].clone();
                if task.removed {
                    slot.remove(i);
                    self.task_positions.remove(&task.id);
                    continue;
                }
                if task.circle > 0 {
                    task.circle -= 1;
                    i += 1;
                    continue;
                }
                if task.diff > 0 {
                    let next_pos = (current_pos + task.diff as usize) % self.num_slots;
                    task.diff = 0;
                    to_reschedule.push((task, next_pos));
                    slot.remove(i);
                    continue;
                }
                let id = task.id.clone();
                let payload = task.payload.clone();
                let execution_time = task.execution_time;
                ready.push((payload, execution_time, id));
                slot.remove(i);
                self.task_positions.remove(&task.id);
            }
        }
        for (task, pos) in to_reschedule {
            {
                let mut target = self.slots[pos].write().await;
                target.push_back(task.clone());
            }
            self.task_positions.insert(
                task.id.clone(),
                PositionEntry {
                    task,
                    position: pos,
                },
            );
        }
        ready
    }

    pub async fn get(&self, id: &Id) -> Option<Arc<P>> {
        self.task_positions.get(id).map(|e| e.task.payload.clone())
    }

    pub async fn exists(&self, id: &Id) -> bool {
        self.task_positions.contains_key(id)
    }

    pub async fn remove(&self, id: &Id) {
        if let Some((_, position_entry)) = self.task_positions.remove(id) {
            let mut slot = self.slots[position_entry.position].write().await;
            for task in slot.iter_mut() {
                if task.id == *id {
                    task.removed = true;
                    break;
                }
            }
        }
    }

    pub async fn clear(&self) {
        for slot in &self.slots {
            slot.write().await.clear();
        }
        self.task_positions.clear();
        *self.ticked_pos.write().await = self.num_slots - 1;
        *self.start_time.write().await = Instant::now();
    }
}
