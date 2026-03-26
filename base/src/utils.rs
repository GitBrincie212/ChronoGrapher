pub mod identifier;
pub mod timing_wheel;
pub mod slotmap;

pub use identifier::*;
pub use timing_wheel::*;

pub(crate) mod macros {
    macro_rules! define_event {
        ($(#[$($attrss:tt)*])* $name: ident, $payload: ty) => {
            $(#[$($attrss)*])*
            #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
            pub struct $name;
    
            impl TaskHookEvent for $name {
                type Payload<'a> = $payload where Self: 'a;
            }
        };
    }
    
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
            pub trait $name<'a>: TaskHookEvent<Payload<'a> = $payload> {}
            $(
            impl<'a> $name<'a> for $events {}
            )*
        };
    }
    
    pub(crate) use define_event;
    pub(crate) use define_event_group;
}