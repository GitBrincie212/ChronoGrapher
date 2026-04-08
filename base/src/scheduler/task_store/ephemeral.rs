use std::any::TypeId;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, Weak};
use crate::scheduler::SchedulerConfig;
use crate::scheduler::task_store::{SchedulerTaskStore, TaskHookHandle, TaskRef};
use crate::task::{DynTaskFrame, StaleTaskHook, TaskDefinitions, TaskFrame, TaskHook, TaskHookErased, TaskHookEvent, TaskTrigger};
use dashmap::DashMap;
use crossbeam::sync::ShardedLock;
use slotmap::{new_key_type, Key, SlotMap};
use crate::errors::TaskError;

new_key_type! {struct TaskRecordKey;}
new_key_type! {struct HookKey;}

type EphemeralTaskSlotMapLayer<T> = ShardedLock<SlotMap<TaskRecordKey, EphemeralTaskRecord<T>>>;
type EphemeralHookSlotMapLayer = ShardedLock<SlotMap<HookKey, Box<dyn StaleTaskHook>>>;
type EphemeralHookLookupLayer = DashMap<TaskRecordKey, EphemeralTaskHookRegistry>;

struct EphemeralTaskHookRegistry {
    by_type: HashMap<TypeId, VecDeque<HookKey>>,
    by_event: HashMap<TypeId, VecDeque<(TypeId, HookKey)>>,
}

pub struct EphemeralTaskRecord<T: TaskError> {
    frame: Box<dyn DynTaskFrame<T>>,
    trigger: Box<dyn TaskTrigger>,
    key: TaskRecordKey
}

pub struct EphemeralTaskHandle<T: TaskError> {
    key: TaskRecordKey,
    tasks: Weak<EphemeralTaskSlotMapLayer<T>>,
    hooks: Weak<EphemeralHookSlotMapLayer>,
    hook_lookup: Weak<EphemeralHookLookupLayer>,
}

impl<T: TaskError> Clone for EphemeralTaskHandle<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            tasks: self.tasks.clone(),
            hooks: self.hooks.clone(),
            hook_lookup: self.hook_lookup.clone(),
        }
    }
}

impl<T: TaskError> PartialEq for EphemeralTaskHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.tasks.ptr_eq(&other.tasks)
            && self.hooks.ptr_eq(&other.hooks)
            && self.hook_lookup.ptr_eq(&other.hook_lookup)
    }
}

impl<T: TaskError> Eq for EphemeralTaskHandle<T> {}

pub struct EphemeralTaskHookHandle<E: TaskError, T: StaleTaskHook> {
    hook_key: HookKey,
    task_key: TaskRecordKey,
    tasks: Weak<EphemeralTaskSlotMapLayer<T>>,
    hooks: Weak<EphemeralHookSlotMapLayer>,
    hook_lookup: Weak<EphemeralHookLookupLayer>,
}

impl<E: TaskError, T: StaleTaskHook> Clone for EphemeralTaskHookHandle<E, T> {
    fn clone(&self) -> Self {
        Self {
            hook_key: self.hook_key.clone(),
            task_key: self.task_key.clone(),
            tasks: self.tasks.clone(),
            hooks: self.hooks.clone(),
            hook_lookup: self.hook_lookup.clone(),
        }
    }
}

impl<E: TaskError, T: StaleTaskHook> PartialEq for EphemeralTaskHookHandle<E, T> {
    fn eq(&self, other: &Self) -> bool {
        self.hook_key == other.hook_key
            && self.task_key == other.task_key
            && self.tasks.ptr_eq(&other.tasks)
            && self.hooks.ptr_eq(&other.hooks)
            && self.hook_lookup.ptr_eq(&other.hook_lookup)
    }
}

impl<E: TaskError, T: StaleTaskHook> Eq for EphemeralTaskHookHandle<E, T> {}

impl<E: TaskError, T: StaleTaskHook> TaskHookHandle<T> for EphemeralTaskHookHandle<E, T> {
    fn get(&self) -> impl Future<Output=Option<&T>> + Send {
        std::future::ready(
            Some(
                self.hook_lookup.upgrade()?
                    .get(&self.task_key)?
                    .by_type
                    .get(&TypeId::of::<T>())?
            )
        )
    }

    fn get_from<EV: TaskHookEvent>(&self) -> impl Future<Output=Option<&T>> + Send {
        std::future::ready(
            Some(
                self.hook_lookup.upgrade()?
                    .get(&self.task_key)?
                    .by_type
                    .get(&TypeId::of::<T>())?
            )
        )
    }

    fn subscribe<EV: TaskHookEvent>(&self) -> impl Future<Output=()> + Send {
        if let Some(lookup) = self.hook_lookup.upgrade()
            && let Some(hooks) = self.hooks.upgrade()
            && let Some(tasks) = self.tasks.upgrade()
            && hooks.read().unwrap().contains_key(self.hook_key)
            && tasks.read().unwrap().contains_key(self.task_key)
            && let Some(mut subscribed) = lookup.get_mut(&self.task_key)
        {
            subscribed.by_event.entry(TypeId::of::<EV>())
                .or_default()
                .push_back((TypeId::of::<T>(), self.hook_key));
        }
        std::future::ready(())
    }

    fn unsubscribe<EV: TaskHookEvent>(&self) -> impl Future<Output=()> + Send {
        if let Some(lookup) = self.hook_lookup.upgrade()
            && let Some(mut subscribed) = lookup.get_mut(&self.task_key)
            && let Some(subscriptions) = subscribed.by_event.get_mut(&TypeId::of::<EV>())
            && let Some(index) = subscriptions.iter().position(|(_, value)| *value == self.hook_key)
        {
            subscriptions.remove(index);
        }
        std::future::ready(())
    }

    async fn emit<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) {
        if let Some(events) = self.hook_lookup.upgrade()
            && let Some(hooks) = self.hooks.upgrade()
            && let Some(hook) = hooks.read().unwrap().get(self.hook_key)
            && let Some(subscribed_events) = events.get(&self.task_key)
            && let Some(instances) = subscribed_events.by_event.get(&TypeId::of::<EV>())
            && instances.iter().find(|&(_, x)| *x == self.hook_key).is_some()
        {
            hook.call_erased(ctx, payload).await;
        }
    }

    fn is_valid(&self) -> impl Future<Output=bool> + Send {
        std::future::ready(
            self.hook_lookup.strong_count() != 0
                && self.hooks.strong_count() != 0
                && self.tasks.strong_count() != 0
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
    type TaskHookHandle<TH: StaleTaskHook> = ();

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

    async fn attach_hook<TH: StaleTaskHook>(&self, value: TH) -> Option<Self::TaskHookHandle<TH>> {
        self.hooks.upgrade().map(|hooks| {
            let hook = hooks.write().unwrap().insert(Box::new(value));
        })
    }

    fn detach_hook<TH: StaleTaskHook>(&self) -> impl Future<Output=()> + Send {
        todo!()
    }

    fn detach_hook_from<EV: TaskHookEvent, TH: TaskHook<EV>>(&self) -> impl Future<Output=()> + Send {
        todo!()
    }

    fn get_hook_from<EV: TaskHookEvent, TH: TaskHook<EV>>(&self) -> impl Future<Output=Self::TaskHookHandle<TH>> + Send {
        let mut handle = EphemeralTaskHookHandle::<T, TH> {
            hook_key: HookKey::null(),
            task_key: TaskRecordKey::null(),
            tasks: Weak::new(),
            hooks: Weak::new(),
            hook_lookup: Weak::new(),
        };

        let task_hook_type = TypeId::of::<TH>();

        if let Some(hook_lookup) = self.hook_lookup.upgrade()
            && let Some(local_hooks) = hook_lookup.get(&self.key)
            && let Some(hook_keys) = local_hooks.value().by_event.get(&TypeId::of::<EV>())
            && let Some(last_key) = hook_keys.iter().position(|(x, _)| *x == task_hook_type)
        {
            handle.hook_key = *last_key;
            handle.task_key = self.key;
            handle.tasks = self.tasks;
            handle.hooks = self.hooks;
            handle.hook_lookup = self.hook_lookup;
        }

        std::future::ready(handle)
    }

    fn get_hook<TH: StaleTaskHook>(&self) -> impl Future<Output= Self::TaskHookHandle<TH>> + Send {
        let mut handle = EphemeralTaskHookHandle::<T, TH> {
            hook_key: HookKey::null(),
            task_key: TaskRecordKey::null(),
            tasks: Weak::new(),
            hooks: Weak::new(),
            hook_lookup: Weak::new(),
        };

        if let Some(hook_lookup) = self.hook_lookup.upgrade()
            && let Some(local_hooks) = hook_lookup.get(&self.key)
            && let Some(hook_keys) = local_hooks.value().by_type.get(&TypeId::of::<TH>())
            && let Some(last_key) = hook_keys.value().back()
        {
            handle.hook_key = *last_key;
            handle.task_key = self.key;
            handle.tasks = self.tasks;
            handle.hooks = self.hooks;
            handle.hook_lookup = self.hook_lookup;
        }

        std::future::ready(handle)
    }

    async fn emit_event<EV: TaskHookEvent>(&self, payload: &EV::Payload<'_>) {
        if let Some(events) = self.hook_lookup.upgrade()
            && let Some(hooks) = self.hooks.upgrade()
            && let Some(task_events) = events.get(&self.key)
            && let Some(instances) = task_events.by_event.get(&TypeId::of::<EV>())
        {
            for (_, key) in instances {
                let Some(instance) = hooks.read().unwrap().get(key.clone()) else {
                    continue;
                };

                instance.call_erased(ctx, payload).await;
            }
        }
    }

    fn is_valid(&self) -> impl Future<Output=bool> + Send {
        std::future::ready(
            self.hook_lookup.strong_count() != 0
                && self.hooks.strong_count() != 0
                && self.tasks.strong_count() != 0
        )
    }

    fn deallocate(&self) -> impl Future<Output=()> + Send {
        if let Some(tasks) = self.tasks.upgrade()
            && let Some(slotmap_hooks) = self.hooks.upgrade()
            && let Some(lookup) = self.hook_lookup.upgrade()
            && let Some((_, mut registry)) = lookup.remove(&self.key)
        {
            tasks.write().unwrap().remove(self.key);
            registry.by_event.drain()
                .chain(registry.by_type.drain())
                .map(|mut x| x.1.drain(..))
                .flatten()
                .for_each(|x| {slotmap_hooks.write().unwrap().remove(x);})
        }
        std::future::ready(())
    }

    fn key(&self) -> &Self::Key {
        &self.key
    }
}

pub struct EphemeralSchedulerTaskStore<C: SchedulerConfig> {
    tasks: Arc<EphemeralTaskSlotMapLayer<C::TaskError>>,
    hooks: Arc<EphemeralHookSlotMapLayer>,
    hook_lookup: Arc<EphemeralHookLookupLayer>,
}

impl<C: SchedulerConfig> Default for EphemeralSchedulerTaskStore<C> {
    fn default() -> Self {
        Self{
            tasks: Arc::new(ShardedLock::new(SlotMap::default())),
            hooks: Arc::new(ShardedLock::new(SlotMap::with_key())),
            hook_lookup: Arc::new(DashMap::new()),
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

        let mut event_map = HashMap::with_capacity(task.events.len());

        for (event, nums) in task.events {
            let instances = nums.into_iter()
                .map(|num| event_map.get(&num)
                    .expect("Unexpected Numeric Identifier For TaskHook Identified")
                    .value()
                    .clone()
                )
                .collect::<VecDeque<_>>();

            event_map.insert(event, instances);
        }

        self.hook_lookup.insert(key.clone(), event_map);

        EphemeralTaskHandle::<C> {
            key,
            tasks: Arc::downgrade(&self.tasks),
            hooks: Arc::downgrade(&self.hooks),
            hook_lookup: Arc::downgrade(&self.hook_lookup),
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
            hook_lookup: Arc::downgrade(&self.hook_lookup),
        })
    }

    async fn clear(&self) {
        self.tasks.write().unwrap().clear();
        self.hooks.write().unwrap().clear();
        self.hook_lookup.clear()
    }
}