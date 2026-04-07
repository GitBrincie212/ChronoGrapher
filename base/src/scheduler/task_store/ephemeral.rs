use std::any::TypeId;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Weak};
use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_store::{SchedulerTaskStore, TaskHookHandle, TaskRef};
use crate::task::{DynTaskFrame, StaleTaskHook, TaskDefinitions, TaskFrame, TaskHook, TaskHookErased, TaskHookEvent, TaskTrigger};
use dashmap::DashMap;
use crossbeam::sync::ShardedLock;
use slotmap::{new_key_type, SlotMap};
use crate::errors::TaskError;

new_key_type! {struct TaskRecordKey;}
new_key_type! {struct HookKey;}

type EphemeralTaskLayer<T> = ShardedLock<SlotMap<TaskRecordKey, EphemeralTaskRecord<T>>>;
type EphemeralTaskHookLayer = ShardedLock<SlotMap<HookKey, Box<dyn TaskHookErased>>>;
type EphemeralEventsLayer = DashMap<TaskRecordKey, HashMap<TypeId, VecDeque<HookKey>>>;


pub struct EphemeralTaskRecord<T: TaskError> {
    frame: Box<dyn DynTaskFrame<T>>,
    trigger: Box<dyn TaskTrigger>,
    key: TaskRecordKey
}

pub struct EphemeralTaskHandle<T: TaskError> {
    key: TaskRecordKey,
    tasks: Weak<EphemeralTaskLayer<T>>,
    hooks: Weak<EphemeralTaskHookLayer>,
    events: Weak<EphemeralEventsLayer>,
}

impl<T: TaskError> Clone for EphemeralTaskHandle<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            tasks: self.tasks.clone(),
            hooks: self.hooks.clone(),
            events: self.events.clone(),
        }
    }
}

impl<T: TaskError> PartialEq for EphemeralTaskHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.tasks.ptr_eq(&other.tasks)
            && self.hooks.ptr_eq(&other.hooks)
            && self.events.ptr_eq(&other.events)
    }
}

impl<T: TaskError> Eq for EphemeralTaskHandle<T> {}

pub struct EphemeralTaskHookHandle<T: TaskError> {
    hook_key: HookKey,
    task_key: TaskRecordKey,
    tasks: Weak<EphemeralTaskLayer<T>>,
    hooks: Weak<EphemeralTaskHookLayer>,
    events: Weak<EphemeralEventsLayer>,
}

impl<T: TaskError> Clone for EphemeralTaskHookHandle<T> {
    fn clone(&self) -> Self {
        Self {
            hook_key: self.hook_key.clone(),
            task_key: self.task_key.clone(),
            tasks: self.tasks.clone(),
            hooks: self.hooks.clone(),
            events: self.events.clone(),
        }
    }
}

impl<T: TaskError> PartialEq for EphemeralTaskHookHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.hook_key == other.hook_key
            && self.task_key == other.task_key
            && self.tasks.ptr_eq(&other.tasks)
            && self.hooks.ptr_eq(&other.hooks)
            && self.events.ptr_eq(&other.events)
    }
}

impl<T: TaskError> Eq for EphemeralTaskHookHandle<T> {}

impl<T: TaskError> TaskHookHandle for EphemeralTaskHookHandle<T> {
    fn get<E: TaskHookEvent, T: TaskHook<E>>(&self) -> impl Future<Output=&T> + Send {
        todo!()
    }

    fn get_dyn(&self) -> impl Future<Output=&dyn TaskHook<()>> + Send {
        todo!()
    }

    fn subscribe<E: TaskHookEvent>(&self) -> impl Future<Output=()> + Send {
        todo!()
    }

    fn unsubscribe<E: TaskHookEvent>(&self) -> impl Future<Output=()> + Send {
        todo!()
    }

    async fn emit<E: TaskHookEvent>(&self, payload: &E::Payload<'_>) {
        if let Some(events) = self.events.upgrade()
            && let Some(hooks) = self.hooks.upgrade()
            && let Some(hook) = hooks.read().unwrap().get(self.hook_key)
            && let Some(subscribed_events) = events.get(&self.task_key)
            && let Some(instances) = subscribed_events.get(&TypeId::of::<E>())
            && instances.contains(&self.hook_key)
        {
            hook.call_erased(ctx, payload).await;
        }
    }

    fn is_valid(&self) -> impl Future<Output=bool> + Send {
        std::future::ready(
            self.events.strong_count() == 0
                && self.hooks.strong_count() == 0
                && self.tasks.strong_count() == 0
        )
    }

    fn detach(&self) -> impl Future<Output=()> + Send {
        std::future::ready(())
    }
}

impl<T: TaskError> TaskRef for EphemeralTaskHandle<T> {
    type TaskError = T;
    type Key = TaskRecordKey;
    type TaskTrigger = Box<dyn TaskTrigger>;
    type TaskFrame = Box<dyn DynTaskFrame<T>>;
    type TaskHookHandle<T: StaleTaskHook> = ();

    fn frame(&self) -> impl Future<Output= Option<&Self::TaskFrame>> + Send {
        std::future::ready(
            self.tasks.upgrade().map(|tasks| {
                &tasks.read().unwrap().get(self.key)?.frame
            })
        )
    }

    fn trigger(&self) -> impl Future<Output=Option<&Self::TaskTrigger>> + Send {
        std::future::ready(
            self.tasks.upgrade().map(|tasks| {
                &tasks.read().unwrap().get(self.key)?.trigger
            })
        )
    }

    async fn attach_hook<T: StaleTaskHook>(&self, value: T) -> Option<Self::TaskHookHandle<T>> {
        self.hooks.upgrade().map(|hooks| {
            let hook = hooks.write().unwrap().insert(Box::new(value));

        })
    }

    fn detach_hook<T: StaleTaskHook>(&self) -> impl Future<Output=()> + Send {
        todo!()
    }

    fn detach_hook_from<E: TaskHookEvent, T: TaskHook<E>>(&self) -> impl Future<Output=()> + Send {
        todo!()
    }

    fn get_hook_from<E: TaskHookEvent, T: TaskHook<E>>(&self) -> impl Future<Output=Option<Self::TaskHookHandle<T>>> + Send {
        todo!()
    }

    fn get_hook<T: StaleTaskHook>(&self) -> impl Future<Output=Option<Self::TaskHookHandle<T>>> + Send {
        todo!()
    }

    fn emit_event<E: TaskHookEvent>(&self, payload: &E::Payload<'_>) -> impl Future<Output=()> + Send {
        todo!()
    }

    fn is_valid(&self) -> impl Future<Output=bool> + Send {
        todo!()
    }

    fn invalidate(&self) -> impl Future<Output=()> + Send {
        if let Some(tasks) = self.tasks.upgrade()
            && let Some(hooks) = self.hooks.upgrade()
            && let Some(events) = self.events.upgrade()
        {
            tasks.write().unwrap().remove(self.key);
            if let Some(hook_instances) = events.remove(&self.key) {
                hook_instances.1
                    .flatten()
                    .for_each(|x| {
                        hooks.write().unwrap().remove(x)
                    })
            }
        }
        std::future::ready(())
    }

    fn key(&self) -> &Self::Key {
        &self.key
    }
}

pub struct EphemeralSchedulerTaskStore<C: SchedulerConfig> {
    tasks: Arc<EphemeralTaskLayer<C::TaskError>>,
    hooks: Arc<EphemeralTaskHookLayer>,
    events: Arc<EphemeralEventsLayer>
}

impl<C: SchedulerConfig> Default for EphemeralSchedulerTaskStore<C> {
    fn default() -> Self {
        Self{
            tasks: Arc::new(ShardedLock::new(SlotMap::default())),
            hooks: Arc::new(ShardedLock::new(SlotMap::with_key())),
            events: Arc::new(DashMap::new())
        }
    }
}

impl<C: SchedulerConfig> SchedulerTaskStore<C> for EphemeralSchedulerTaskStore<C> {
    type TaskRef = EphemeralTaskHandle<C::TaskError>;

    async fn allocate<T1: TaskFrame<Error=C::TaskError>, T2: TaskTrigger>(
        &self, task: TaskDefinitions<T1, T2>
    ) -> Self::TaskRef {
        let map = DashMap::with_capacity(task.hooks.len());
        for (num, boxed) in task.hooks {
            let hook = self.hooks.write().unwrap().insert(boxed);
            map.insert(num, hook);
        }

        let key = self.tasks.write().unwrap()
            .insert_with_key(|key| EphemeralTaskRecord::<C> {
                frame: Box::new(task.frame),
                trigger: Box::new(task.trigger),
                key,
            });

        let mut map = HashMap::with_capacity(task.events.len());
        for (event, nums) in task.events {
            let instances = nums.into_iter()
                .map(|num| map.get(&num)
                    .expect("Unexpected Numeric Identifier For TaskHook Identified")
                    .value()
                    .clone()
                )
                .collect::<VecDeque<_>>();

            map.insert(event, instances);
        }

        self.events.insert(key.clone(), map);

        EphemeralTaskHandle::<C> {
            key,
            tasks: Arc::downgrade(&self.tasks),
            hooks: Arc::downgrade(&self.hooks),
            events: Arc::downgrade(&self.events),
        }
    }

    async fn resolve(&self, key: &<Self::TaskRef as TaskRef>::Key) -> Option<Self::TaskRef> {
        if !self.tasks.read().unwrap().contains_key(*key) {
            return None;
        }

        Some(EphemeralTaskHandle::<C> {
            key: key.clone(),
            tasks: Arc::downgrade(&self.tasks),
            hooks: Arc::downgrade(&self.hooks),
            events: Arc::downgrade(&self.events),
        })
    }

    async fn clear(&self) {
        self.tasks.write().unwrap().clear();
        self.hooks.write().unwrap().clear();
        self.events.clear()
    }
}