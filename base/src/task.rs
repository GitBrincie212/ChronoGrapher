pub mod dependency; // skipcq: RS-D1001

pub mod frames; // skipcq: RS-D1001

pub mod frame_builder; // skipcq: RS-D1001

pub mod hooks; // skipcq: RS-D1001

pub mod trigger; // skipcq: RS-D1001

use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};
pub use frame_builder::*;
pub use frames::*;
pub use hooks::*;
pub use trigger::*;
pub use schedule::*;

use crate::errors::TaskError;
#[allow(unused_imports)]
use crate::scheduler::Scheduler;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, LazyLock};
use std::sync::atomic::AtomicUsize;

static INSTANCE_ID: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

pub type ErasedTask<E> = Task<Box<dyn DynTaskFrame<E>>, Box<dyn TaskTrigger>>;

pub struct Task<T1, T2> {
    counter: usize,
    frame: T1,
    trigger: T2,
    events: HashMap<TypeId, VecDeque<usize>>,
    hooks: HashMap<usize, Box<dyn TaskHook<()>>>,
}

pub struct TaskDefinitions<T1: TaskFrame, T2: TaskTrigger> {
    pub frame: T1,
    pub trigger: T2,
    pub events: HashMap<TypeId, VecDeque<usize>>,
    pub hooks: HashMap<usize, Box<dyn TaskHook<()>>>,
}

impl<T1: TaskFrame, T2: TaskTrigger> From<Task<T1, T2>> for TaskDefinitions<T1, T2> {
    fn from(task: Task<T1, T2>) -> Self {
        Self {
            frame: task.frame,
            trigger: task.trigger,
            events: task.events,
            hooks: task.hooks,
        }
    }
}

pub struct TaskHookHandle<'a, T: TaskHook<()>> {
    key: usize,
    hooks: &'a mut HashMap<usize, Box<dyn TaskHook<()>>>,
    events: &'a mut HashMap<TypeId, VecDeque<usize>>,
    _marker: PhantomData<T>,
}

impl<T: TaskHook<()>> TaskHookHandle<'_, T> {
    pub fn get(&self) -> Option<&T> {
        let hook = self.hooks.get(&self.key)?.as_ref();
        <dyn Any>::downcast_ref::<T>(hook)
    }

    pub fn subscribe<E: TaskHookEvent>(&mut self) {
        self.events.entry(TypeId::of::<E>())
            .or_default()
            .push_back(self.key);
    }

    pub fn unsubscribe<E: TaskHookEvent>(&mut self) {
        if let Some(event) = self.events.get_mut(&TypeId::of::<E>()) {
            for i in 0..event.len() {
                if event[i] == self.key {
                    event.swap_remove_back(i);
                }
            }
        }
    }

    pub fn detach(&mut self) {
        self.hooks.remove(&self.key);
    }
}

impl<T1: TaskFrame + Default, T2: TaskTrigger + Default> Default for Task<T1, T2> {
    fn default() -> Self {
        Self {
            counter: 0,
            frame: T1::default(),
            trigger: T2::default(),
            events: HashMap::new(),
            hooks: HashMap::new(),
        }
    }
}

impl<T1: TaskFrame, T2: TaskTrigger> Task<T1, T2> {
    pub fn frame(&self) -> &T1 {
        &self.frame
    }

    pub fn trigger(&self) -> &T2  {
        &self.trigger
    }

    pub fn set_frame<T: TaskFrame>(self, frame: T) -> Task<T, T2> {
        Task::<T, T2> {
            counter: self.counter,
            frame,
            trigger: self.trigger,
            events: self.events,
            hooks: self.hooks,
        }
    }

    pub fn set_trigger<T: TaskTrigger>(self, trigger: T) -> Task<T1, T> {
        Task::<T1, T> {
            counter: self.counter,
            frame: self.frame,
            trigger,
            events: self.events,
            hooks: self.hooks,
        }
    }

    pub fn attach_hook<T: TaskHook<()>>(&mut self, hook: T) -> TaskHookHandle<'_, T> {
        self.hooks.insert(self.counter, Box::new(hook) as Box<dyn TaskHook<()>>);
        self.counter += 1;

        TaskHookHandle {
            key: self.counter,
            hooks: &mut self.hooks,
            events: &mut self.events,
            _marker: PhantomData,
        }
    }
}

impl<E: TaskError> ErasedTask<E> {
    pub fn run(&self) -> impl Future<Output = Result<(), E>> + Send {
        let ctx = TaskFrameContext(RestrictTaskFrameContext::new(self));
        async move {
            ctx.emit::<OnTaskStart>(&()).await; // skipcq: RS-E1015
            let result: Result<(), E> = self.frame.erased_execute(&ctx).await;
            let err = result.as_ref().err().map(|x| x as &dyn TaskError);
            ctx.emit::<OnTaskEnd>(&err).await;

            result
        }
    }
}