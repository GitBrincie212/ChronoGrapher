use chrono::{DateTime, Local, TimeZone};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[macro_export]
macro_rules! define_generic_event {
    ($name: ident) => {
        #[doc =
            concat!(
                "[`", stringify!($name), "`] is an implementation of [`TaskHookEvent`] (a system used \
                closely with [`TaskHook`]). The concrete payload type of [`", stringify!($name),
                "`] is ``TaskHookEvent<P>``. Unlike most events, this is generic-based TaskEvent, it has \
                to do with lifecycle of TaskHooks"
            )
        ]
        ///
        /// # Constructor(s)
        #[doc =
            concat!(
                "When constructing a [`", stringify!($name), "`] due to the fact this is a marker ``struct``, \
                making it as such zero-sized, one can either use [`", stringify!($name), "::default`]
                or via simply pasting the struct name ([`", stringify!($name), "`])"
            )
        ]
        ///
        /// # Trait Implementation(s)
        #[doc =
            concat!(
                "It is obvious that [`", stringify!($name), "`] implements the [`TaskHookEvent`], but also many \
                other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`] \
                and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]"
            )
        ]
        ///
        /// # Cloning Semantics
        #[doc = concat!(
            "When cloning / copy a [`", stringify!($name), "`] it fully creates a \
            new independent version of that instance"
        )]
        ///
        /// # See Also
        /// - [`TaskHook`]
        /// - [`TaskHookEvent`]
        /// - [`Task`]
        /// - [`TaskFrame`]
        #[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
        pub struct $name<E: TaskHookEvent>(PhantomData<E>);

        impl<E: TaskHookEvent> Default for $name<E> {
            fn default() -> Self {
                $name(PhantomData)
            }
        }

        impl<E: TaskHookEvent> TaskHookEvent for $name<E> {
            type Payload = E;
            const PERSISTENCE_ID: &'static str = concat!("chronographer_core#", stringify!($name));
        }
    };
}

#[macro_export]
macro_rules! define_event {
    ($(#[$($attrss:tt)*])* $name: ident, $payload: ty) => {
        #[doc =
            concat!(
                "[`", stringify!($name), "`] is an implementation of [`TaskHookEvent`] (a system used \
                closely with [`TaskHook`]). The concrete payload type of [`", stringify!($name),
                "`] is ``", stringify!($payload), "``"
            )
        ]
        ///
        /// # Constructor(s)
        #[doc =
            concat!(
                "When constructing a [`", stringify!($name), "`] due to the fact this is a marker ``struct``, \
                making it as such zero-sized, one can either use [`", stringify!($name), "::default`]
                or via simply pasting the struct name ([`", stringify!($name), "`])"
            )
        ]
        ///
        /// # Trait Implementation(s)
        #[doc =
            concat!(
                "It is obvious that [`", stringify!($name), "`] implements the [`TaskHookEvent`], but also many \
                other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`] \
                and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]"
            )
        ]
        ///
        /// # Cloning Semantics
        #[doc = concat!(
            "When cloning / copy a [`", stringify!($name), "`] it fully creates a \
            new independent version of that instance"
        )]
        ///
        $(#[$($attrss)*])*
        /// - [`TaskHook`]
        /// - [`TaskHookEvent`]
        /// - [`Task`]
        /// - [`TaskFrame`]
        #[derive(Default, Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
        pub struct $name;
        impl<'a> TaskHookEvent for $name {
            type Payload = $payload;
            const PERSISTENCE_ID: &'static str = concat!("chronographer_core#", stringify!($name));
        }
    };
}

/// Simply converts the ``SystemTime`` to a ``DateTime<Local>``, it is a private
/// method used internally by ChronoGrapher, as such why it lives in utils module
pub(crate) fn system_time_to_date_time(t: SystemTime) -> DateTime<Local> {
    let (sec, nsec) = match t.duration_since(UNIX_EPOCH) {
        Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
        Err(e) => {
            let dur = e.duration();
            let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
            if nsec == 0 {
                (-sec, 0)
            } else {
                (-sec - 1, 1_000_000_000 - nsec)
            }
        }
    };
    Local.timestamp_opt(sec, nsec).unwrap()
}

/// Simply converts the ``DateTime<Local>`` to a ``SystemTime``, it is a private
/// method used internally by ChronoGrapher, as such why it lives in utils module
pub(crate) fn date_time_to_system_time(dt: DateTime<impl TimeZone>) -> SystemTime {
    let duration_since_epoch = dt.timestamp_nanos_opt().unwrap();
    if duration_since_epoch >= 0 {
        UNIX_EPOCH + Duration::from_nanos(duration_since_epoch as u64)
    } else {
        UNIX_EPOCH - Duration::from_nanos((-duration_since_epoch) as u64)
    }
}
