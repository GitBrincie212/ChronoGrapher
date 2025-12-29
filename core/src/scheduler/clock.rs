pub mod progressive_clock; // skipcq: RS-D1001

pub mod virtual_clock; // skipcq: RS-D1001

pub use progressive_clock::ProgressiveClock;
use std::ops::Deref;
use std::time::{Duration, SystemTime};
pub use virtual_clock::VirtualClock;

use crate::utils::Timestamp;
use async_trait::async_trait;

/// [`SchedulerClock`] is a trait for implementing a custom scheduler clock, typical operations
/// include getting the current time, idle for a specific duration (or til a specific date is reached).
///
/// # Usage Note(s)
/// The precision of [`SchedulerClock`] can depend on the underlying OS-specific time format due
/// to the fact it uses `SystemTime` under the hood. For example, on Windows, the time is represented
/// in 100 nanosecond intervals, whereas Linux can represent nanosecond intervals... etc
///
/// # Required Methods
/// When implementing the [`SchedulerClock`], one must provide implementations for two methods, those
/// being [`SchedulerClock::now`] and [`SchedulerClock::idle_to`], the former is used to get the
/// current time while the latter is used to idle to a specific time of interest, both methods are
/// used by the [`Scheduler`] under the hood
///
/// # Trait Implementation(s)
///  Specifically, there are 2 noteworthy implementations to list, those being:
///
/// - [`VirtualClock`] used to simulate time (for unit-tests, debugging,
/// [`flashcrowd`](https://en.wiktionary.org/wiki/flashcrowd#English) simulations... etc.), it doesn't
/// go forward without explicit advancing and implements as well as the [`AdvanceableScheduleClock`]
/// trait
///
/// - [`ProgressiveClock`] the default go-to clock, it automatically goes forward and doesn't wait around,
/// it doesn't implement the trait [`AdvanceableScheduleClock`] trait due to its nature
///
/// # Extension Trait(s)
/// there is [`AdvanceableScheduleClock`] which allows the explicit advancing of time via methods
/// it provides. Specifically, the [`VirtualClock`] implements this to allow for the explicit advancing
/// from now to points of interest
///
/// # See Also
/// - [`VirtualClock`]
/// - [`ProgressiveClock`]
/// - [`AdvanceableScheduleClock`]
#[async_trait]
pub trait SchedulerClock<D: Timestamp>: 'static + Send + Sync {
    async fn init(&self) {}

    /// Gets the current time of the clock
    ///
    /// # Returns
    /// The current time of the clock represented as [`SystemTime`] (to avoid any timezone issues
    /// and let the user convert it to their timezone of their choice representation)
    ///
    /// # See Also
    /// - [`SystemTime`]
    /// - [`SchedulerClock`]
    async fn now(&self) -> D;

    /// Idle until this specified time is reached (if it is in the past or present, it doesn't idle)
    ///
    /// # Arguments
    /// It accepts a ``to`` parameter, it specifies the point in time to
    /// reach by simply idling around. The ``to`` parameter is type of [`SystemTime`] (to
    /// avoid any timezone issues and let the user convert it to their timezones of their
    /// choice representation)
    ///
    /// # See Also
    /// - [`SystemTime`]
    /// - [`SchedulerClock`]
    async fn idle_to(&self, to: D);
}

#[async_trait]
impl<T, D> SchedulerClock<D> for T
where
    T: Deref + Send + Sync + 'static,
    T::Target: SchedulerClock<D>,
    D: Timestamp,
{
    async fn now(&self) -> D {
        self.deref().now().await
    }

    async fn idle_to(&self, to: D) {
        self.deref().idle_to(to).await
    }
}

/// [`AdvanceableScheduleClock`] is an optional extension to [`SchedulerClock`] which, as the name
/// suggests, allows for arbitrary advancement of time, specific clocks might not support arbitrary
/// advancement (such as [`ProgressiveClock`]), as such why it is an optional trait
///
/// # Required Methods
/// When implementing the [`AdvanceableScheduleClock`], one has to fully implement one method
/// being [`AdvanceableScheduleClock::advance_to`] which is used for advancing the time to
/// a specific point of interest
///
/// # Trait Implementation(s)
/// Specifically, only one type implements the [`AdvanceableScheduleClock`] trait, that is
/// the [`VirtualClock`] which allows for the explicit advancement of arbitrary points in time
///
/// # Supertrait(s)
/// as discussed above, [`AdvanceableScheduleClock`] is an extension to [`SchedulerClock`], as such
/// when implementing this trait, one has to also implement the [`SchedulerClock`] trait
///
/// # See Also
/// - [`SchedulerClock`]
/// - [`VirtualClock`]
#[async_trait]
pub trait AdvanceableScheduleClock<D: Timestamp>: SchedulerClock<D> {
    /// Advance the time by a specified duration forward
    ///
    /// # Arguments
    /// It accepts a ``duration`` parameter of type [`Duration`], used to advance the
    /// time by that specific duration, it acts similar in spirit to [`AdvanceableScheduleClock::advance_to`]
    /// (in fact it uses this method under the hood), but for duration
    ///
    /// # See Also
    /// - [`Duration`]
    /// - [`SchedulerClock`]
    /// - [`AdvanceableScheduleClock`]
    async fn advance(&self, duration: Duration);

    /// Advance the time to a specified desired future point of time
    ///
    /// # Arguments
    /// It accepts a ``to`` parameter of type [`SystemTime`] (to avoid any timezone issues and
    /// let the user convert it to their own timezones of choice representation). It is used to advance the
    /// time to that point of time. It acts similarly to [`AdvanceableScheduleClock::advance`] but
    /// for time points, this method is required to specify an implementation
    ///
    /// # See Also
    /// - [`Duration`]
    /// - [`SchedulerClock`]
    /// - [`AdvanceableScheduleClock`]
    async fn advance_to(&self, to: SystemTime);
}

#[async_trait]
impl<T, D> AdvanceableScheduleClock<D> for T
where
    T: Deref + Send + Sync + 'static,
    T::Target: AdvanceableScheduleClock<D>,
    D: Timestamp,
{
    async fn advance(&self, duration: Duration) {
        self.deref().advance(duration).await
    }

    async fn advance_to(&self, to: SystemTime) {
        self.deref().advance_to(to).await
    }
}
