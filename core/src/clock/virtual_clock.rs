use crate::clock::{AdvanceableScheduleClock, SchedulerClock};
use crate::utils::system_time_to_date_time;
use async_trait::async_trait;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Notify;

/// [`VirtualClock`] is an implementation of the [`SchedulerClock`] trait, it acts as a mock object, allowing
/// to simulate time without the waiting around. This can especially be useful for unit tests,
/// simulations of a [`flashcrowd`](https://en.wiktionary.org/wiki/flashcrowd#English) and so on
///
/// Unlike [`SystemClock`], this clock doesn't move forward, rather it needs explicit
/// calls to advance methods ([`VirtualClock`] implements the [`AdvanceableScheduleClock`] extension
/// trait), which makes it predictable at any point throughout the program
///
/// # Constructor(s)
/// When constructing a [`VirtualClock`], one can use a variety of constructor methods, those being:
/// - [`VirtualClock::new`] For creating one based an initial time based on [`SystemTime`]
/// - [`VirtualClock::from_value`] For creating one based on the supplied ``u64`` number (as milliseconds)
/// - [`VirtualClock::from_current_time`] For creating one based on the current time
/// - [`VirtualClock::from_epoch`] For creating one based on the (UNIX Epoch)[https://en.wikipedia.org/wiki/Unix_time]
///
/// # Trait Implementation(s)
/// It is clear as day, that [`VirtualClock`] implements the [`SchedulerClock`] but it also implements
/// the [`AdvanceableScheduleClock`] extension trait and the [`Debug`] trait
///
/// # Example
/// ```ignore
/// use std::thread::sleep;
/// use std::time::{Duration, SystemTime};
/// use chronographer_core::clock::VirtualClock;
///
/// let clock = VirtualClock::from_value(0);
///
/// // === Some Other Place ===
/// clock.idle_to(SystemTime::now()).await;
/// // === Some Other Place ===
///
/// sleep(Duration::from_secs_f64(2.3)); // Suppose time passes
/// assert_eq!(clock.now().await, 0);
/// clock.advance(Duration::from_secs(1)).await // Manual Advancement
/// assert_eq!(clock.now().await, 1);
/// clock.advance_to(SystemTime::now()).await // Manual Advancement (Stops the idling)
/// assert_eq!(clock.now().await, SystemTime::now());
/// ```
///
/// # See Also
/// - [`SystemClock`]
/// - [`AdvanceableScheduleClock`]
/// - [`SchedulerClock`]
pub struct VirtualClock {
    current_time: AtomicU64,
    notify: Notify,
}

impl Debug for VirtualClock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualClock")
            .field(
                "current_time",
                &system_time_to_date_time(
                    UNIX_EPOCH + Duration::from_millis(self.current_time.load(Ordering::Relaxed)),
                ),
            )
            .finish()
    }
}

impl VirtualClock {
    /// Creates / Constructs a new [`VirtualClock`] instance
    ///
    /// # Argument(s)
    /// This method requests an ``initial_time`` as argument, with type [`SystemTime`]
    ///
    /// # Returns
    /// The newly created [`VirtualClock`] instance with the time set to the ``initial_time``
    ///
    /// # See Also
    /// - [`VirtualClock`]
    pub fn new(initial_time: SystemTime) -> Self {
        VirtualClock::from_value(
            initial_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        )
    }

    /// Creates / Constructs a new [`VirtualClock`] instance
    ///
    /// # Argument(s)
    /// This method requires one argument, this being a ``initial_value`` with type ``u64``,
    /// this value is represented in **total milliseconds**
    ///
    /// # Returns
    /// The newly created [`VirtualClock`] instance with the time set to the ``initial_value``
    ///
    /// # See Also
    /// - [`VirtualClock`]
    pub fn from_value(initial_value: u64) -> Self {
        VirtualClock {
            current_time: AtomicU64::new(initial_value),
            notify: Notify::new(),
        }
    }

    /// Creates / Constructs a new [`VirtualClock`] instance from the current time
    ///
    /// # Returns
    /// The newly created [`VirtualClock`] instance with the time set to the current time
    ///
    /// # See Also
    /// - [`VirtualClock`]
    pub fn from_current_time() -> Self {
        Self::new(SystemTime::now())
    }

    /// Creates / Constructs a new [`VirtualClock`] instance from the UNIX Epoch
    ///
    /// # Returns
    /// The newly created [`VirtualClock`] instance with the time set to the UNIX Epoch
    ///
    /// # See Also
    /// - [`VirtualClock`]
    pub fn from_epoch() -> Self {
        Self::new(SystemTime::UNIX_EPOCH)
    }
}

#[async_trait]
impl AdvanceableScheduleClock for VirtualClock {
    async fn advance_to(&self, to: SystemTime) {
        let to_millis = to
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.current_time.store(to_millis, Ordering::Relaxed);
        self.notify.notify_waiters();
    }
}

#[async_trait]
impl SchedulerClock for VirtualClock {
    async fn now(&self) -> SystemTime {
        let now = self.current_time.load(Ordering::Relaxed);
        UNIX_EPOCH + Duration::from_millis(now)
    }

    async fn idle_to(&self, to: SystemTime) {
        while self.now().await < to {
            self.notify.notified().await;
        }
    }
}
