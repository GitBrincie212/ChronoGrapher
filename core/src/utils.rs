use crate::prelude::NonObserverTaskHook;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;
use uuid::Uuid;

pub struct SharedHook<T: Send + Sync + 'static>(pub T);
impl<T: Send + Sync + 'static> NonObserverTaskHook for SharedHook<T> {}

#[macro_export]
macro_rules! define_event_group {
    ($(#[$($attrss:tt)*])* $name: ident, $($events: ident),*) => {
        $(#[$($attrss)*])*
        pub trait $name: TaskHookEvent {}

        $(
            impl $name for $events {}
        )*
    };

    ($(#[$($attrss:tt)*])* $name: ident, $payload: ty | $($events: ident),*) => {
        $(#[$($attrss)*])*
        pub trait $name: TaskHookEvent<Payload = $payload> {}

        $(
            impl $name for $events {}
        )*
    };
}

#[macro_export]
macro_rules! define_event {
    ($(#[$($attrss:tt)*])* $name: ident, $payload: ty) => {
        $(#[$($attrss)*])*
        #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name;
        impl<'a> TaskHookEvent for $name {
            type Payload = $payload;
            const EVENT_ID: &'static str = concat!("chronographer_core#", stringify!($name));
        }
    };
}

#[macro_export]
macro_rules! create_shared {
    ($ctx: expr => $init:expr => $block:expr) => {{
        let shared = Arc::new(SharedHook(Arc::new($init)));
        $ctx.attach_hook::<()>(shared.clone());
        $block(shared.0.clone()).await?;
        $ctx.detach_hook::<(), SharedHook<_>>();
    }};
}

pub trait Timestamp: Clone + Ord + Send + Sync + 'static {
    fn now() -> Self;
    fn duration_since(&self, earlier: Self) -> Option<Duration>;
    fn year(&self) -> u32;
    fn month(&self) -> u8;
    fn day(&self) -> u8;
    fn hour(&self) -> u8;
    fn minute(&self) -> u8;
    fn second(&self) -> u8;
    fn millisecond(&self) -> u16;
}

pub trait TaskIdentifier:
    'static + Debug + Clone + Eq + PartialEq<Self> + Hash + Send + Sync
{
    fn generate() -> Self;
}

impl TaskIdentifier for Uuid {
    fn generate() -> Self {
        Uuid::new_v4()
    }
}
