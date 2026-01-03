use crate::scheduler::SchedulerConfig;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use std::hash::Hash;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

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

impl Timestamp for SystemTime {
    fn now() -> Self {
        SystemTime::now()
    }

    fn duration_since(&self, earlier: Self) -> Option<Duration> {
        self.duration_since(earlier).ok()
    }

    fn year(&self) -> u32 {
        system_time_to_date_time(self).year() as u32
    }

    fn month(&self) -> u8 {
        system_time_to_date_time(self).month0() as u8
    }

    fn day(&self) -> u8 {
        system_time_to_date_time(self).day0() as u8
    }

    fn hour(&self) -> u8 {
        system_time_to_date_time(self).hour() as u8
    }

    fn minute(&self) -> u8 {
        system_time_to_date_time(self).minute() as u8
    }

    fn second(&self) -> u8 {
        system_time_to_date_time(self).second() as u8
    }

    fn millisecond(&self) -> u16 {
        system_time_to_date_time(self).timestamp_subsec_millis() as u16
    }
}

pub trait TaskIdentifier: 'static + Clone + Eq + Hash + Send + Sync {
    fn generate() -> Self;
}

impl TaskIdentifier for Uuid {
    fn generate() -> Self {
        Uuid::new_v4()
    }
}

#[async_trait]
pub trait RescheduleAlerter: 'static + Send + Sync {
    async fn notify_task_finish(&self);
}

/// Simply converts the ``SystemTime`` to a ``DateTime<Local>``, it is a private
/// method used internally by ChronoGrapher, as such why it lives in utils module
pub(crate) fn system_time_to_date_time(t: &SystemTime) -> DateTime<Local> {
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
