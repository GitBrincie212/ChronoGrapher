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

    macro_rules! payload_wrapper {
        ($(#[$($attrss:tt)*])* $name: ident ($($toks: tt)*)) => {
            $(#[$($attrss)*])*
            #[repr(transparent)]
            pub struct $name($($toks)+);

            impl From<$name> for $($toks)+ {
                fn from(value: $name) -> Self {
                    value.0
                }
            }

            impl Deref for $name {
                type Target = $($toks)+;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        };

        ($(#[$($attrss:tt)*])* $name: ident <$($lt: lifetime),*> ($($toks: tt)*)) => {
            $(#[$($attrss)*])*
            #[repr(transparent)]
            pub struct $name<$($lt),*>($($toks)+);

            impl<$($lt),*> From<$name<$($lt),*>> for $($toks)+ {
                fn from(value: $name<$($lt),*>) -> Self {
                    value.0
                }
            }

            impl<$($lt),*> Deref for $name<$($lt),*> {
                type Target = $($toks)+;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        };
    }

    pub(crate) use define_event;
    pub(crate) use define_event_group;
    pub(crate) use payload_wrapper;
}
