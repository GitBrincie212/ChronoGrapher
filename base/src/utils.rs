pub mod timing_wheel;
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
            #[doc(hidden)]
            mod sealed {
                #[doc(hidden)]
                pub trait Sealed {}
            }

            $(
            impl sealed::Sealed for $events {}
            )*

            $(#[$($attrss)*])*
            pub trait $name: TaskHookEvent + sealed::Sealed {}
            $(
            impl $name for $events {}
            )*
        };

        ($(#[$($attrss:tt)*])* $name: ident, $payload: ty | $($events: ident),*) => {
            #[doc(hidden)]
            mod sealed {
                #[doc(hidden)]
                pub trait Sealed {}
            }

            $(
            impl sealed::Sealed for $events {}
            )*

            $(#[$($attrss)*])*
            pub trait $name<'a>: TaskHookEvent<Payload<'a> = $payload> + sealed::Sealed {}
            $(
            impl<'a> $name<'a> for $events {}
            )*
        };
    }

    pub(crate) use define_event;
    pub(crate) use define_event_group;
}
