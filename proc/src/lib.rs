mod every;
mod utils;
mod task;
mod taskframe;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// The [`every`] attribute macro is an alternative ergonomic way to write interval-based schedule as
/// opposed to manually constructing the [`TaskScheduleInterval`](chronographer::prelude::TaskScheduleInterval)
/// object from the ground up.
///
/// # Expansion Semantics
/// It utilizes under the hood [`TaskScheduleInterval`](chronographer::prelude::TaskScheduleInterval)
/// and calculates the appropriate time from the time-literal expression at compile-time.
///
/// The translated / expanded version typically looks this:
/// ```ignore
/// TaskScheduleInterval::from_secs_f64(...).unwrap()
/// ```
///
/// # Invocation Syntax
/// This macro uses its own syntax in order to form an interval via multiple **Time Literals**. The
/// format of a time literal is a positive number followed by a time prefix.
///
/// The defined time prefixes of this macro are as follows:
/// ```rust
/// use chronographer::every;
///
/// every!(500ms) // 500 Milliseconds via "ms"
/// every!(1s) // 1 Second via "s"
/// every!(2m) // 2 Minutes via "m"
/// every!(3h) // 3 Hours via "h"
/// every!(4d) // 4 Days via "d"
/// ```
///
/// The [`every`] macro allows to define more specific times via multiple time literals sorted from most
/// significant / longest to least significant / shortest, the significance order of each time prefix
/// is listed below:
/// - Days = ``d``
/// - Hours = ``h``
/// - Minutes = ``m``
/// - Seconds = ``s``
/// - Milliseconds = ``ms``
///
/// You can skip a time literal such as only including the day and second time fields. Each time literal
/// must be separated with either a space or comma, some examples include:
/// ```rust
/// use chronographer::every;
///
/// every!(1s, 500ms) // 1 Second & 500 Milliseconds
/// every!(3m, 30s) // 3 Minutes & 30 Seconds
/// every!(4h, 20s) // 4 Hours & 20 Seconds
/// every!(6h, 20m, 45s) // 4 Hours, 20 Minutes & 45 Seconds
/// every!(1d, 20ms) // 1 Day & 20 Milliseconds
/// every!(1d, 1h, 1m, 1s, 1ms) // 1 Day, 1 Hour, 1 Minute, 1 Second & 1 Millisecond
/// ```
///
/// Finally, the [`every`] macro additionally supports the use of decimals. Though, you can only use it in
/// the last time field literal, milliseconds do not support this property.
///
/// A couple of examples of decimals are demonstrated below:
/// ```rust
/// use chronographer::every;
///
/// every!(1.34s) // Stand-alone Decimal For 1.34 Seconds
/// every!(3.4d) // 3.4 Days = 3 Days, 9 Hours & 36 Minutes
/// every!(3d, 9.6h) // Same as above but with multiple time literals
/// every!(3m, 1.5s) // 3 Minutes & 1.5 Seconds
/// ```
///
/// # Limitations
/// Any lower-order time units (below milliseconds, such as nanoseconds, picoseconds... etc.), CANNOT be represented with
/// the [`every`] macro, though usually it isn't particularly needed.
///
/// The same thing applies with higher-order time units (above days, such as weeks, months, years, decades... etc.) do
/// NOT include a time literal. Though a workaround of this issue is utilizing higher values for days such as:
/// ```rust
/// use chronographer::every;
///
/// every!(7d) // For 1 Week
/// every!(30d) // For ~1 Month (without counting the edge cases)
/// every!(365d) // For ~1 Year (again without edge cases)
/// ```
///
/// If these needs are more common, it is suggested to take a look at [`cron`], [`calendar`] or their Base
/// API equivalents [`TaskScheduleCron`](chronographer::prelude::TaskScheduleCron) and
/// [`TaskScheduleCalendar`](chronographer::prelude::TaskScheduleCalendar) respectively.
///
/// # See Also
/// - [`TaskScheduleInterval`](chronographer::prelude::TaskScheduleInterval) The base API equivalent
/// - [`cron`] For making more complex schedules with a CRON expression
/// - [`calendar`] For making more complex schedules with a human-readable calendar expression
/// - [`TaskSchedule`](chronographer::prelude::TaskSchedule) The trait that makes schedules possible
#[proc_macro]
pub fn every(input: TokenStream) -> TokenStream {
    every::every(input)
}

#[proc_macro_attribute]
pub fn task(attr: TokenStream, item: TokenStream) -> TokenStream {
    task::task(attr, item)
}

#[proc_macro_attribute]
pub fn taskframe(attrs: TokenStream, item: TokenStream) -> TokenStream {
    taskframe::taskframe(attrs, item)
}

#[proc_macro]
pub fn cron(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn greet() -> String {
                my_library::hello(stringify!(#name))
            }
        }
    };

    TokenStream::from(expanded)
}