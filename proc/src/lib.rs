mod every;
mod utils;
mod task;
mod taskframe;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// The [`every`] proc-macro is an alternative ergonomic way to write interval-based schedule as
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

/// The [`task`] attribute macro is an alternative more ergonomic way to write Tasks as opposed to
/// manually constructing them via the Base API and Rust internals from the ground up.
///
/// The bare minimal interface is essentially a typical Rust function with a schedule:
/// ```rust
/// use chronographer::prelude::*;
///
/// #[task(schedule = every!(1s))]
/// pub async fn MyCoolThing(ctx: TaskFrameContext) -> Result<(), String> {
///     todo!()
/// }
/// ```
/// > **NOTE:** The camelCase is done on purpose, since the macro translates it into a struct
///
/// Tasks can either be singleton (one instance can be fetched globally from anywhere) or
/// non-singleton / multi-instanced (you can create new instances of tasks).
///
/// [`task`] macro includes a "clever" auto-append feature in which it adds the Task or TaskFrame prefix
/// if not included depending on if it is a Task or TaskFrame that is currently being expanded to.
///
/// Though it allows the for an escape hatch, to override names with two attribute parameters
/// (see below for more info about).
///
/// Everything else is almost identical to the [`taskframe`] attribute macro as such it's recommended
/// to read more about it
///
/// # Valid Targets
/// The [`task`] macro is applied primarily async functions, these functions cannot be methods (include
/// &self or &mut self as first argument in a struct / enum / trait).
///
/// # Attributes & Parameters
/// The [`task`] macro contains 4 attribute parameters, one of which is required while the other three
/// are optional and one out of these is an attribute flag:
/// - **schedule** Specifies the schedule to use, this can be anything (from a type initialization to a macro)
/// that translates or is something that implements the [`TaskSchedule`](chronographer::prelude::TaskSchedule) trait
///
/// - **non_singleton** Fully optional, specifies the Task to be non-singleton, with this set, developers
/// can use the ``new()`` method to create new Task instances as opposed to ``instance()`` for getting the
/// global singleton instance
///
/// - **task_name_override** Fully optional, overrides the name of the Task (not its TaskFrame), disables
/// the clever auto-append feature
///
/// - **taskframe_name_override** Fully optional, overrides the name of the TaskFrame (not its Task),
/// disables the clever auto-append feature
///
/// # Expansion Semantics
/// The [`task`] macro has two ways of expanding, all depending on whenever the Task is either singleton
/// or non-singleton / multi-instanced. In both cases it uses the [`taskframe`] macro under the hood.
///
/// For the former where the Task is a singleton it is similar to:
/// ```ignore
/// /* Input:
/// #[task(schedule = [SCHEDULE])]
/// pub async fn MyTask(_ctx: &TaskFrameContext) -> Result<(), [ERROR]> {
///     // ...
/// }
/// */
///
/// pub struct MyTask;
/// impl MyTaskFrame {
///     pub fn instance() -> &'static Task<MyTaskFrame> {
///         static INSTANCE: OnceLock<Task<MyTaskFrame>> = OnceLock::new();
///         INSTANCE.get_or_init(|| Task::new(MyTaskFrame::default(), [SCHEDULE]))
///     }
/// }
/// #[taskframe(name_override = MyTaskFrame)]
/// pub async fn MyTask(_ctx: &TaskFrameContext) -> Result<(), [ERROR]> {
///     // ...
/// }
/// ```
///
/// Both ``[SCHEDULE]`` and ``[ERROR]`` are placeholders for what kind of scheduler to use and the
/// error type to use respectively. The latter on the other hand typically takes the form of:
/// ```ignore
/// /* Input:
/// #[task(schedule = [SCHEDULE])]
/// pub async fn MyTask(_ctx: &TaskFrameContext) -> Result<(), [ERROR]> {
///     // ...
/// }
/// */
///
/// pub struct MyTask;
/// impl MyTask {
///     pub fn new() -> Task<MyTaskFrame> {
///         Task::new(MyTaskFrame::default(), [SCHEDULE])
///     }
/// }
///
/// #[taskframe(name_override = MyTaskFrame)]
/// pub async fn MyTaskFrame(_ctx: &TaskFrameContext) -> Result<(), [ERROR]> {
///     // ...
/// }
/// ```
///
/// Again, when it comes to the function itself, it is highly recommended to check how [`taskframe`]
/// works as it borrows the same syntax with a minor caveat (see the limitations below).
///
/// # External Interactions
/// The [`task`] macro preserves every other attribute macro and mounts it onto the generated results,
/// while also having its own interactions with other attribute macros.
///
/// When specifying ``#[workflow(...)]`` modifies the TaskFrame initialization of the Task with
/// the specified workflow (including the function of the Task).
///
/// Whereas specifying ``#[hooks(...)]`` automatically attaches the specified TaskHooks that subscribed
/// to specific events upon initialization of the Task.
///
/// # Limitations
/// While [`taskframe`] generics work mostly out of the box, there is a caveat for [`task`].
/// Due to static-based limitations, there can be no singleton Task with generics. As such either remove
/// the use of generics or make it non-singleton.
///
/// In addition to this, just like [`taskframe`] generics, lifetimes (due to async limitations) and
/// ABI functions are unsupported. It should also be mentioned you can't use [`task`] macro in methods of
/// structs / enums / traits, just pure functions.
///
/// # See Also
/// - [`taskframe`] - The macro closely related to [`task`] for producing TaskFrames
/// - [`workflow`] - The macro used for defining workflows, and has close relations with [`task`] and [`taskframe`]
/// - [`hooks`] - The macro used for attaching TaskHooks to events, and has close relations with [`task`]
/// - [`TaskFrame`](chronographer::prelude::TaskFrame) - The trait that makes TaskFrames possible
/// - [`TaskSchedule`](chronographer::prelude::TaskSchedule) - The trait that makes schedules possible
#[proc_macro_attribute]
pub fn task(attr: TokenStream, item: TokenStream) -> TokenStream {
    task::task(attr, item)
}


/// The [`taskframe`] attribute macro is an alternative more ergonomic way to write TaskFrames as opposed to
/// manually constructing them via the Base API and Rust internals from the ground up.
///
/// The bare minimal interface is essentially a typical Rust function:
/// ```rust
/// use chronographer::prelude::*;
///
/// #[taskframe]
/// pub async fn MyCoolThing(ctx: TaskFrameContext) -> Result<(), String> {
///     todo!()
/// }
/// ```
/// > **NOTE:** The camelCase is done on purpose, since the macro translates it into a struct
///
/// [`taskframe`] macro includes a "clever" auto-append feature in which it adds the TaskFrame prefix
/// if not included. Though it allows the for an escape hatch, to override the name with one attribute
/// parameter (see below for more info about).
///
/// Whe it comes to creating full Tasks objects, it is recommended to check the [`task`] attribute macro,
/// its interface is almost identical and in fact the [`taskframe`] macro is used under the hood.
///
/// # Valid Targets
/// The [`taskframe`] macro is applied primarily async functions, these functions cannot be methods (include
/// &self or &mut self as first argument in a struct / enum / trait).
///
/// # Attributes & Parameters
/// The [`taskframe`] contains only one attribute parameter that being ``name_override`` which allows
/// users to modify the name of the final TaskFrame generated (disables the clever auto-append feature).
///
/// # Expansion Semantics
/// The [`taskframe`] syntax is almost if not identical to a pure Rust function, when the macro expands
/// it typically takes the form of:
/// ```ignore
/// /* Input:
/// #[taskframe]
/// pub async fn MyTaskFrame(_ctx: &TaskFrameContext) -> Result<(), [ERROR]> {
///     // ...
/// }
/// */
///
/// #[derive(Default, Clone, Copy)]
/// pub struct MyTaskFrame;
/// impl TaskFrame for MyTaskFrame {
///     type Args = ();
///     type Error = [ERROR];
///
///     async fn execute(
///         &self,
///         _ctx: &TaskFrameContext,
///         args: &<MyTaskFrame as TaskFrame>::Args
///     ) -> Result<(), Self::Error> {
///         // ...
///     }
/// }
/// ```
/// The only restrictions the [`taskframe`] macro imposes is it has to be an async function, has
/// to contain as first argument a reference to the [`TaskFrameContext`] and its return type must be
/// a ``Result<(), E>`` with E being your error type.
///
/// One of the things the [`taskframe`] macro supports is the use of the ``unsafe`` keyword in TaskFrames
/// which is automatically embedded to the implementation.
///
/// Moreover, the more powerful feature is the ability to specify more arguments. In the base API the
/// [`TaskFrame`](chronographer::prelude::TaskFrame) requires the arguments to be a tuple in the form of
/// ``(T1, T2, T3 ... Tn)``.
///
/// Then the user must extract those values and name them themselves which is slightly cumbersome and
/// non-ergonomic, as changing the argument structure requires 2 places to change (the Args associated
/// type and the extraction logic).
///
/// The [`taskframe`] macro addresses this, letting you write:
/// ```rust
/// #[taskframe]
/// pub async fn MyTaskFrame(_ctx: &TaskFrameContext, arg1: u8, arg2: Vec<u8>, arg3: Option<String>) -> Result<(), [ERROR]> {
///     println!("{arg1:?} {arg2:?} {arg3:?}"); // Using the arguments in our code
///     // ...
/// }
/// ```
///
/// Under the hood it packs it to the tuple and extracts in the code the types with their matching name
/// (first ``u8`` for arg1, second ``Vec<u8>`` for arg2... etc.), no restrictions imposed on the number
/// of arguments.
///
/// Finally, the use of generics is possible in functions including type parameters and constant
/// parameters (with one certain limitation in the Limitations section below) as seen:
/// ```rust
/// use std::fmt::{Debug, Display};
///
/// #[taskframe]
/// pub async fn MyTaskFrame<T, E, const N: usize>(
///     _ctx: &TaskFrameContext,
///     arg1: T,
///     arg2: Vec<T>,
///     arg3: Option<T>
/// ) -> Result<(), E>
///where
///    T: Send + Sync + 'static,
///    E: Debug + Display + Send + Sync + 'static
/// {
///     // ...
/// }
/// ```
/// We added 3 generics to our example, two of which are type parameters where the former is ``T`` and
/// used in our arguments and the latter ``E`` is used for our error.
///
/// Type parameters must implement ``Send``, ``Sync`` and have a lifetime of ``'static`` due to rust
/// async limitations. *Should be noted generics aren't limited to either arguments or the return type,
/// they work the same as Rust generics*.
///
/// Additionally, we have one constant parameter ``N`` that is type of usize, this isn't used anywhere
/// obvious but could theoritically be used in our code. Moreover, we also have a ``where`` clause which is supported,
/// alternatively we can specify the trait bounds directly if we want to.
///
/// # External Interactions
/// The [`taskframe`] macro preserves every other attribute macro and mounts it onto the generated results,
/// while also having its own interactions with one other attribute macro.
///
/// When specifying ``#[workflow(...)]`` modifies the TaskFrame to include an additional method to
/// create the workflow specified via the ``new_workflow`` constructor.
///
/// # Limitations
/// When it comes to generics, lifetimes (due to async limitations) and ABI functions are unsupported.
/// It should also be mentioned you can't use [`taskframe`] macro in methods of structs / enums / traits, just
/// pure functions.
///
/// # See Also
/// - [`task`] - An upgrade of the [`taskframe`] macro for specifying full Task objects
/// - [`workflow`] - The macro used for defining workflows, and has close relations with [`taskframe`]
/// - [`TaskFrame`](chronographer::prelude::TaskFrame) - The trait that makes TaskFrames possible
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