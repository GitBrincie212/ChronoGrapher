mod entry;
mod every;
mod cron;
mod utils;
mod task;
mod taskframe;
mod workflow;

use proc_macro::TokenStream;

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

/// The [`task`] attribute macro is an alternative more ergonomic way to write [`Task`](chronographer::prelude::Task)
/// as opposed to manually constructing them via the Base API and Rust internals from the ground up.
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
/// [`Task(s)`](chronographer::prelude::Task) generated with the macro can either be singleton
/// (one instance can be fetched globally from anywhere) or non-singleton / multi-instanced
/// (you can create new instances of tasks).
///
/// [`task`] macro includes a "clever" auto-append feature in which it adds the "Task" or "TaskFrame" prefix
/// if not included depending on if it is a "Task" or "TaskFrame" that is currently being expanded to.
///
/// Though it allows the for an escape hatch to override names with two respective attribute parameters
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
/// - **non_singleton** Fully optional, specifies the [`Task`](chronographer::prelude::Task) to be non-singleton,
/// with this set, developers can use the ``new()`` method to create new [`Task`](chronographer::prelude::Task)
/// instances as opposed to ``instance()`` for getting the global singleton instance
///
/// - **task_name_override** Fully optional, overrides the name of the generated [`Task`](chronographer::prelude::Task)
/// (not its [`TaskFrame`](chronographer::prelude::TaskFrame)), disables the clever auto-append feature
///
/// - **taskframe_name_override** Fully optional, overrides the name of the [`TaskFrame`](chronographer::prelude::TaskFrame)
/// (not its [`Task`](chronographer::prelude::Task)), disables the clever auto-append feature
///
/// # Expansion Semantics
/// The [`task`] macro has two ways of expanding, all depending on whenever the [`Task`](chronographer::prelude::Task) is either singleton
/// or non-singleton / multi-instanced. In both cases it uses the [`taskframe`] macro under the hood.
///
/// For the former where the [`Task`](chronographer::prelude::Task) is a singleton it is similar to:
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
/// Both ``[SCHEDULE]`` and ``[ERROR]`` are placeholders for what kind of schedule to use and the
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
/// When specifying [`workflow`] modifies the [`TaskFrame`](chronographer::prelude::TaskFrame)
/// initialization of the [`Task`](chronographer::prelude::Task) with the specified workflow
/// (including the function of the generated [`Task`](chronographer::prelude::Task)).
///
/// Whereas specifying [`hook`] automatically attaches the specified [`TaskHooks`](chronographer::prelude::TaskHook)
/// that subscribed to specific events upon initialization of the [`Task`](chronographer::prelude::Task).
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
/// - [`Task`](chronographer::prelude::Task) - The base API "equivalent" used internally
/// - [`taskframe`] - The macro closely related to [`task`] for producing TaskFrames
/// - [`workflow`] - The macro used for defining workflows, has close relations with [`task`] and [`taskframe`]
/// - [`hooks`] - The macro used for attaching TaskHooks to events, and has close relations with [`task`]
/// - [`TaskFrame`](chronographer::prelude::TaskFrame) - The trait that makes TaskFrames possible
/// - [`TaskSchedule`](chronographer::prelude::TaskSchedule) - The trait that makes schedules possible
/// - [`TaskHook`](chronographer::prelude::Task) - The system used for the [`hooks`] macro
#[proc_macro_attribute]
pub fn task(attr: TokenStream, item: TokenStream) -> TokenStream {
    task::task(attr, item)
}

/// The [`taskframe`] attribute macro is an alternative more ergonomic way to write
/// [`TaskFrames`](chronographer::prelude::TaskFrame) as opposed to manually constructing them via the
/// Base API and Rust internals from the ground up.
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
/// Whe it comes to creating full Tasks objects, it is recommended to check the [`task`] attribute macro,
/// its interface is almost identical and in fact the [`taskframe`] macro is used under the hood.
///
/// # Valid Targets
/// The [`taskframe`] macro is applied primarily async functions, these functions cannot be methods (include
/// &self or &mut self as first argument in a struct / enum / trait).
///
/// # Attributes & Parameters
/// The [`taskframe`] contains no attribute parameters (apart from an internal one which under any
/// circumstances should **NOT** be used due to being an antipattern).
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
/// to contain as first argument a reference to the [`TaskFrameContext`](chronographer::prelude::TaskFrameContext)
/// and its return type must be a ``Result<(), E>`` with E being your error type.
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
/// obvious but could theoritically be used in our code. Moreover, we also have a ``where`` clause which is
/// supported, alternatively we can specify the trait bounds directly if we want to.
///
/// # External Interactions
/// The [`taskframe`] macro preserves every other attribute macro and mounts it onto the generated results,
/// while also having its own interactions with one other attribute macro.
///
/// When specifying [`workflow`] modifies the TaskFrame to include an additional method to
/// create the workflow specified via the ``workflow`` constructor.
///
/// # Limitations
/// When it comes to generics, lifetimes (due to async limitations) and ABI functions are unsupported.
/// It should also be mentioned you can't use [`taskframe`] macro in methods of structs / enums / traits, just
/// pure functions.
///
/// # See Also
/// - [`TaskFrame`](chronographer::prelude::TaskFrame) - The base API equivelent
/// - [`task`] - An upgrade of the [`taskframe`] macro for specifying full Task objects
/// - [`workflow`] - The macro used for defining workflows, has close relations with [`taskframe`] and [`task`]
#[proc_macro_attribute]
pub fn taskframe(attrs: TokenStream, item: TokenStream) -> TokenStream {
    taskframe::taskframe(attrs, item)
}

/// The [`workflow`] attribute macro is a special macro from the rest macro, like with [`hooks`] it
/// behaves as an annotation working alongside [`task`] and [`taskframe`] rather than a macro which
/// transforms directly the input provided into something new.
///
/// With that said, it allows users to write ergonomically workflows ([`TaskFrames`](chronographer::prelude::TaskFrame)
/// stacking on top of each other) which works on top of the function / code they have already written.
///
/// Users can write any kind of workflow via the provided built-in workflow primitives,
/// from simplest (one workflow primitive) to most complex (with basically an infinite number of these).
///
/// Just like the Base API, ordering matters significantly as the workflow will behave drastically
/// differently under various ordering configurations. Everything is applied from top to bottom.
///
/// The bare minimal interface is essentially mounting a workflow below either [`task`] or [`taskframe`]:
/// ```rust
/// use chronographer::prelude::*;
///
/// #[taskframe]
/// #[workflow(
///     threshold(10), // Allow this workflow to run up to 10 times, then skip it when tempted
///     timeout(20s), // Timeout the entire workflow if it lasts >20 seconds
///     retry(5) // If the workflow fails, retry it immediately up to 5 times
/// )]
/// pub async fn MyCoolTaskFrame(ctx: TaskFrameContext) -> Result<(), String> {
///     todo!()
/// }
/// ```
///
/// # Valid Targets
/// The [`workflow`] macro is applied primarily async functions which include either the [`taskframe`]
/// or [`task`] attribute macro at the top. See the aforementioned attribute macros for more information
/// about their restrictions.
///
/// # Attributes & Parameters
/// Unlike most macros which have a predictable set of named parameters. The [`workflow`] macro more
/// functions as a DSL (Domain-Specific Language) and contains a list of workflow primitives.
///
/// The basic grammar of a workflow primitive is in the form ``WORKFLOW(...)`` where "WORKFLOW" is the
/// name of the workflow primitive we want to use and ``...`` are its arguments.
///
/// Unlike typical Rust, workflow primitive arguments can be positional (no argument name, just value) and
/// named (with the argument's name) or in special occasions contain a variable number of arguments.
///
/// In essence, workflow primitive arguments behave similarly to Python's ``args`` / ``kwargs``, which
/// means positional arguments should **NOT** be followed after named arguments.
///
/// As previously said, there can be any number workflow primitives. However, the [`workflow`] macro
/// imposes a minimum threshold of one workflow primitive (though no upper limit).
///
/// ## Quick Reference
/// - [Retry Workflow Primitive](#retry%20workflow%20primitive) The syntax of the ``retry`` workflow primitive.
/// - [Fallback Workflow Primitive](#fallback%20workflow%20primitive) The syntax of the ``fallback`` workflow primitive.
/// - [Delay Workflow Primitive](#delay%20workflow%20primitive) The syntax of the ``delay`` workflow primitive.
/// - [Timeout Workflow Primitive](#timeout%20workflow%20primitive) The syntax of the ``timeout`` workflow primitive.
/// - [Threshold Workflow Primitive](#threshold%20workflow%20primitive) The syntax of the ``threshold`` workflow primitive.
/// - [Dependency Workflow Primitive](#dependency%20workflow%20primitive) The syntax of the ``dependency`` workflow primitive.
/// - [Condition Workflow Primitive](#condition%20workflow%20primitive) The syntax of the ``condition`` workflow primitive.
///
/// ## Retry Workflow Primitive
/// ```ignore
/// retry(max = INT | MACRO | IDENT, delay? = RETRY_DELAY, when? = RETRY_FILTER)
/// ```
///
/// The retry workflow primitive behaves identically to [`RetriableTaskFrame`](chronographer::prelude::RetriableTaskFrame),
/// it allows to retry the workflow up to a specified number of times with a configurable delay and error filter.
///
/// ### Arguments
/// - ``max`` The upper bound of times to retry a workflow until it succeeds. Unlike other arguments,
/// this one is required to be specified, additionally it can be any source of an integer as long as it
/// can be converted internally to a ``NonZeroU32``.
///
/// - ``delay`` The delay in-between every retry, this can be as simple as ``immediate``, providing a
/// constant time / duration literal or even a backoff strategy. Its fully optional to specify and
/// by default immediately retries (zero-delay). The syntaxes of the backoff strategies are as follows:
///     1. ``immediate`` The default backoff strategy, it retries immediately
///     2. ``constant(value = DURATION)`` Same as using a plain duration / time literal.
///     3. ``linear(factor = DURATION_EXPR, start? = DURATION_EXPR, clamp? = DURATION_EXPR)`` Adjusts the delay between retries
///     based on a linear functon from the growth factor, the start and an upper bound (clamp).
///     4. ``exponential (factor = FLOAT_EXPR, start? = DURATION_EXPR, clamp? = DURATION_EXPR)`` Adjusts the delay between
///     retries based on an exponential function with the factor as base, the start and an upper bound (clamp).
///     5. ``jitter(jitter_type = JITTER_TYPE, backoff = RETRY_DELAY)`` Adjusts the delay between retries
///     based on a jittered result from the supplied backoff's results. This jitter type can be either
///     ``full``, ``equal`` or ``decorrelated(VALUE)`` which specify how the jitter should behave. It is
///     recommended to read more the article [When APIs Fail: A Developer's Journey with Retries, Back Off, and Jitter](https://dev.to/kengowada/when-apis-fail-a-developers-journey-with-retries-back-off-and-jitter-1g2f)
///
/// - ``when`` The error filter composed of a list of patterns encapsulated in brackets (``[...]``)
/// with optionally an exclamation mark (``!``) as prefix. When used without any exclamation marks it's
/// a whitelist (one of the pattern must match) whereas with one it turns into a blacklist (none of the
/// patterns must match). Patterns match based on the error's structure, its fully optional to specify
/// and by default any error is let through.
///
/// ### Examples:
/// ```rust
/// use chronographer::prelude::*;
///
/// #[taskframe]
/// #[workflow(
///     retry(5), // Retry up to 5 times immediately
///     retry(2, delay = 3s), // Retry up to 2 times with a delay of 3 seconds
///     retry(7, linear(2s, 300ms)), // ... with a delay starting from 2 seconds and growing linearly
///     retry(3, delay = exponential(2.0)), // ... with a delay exponentially growing by 2^n
///     retry(11, delay = jitter(equal, 2s)), // ... with an equally-jittered delay of 2 seconds
///     retry(8, when = ["A" | "B"]), // ... with an error filter matching either values "A" or "B"
///     retry(1, when = !["C" | "D"]), // ... with an error filter NOT matching either values "C" or "D"
///     retry(4, 5s, ["A" | "B" | "C"]), // Retry up to 4 times with a delay of 5 seconds IF matching the errors
/// )]
/// pub async fn MyCoolTaskFrame(ctx: TaskFrameContext) -> Result<(), String> {
///     todo!()
/// }
/// ```
///
/// ## Fallback Workflow Primitive
/// ```ignore
/// fallback(TASK_FRAMES...)
/// ```
///
/// The fallback workflow primitive behaves almost identically to [`FallbackTaskFrame`](chronographer::prelude::FallbackTaskFrame),
/// it allows the specification of one or multiple fallbacks when things go south (an error occurred).
///
/// Unlike the Base API's [`FallbackTaskFrame`](chronographer::prelude::FallbackTaskFrame), which requires
/// manually stacking one fallback on top of the other to include multiple fallbacks. This workflow
/// primitive does the nesting automatically and isn't restricted to the number of TaskFrames used as fallbacks.
///
/// The ordering goes from left (acting as the first) to right (acting as last). Specifying multiple
/// fallback primitives in rapid succession is deemed an antipattern even if possible.
///
/// ### Arguments
/// This workflow primitive can have an infinite number of arguments, with the only restrictions being
/// they are positional and must be a [`TaskFrames`](chronographer::prelude::TaskFrame) "expression".
///
/// These [`TaskFrames`](chronographer::prelude::TaskFrame) "expressions" are either identifiers
/// or they can be identifiers prefixed with ``@``, both produce polar opposite results.
///
/// The former tells ChronoGrapher to include the entire target [`TaskFrame's`](chronographer::prelude::TaskFrame)
/// workflow (alongside it of course), this concept is called **Workflow Inheritance**.
///
/// While the latter only includes the target [`TaskFrame's`](chronographer::prelude::TaskFrame) raw code,
/// disregarding completely the workflow logic, present or not.
///
/// ### Examples:
/// ```rust
/// use chronographer::prelude::*;
///
/// #[taskframe]
/// pub async fn MyFallbackTaskFrame1(ctx: &TaskFrameContext, _error: String) -> Result<(), String> {
///     todo!()
/// }
///
/// #[taskframe]
/// pub async fn MyFallbackTaskFrame2(ctx: &TaskFrameContext, _error: String) -> Result<(), String> {
///     todo!()
/// }
///
/// #[taskframe]
/// pub async fn MyFallbackTaskFrame3(ctx: &TaskFrameContext, _error: String) -> Result<(), String> {
///     todo!()
/// }
///
/// #[taskframe]
/// #[workflow(
///     fallback(MyFallbackTaskFrame1), // If it fails, then run MyFallbackTaskFrame1 as backup
///     fallback(
///         MyFallbackTaskFrame1,
///         MyFallbackTaskFrame2
///     ), // ... run MyFallbackTaskFrame1 -> MyFallbackTaskFrame2 as backup
///     fallback(
///         MyFallbackTaskFrame1,
///         MyFallbackTaskFrame2,
///         MyFallbackTaskFrame3
///     ), // ... run MyFallbackTaskFrame1 -> MyFallbackTaskFrame2 -> MyFallbackTaskFrame3 as backup
/// )]
/// pub async fn MyCoolTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     todo!()
/// }
/// ```
/// ## Delay Workflow Primitive
/// ```ignore
/// delay(delay = DURATION)
/// ```
///
/// The delay workflow primitive behaves identically to [`DelayTaskFrame`](chronographer::prelude::DelayTaskFrame),
/// it allows the specification of a constant delay before executing the workflow.
///
/// ### Arguments
/// The workflow primitive accepts only argument that being ``delay`` which is a duration based
/// expression either an identifier to a constant, a macro or a time literal. It specifies the amount of
/// time to idle before continuing.
///
/// ### Examples:
/// ```rust
/// use chronographer::prelude::*;
/// use std::time::Duration;
///
/// #[taskframe]
/// #[workflow(
///     delay(5s), // Delay the workflow for 5 seconds
///     delay(delay = 800ms), // ... for 800 milliseconds
///     delay(Duration::from_secs(2)) // ... for 2 seconds
/// )]
/// pub async fn MyCoolTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     todo!()
/// }
/// ```
///
/// ## Timeout Workflow Primitive
/// ```ignore
/// timeout(duration = DURATION)
/// ```
///
/// The timeout workflow primitive behaves identically to [`TimeoutTaskFrame`](chronographer::prelude::TimeoutTaskFrame),
/// it allows the specification of an upper time limit within the workflow. If it fails to complete
/// in that time, it errors out with a timeout error.
///
/// ### Arguments
/// The workflow primitive accepts only argument that being ``duration`` which is a duration based
/// expression either an identifier to a constant, a macro or a time literal. It specifies the maximum
/// time allowed for the workflow to run before timeout.
///
/// ### Examples:
/// ```rust
/// use chronographer::prelude::*;
/// use std::time::Duration;
///
/// #[taskframe]
/// #[workflow(
///     timeout(5s), // Timeout the workflow if it executes for more than 5 seconds
///     timeout(duration = 800ms) // ... if more than 800 milliseconds
///     timeout(Duration::from_secs(2)) // ... if more than 2 seconds
/// )]
/// pub async fn MyCoolTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     todo!()
/// }
/// ```
///
/// ## Threshold Workflow Primitive
/// ```ignore
/// threshold(max = INT | MACRO | IDENT, when_reach? = THRESHOLD_REACH, count? = THRESHOLD_COUNT)
/// ```
///
/// The threshold workflow primitive behaves identically to [`ThresholdTaskFrame`](chronographer::prelude::ThresholdTaskFrame),
/// it allows the specification of an upper time limit in the number of times a workflow is executed
/// based on a criteria. When the workflow is tempted to run again, it can skip, fail with an error or
/// do something custom.
///
/// ### Arguments
/// - ``max`` The upper bound of times to run a workflow. Unlike other arguments, this one is required to
/// be specified, additionally it can be any source of an integer as long as it can be converted internally
/// to a ``NonZeroUsize``.
///
/// - ``when_reach`` A configuration for the threshold on how to react when the upper threshold limit
/// has been reached and the workflow is tempted to run again, the value has to implements the
/// [`ThresholdLogic`](chronographer::task::frames::thresholdframe::ThresholdLogic). Its fully optional
/// to specify and by default it skips it with success. The syntax is as follows:
///     1. ``skip`` Used by default, skips the workflow fully as success
///     2. ``error`` Error out with a special-defined error from ChronoGrapher
///     3. ``custom(EXPR)`` A custom-based reach behavior with its own internal algorithm to determine
///     the corresponding behavior.
///
/// - ``count`` A configuration for the threshold on how to count each run towards the threshold
/// based on criteria, the value has to implements the
/// [`ThresholdReachBehaviour`](chronographer::task::frames::thresholdframe::ThresholdReachBehaviour).
/// Its fully optional to specify and by default it counts any kind of run towards the threshold. The
/// syntax is as follows:
///     1. ``identity``  Used by default, it counts any kind of run as valid
///     2. ``successes`` Counts any kind of <u>successful</u> run.
///     3. ``failures`` Counts any kind of <u>failed</u> runs with any shape of error.
///     4. ``custom`` A custom criteria that identifies when to count or not a specific run.
///
/// ### Examples:
/// ```rust
/// use chronographer::prelude::*;
///
/// #[taskframe]
/// #[workflow(
///     threshold(10), // Allow this workflow to run up to 10 times before skipping it when tempted
///     threshold(5, count = successes), // ... to 5 successful times before skipping it ...
///     threshold(3, count = failures), // ... to 3 failed times before skipping it ...
///     threshold(2, error), // ... up to 2 times before erroring out when tempted
///     threshold(2, error, failures), // Run up to 2 failed times before erroring out ...
/// )]
/// pub async fn MyCoolTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     todo!()
/// }
/// ```
///
/// ## Dependency Workflow Primitive
/// ```ignore
/// dependency(dep = DEPENDENCY, unresolve? = DEPENDENCY_UNRESOLVE)
/// ```
///
/// The dependency workflow primitive behaves identically to [`DependencyTaskFrame`](chronographer::prelude::DependencyTaskFrame),
/// it allows the specification of a required dependency to be resolved before ultimately running the workflow,
/// the shape of the dependency can be as simple as a flag to as complex as a boolean expression with Tasks.
///
/// ### Arguments
/// This workflow primitive only has two arguments, the former being a required argument to specify and
/// specifies the dependency to be resolved before running this workflow. This argument is special while
/// it can accept any [`FrameDependency`](chronographer::task::dependency::FrameDependency). It also allows
/// users to create their own complex dependencies easily:
///
/// Unlike the base API which due to Rust limitations supports as boolean operators ``&``, ``|`` and
/// ``!`` (not the boolean-based operators). Inside a dependency expression it supports ``&&``, ``||``
/// and ``!`` mapping to their respective base API counterpart.
///
/// Additionally, dependency expressions also support the XOR operator (``^``) which translates to the
/// corresponding base API operators under the hood.
///
/// Finally, when it comes to the "leaf" / "atomic" dependencies themselves, there are two categories,
/// by specifying an identifier you reference an outside dependency fully whereas you can create a
/// dependency by utilizing the following "atomic" expressions:
/// - ``MY_TASK(any = INT)`` Creates a task dependency where ``MY_TASK`` is the identifier of the Task,
/// specifying "any" followed by an integer value (N), creates a Task dependency that must run N times
/// before resolving.
///
/// - ``MY_TASK(success = INT)`` Creates a task dependency where ``MY_TASK`` is the identifier of the Task,
/// specifying "success" followed by an integer value (N), creates a Task dependency that must run N successful
/// times before resolving.
///
/// - ``MY_TASK(failures = INT)`` Creates a task dependency where ``MY_TASK`` is the identifier of the Task,
/// specifying "failures" followed by an integer value (N), creates a Task dependency that must run N failed
/// times before resolving.
///
/// - ``MY_TASK(custom = ...)`` Creates a task dependency where ``MY_TASK`` is the identifier of the Task,
/// specifying "custom" followed by an expression, creates a Task dependency that resolves based on a
/// custom criteria.
///
/// - ``dynamic(...)`` Creates a dynamic dependency with a closure as an argument inside that runs
/// when tempted to resolve.
///
/// Finally, the second argument is a configuration that specifies how to react whe dependencies aren't
/// resolved. Its optional and by default it simply skips the workflow with success via ``skip``.
///
/// With ``fail`` you can fail the workflow when dependencies aren't resolved, custom criteria can be achieved
/// via the ``custom(...)`` which runs when dependencies aren't resolved and then decided what to do.
///
/// ### Examples:
/// ```rust
/// use chronographer::prelude::*;
///
/// #[taskframe]
/// #[workflow(
///     dependency(dep = A), // Run this workflow when "A" is resolved, else skip if not
///     dependency(!A), // ... when "A" is NOT resolved ...
///     dependency(A || B), // ... when "A" OR "B" is resolved ...
///     dependency(A && B), // ... when "A" AND "B" is resolved ...
///     dependency(A ^ B), // ... when "A" XOR "B" is resolved ...
///     dependency(MyTask1(any = 3)), // ... when MyTask1 has run at least 3 times ...
///     dependency(MyTask2(success = 2)), // ... when MyTask2 has run at least 2 successful times ...
///     dependency(MyTask3(failures = 4)), // ... when MyTask3 has run at least 4 failed times ...
///     dependency(A, unresolve = fail), // Run this workflow when "A" is resolved, else fail if not
/// )]
/// pub async fn MyCoolTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     todo!()
/// }
/// ```
///
/// ## Condition Workflow Primitive
/// ```ignore
/// condition(predicate = IDENT | CLOSURE, secondary? = TASKFRAME_EXPR, on_false? = CONDITION_RETURN)
/// ```
///
/// The condition workflow primitive behaves identically to [`ConditionalTaskFrame`](chronographer::prelude::ConditionalTaskFrame),
/// it allows the specification of a predicate to be resolved truthfully before ultimately running the workflow.
/// Predicates are functions which run when the workflow is tempted to run and return a boolean value.
///
/// ### Arguments
/// - ``predicate`` The predicate function to run evaluating whenever or not to continue running the
/// workflow. Unlike other arguments, this one is required to be specified. The predicate must either
/// be an identifier or a closure.
///
/// - ``secondary`` A backup TaskFrame to run in case the predicate returns false, its optional and by
/// default runs nothing. Just like the fallback this follows the exact same expression syntax and
/// can inherit automatically the workflow or not via ``@``.
///
/// - ``on_false`` A configuration for the workflow primitive to act in case the predicate returns false.
/// This can either be ``error`` for erroring out or ``success`` to simply skip. **It is important to know**
/// when a secondary TaskFrame runs and fails, its error will be prioritized, if it succeeds then the condition
/// errors out regardless with its own error.
///
/// ### Examples:
/// ```rust
/// use chronographer::prelude::*;
///
/// #[taskframe]
/// #[workflow(
///     condition(MY_PREDICATE), // Run MY_PREDICATE, if it returns true -> run the workflow
///     condition(|| { true }), // Run the provided closure, if it returns true -> run the workflow
///     condition(MY_PREDICATE, on_false = error), // ... if it returns false -> error out
///     condition(MY_PREDICATE, MyTaskFrame2), // ... if it returns false -> run MyTaskFrame2
///     condition(
///         MY_PREDICATE,
///         MyTaskFrame2,
///         error
///     ), // ... if it returns false -> run MyTaskFrame2 and always error out
/// )]
/// pub async fn MyCoolTaskFrame(ctx: &TaskFrameContext) -> Result<(), String> {
///     todo!()
/// }
/// ```
///
/// # Expansion Semantics
/// The output typically depends on which macro ([`task`] or [`taskframe`]), the [`workflow`] macro is
/// attached to, for this reason read more about their expansion semantics in their respective documentation.
///
/// When attaching it anywhere else it infamously produces a compile-time error:
/// ```ignore
/// "Workflow attribute is unsupported outside of Tasks and TaskFrames (via the respective macros)"
/// ```
///
/// # Limitations
/// Due to Rust's macro limitations imposed, the [`workflow`] macro cannot support any type of expression
/// which can produce a non-obvious type in some specific scenarios such as [`TaskFrames`](chronographer::prelude::TaskFrame).
///
/// When it comes to [`TaskFrame`](chronographer::prelude::TaskFrame) "expressions", it is expected for the
/// input [`TaskFrames`](chronographer::prelude::TaskFrame) to be created from with use of [`taskframe`],
/// a wok-around for the base API users is to manually define a workflow method (either directly or indirectly
/// with a trait).
///
/// Additionally due to the way the workflow annotation macro is set up, some IDEs such as RustRover
/// may not display the color of the [`workflow`] macro nicely or rarely provide false positive
/// errors (in this case run ``cargo clean``).
///
/// Finally, while every workflow primitive defined in the core crate is supported through the
/// [`workflow`] macro interface. Any third party crates defining their own [`TaskFrames`](chronographer::prelude::TaskFrame)
/// will not work no matter what and as such require the switch to the base API.
///
/// # See Also
/// - [`task`] - A macro for defining Tasks, can also consume the workflow annotation.
/// - [`taskframe`] - A macro for defining TaskFrames, can also consume the workflow annotation.
/// - [`TaskFrame`](chronographer::prelude::TaskFrame) - The base API building block for defining workflows.
/// - [`TaskFrameBuilder`](chronographer::task::frame_builder::TaskFrameBuilder) - An alternative way to
/// write workflows in the base API ergonomically.
/// - [`RetriableTaskFrame`](chronographer::prelude::RetriableTaskFrame) - The base API equivalent of
/// the ``retry`` workflow primitive.
/// - [`FallbackTaskFrame`](chronographer::prelude::FallbackTaskFrame) - The base API equivalent of
/// the ``fallback`` workflow primitive.
/// - [`DelayTaskFrame`](chronographer::prelude::DelayTaskFrame) - The base API equivalent of
/// the ``delay`` workflow primitive.
/// - [`TimeoutTaskFrame`](chronographer::prelude::TimeoutTaskFrame) - The base API equivalent of
/// the ``timeout`` workflow primitive.
/// - [`DependencyTaskFrame`](chronographer::prelude::DependencyTaskFrame) - The base API equivalent of
/// the ``dependency`` workflow primitive.
/// - [`ConditionalTaskFrame`](chronographer::prelude::ConditionalTaskFrame) - The base API equivalent of
/// the ``condition`` workflow primitive.
#[proc_macro_attribute]
pub fn workflow(attrs: TokenStream, item: TokenStream) -> TokenStream {
    workflow::workflow(attrs, item)
}

/// The [`main`] attribute macro is an alternative more ergonomic way to write the main function
/// / entry-point for ChronoGrapher as opposed to manually making it using ``#[tokio::main]`` from the
/// ground up.
///
/// The bare minimal interface is essentially:
/// ```rust
/// use chronographer::prelude::*;
///
/// #[chronographer::main]
/// async fn main(scheduler: DefaultLiveScheduler<String>) {
///     todo!()
/// }
/// ```
///
/// The arguments inside the main function must be scheduler types in which there must be present at least
/// one argument (apart from that, there can be any number of them).
///
/// The macro sets up the tokio environment while also automatically initializing and starting the
/// schedulers. Finally, these schedulers can be accessed inside the main function
///
/// # Valid Targets
/// The [`main`] macro is applied primarily async functions, specifically a function with the name of
/// *main* that is contained inside a binary file and isn't a method.
///
/// # Attributes & Parameters
/// The [`main`] contains only two attribute parameters, the former being ``thread_count`` which allows
/// users to modify the number of threads allocated (the default behavior is the same as tokio)
///
/// While the latter being an attribute flag ``before_startup`` which modifies when the schedulers start,
/// by default they start after the user code has run, when set, it starts before the user code runs
///
/// # Expansion Semantics
/// Typically when the [`main`] macro expands, its structure is similar to:
/// ```ignore
/// fn main() {
///     let rt = tokio::runtime::Builder::new_multi_thread()
///         .enable_all()
///         .build()
///         .unwrap();
///
///     rt.block_on(async {
///         let scheduler = <DefaultLiveScheduler<String> as Default>::default();
///
///         (async {
///             // ...
///         }).await;
///
///         chronographer::prelude::Scheduler::start(&scheduler).await;
///         tokio::signal::ctrl_c().await.unwrap();
///     });
/// }
/// ```
///
/// Apart from the parameters, it works identically like the *main* function letting you use
/// as the return type ``Result<T, E>`` (you can also use ``?`` for error propagation), for instance:
/// ```ignore
/// #[chronographer::main]
/// async fn main() -> io::Result<()> {
///     Ok(())
/// }
/// ```
///
/// # Limitations
/// When it comes to [`Schedulers`](chronographer::prelude::Scheduler) coupled with their
/// [`SchedulerConfig`](chronographer::prelude::SchedulerConfig), they must implement the ``Default``
/// trait since it calls this specifically when initializing.
///
/// The only workaround if implementing the ``Default`` trait isn't possible is manually creating the
/// main function via ``#[tokio::main]`` macro and such.
///
/// # See Also
/// - [`Scheduler`](chronographer::prelude::Scheduler) - The type required for this macro in its argument(s).
/// - [`SchedulerConfig`](chronographer::prelude::SchedulerConfig) - The config container for the scheduler(s).
#[proc_macro_attribute]
pub fn main(attrs: TokenStream, item: TokenStream) -> TokenStream {
    entry::entry(attrs, item)
}

#[proc_macro]
pub fn cron(input: TokenStream) -> TokenStream {
    cron::cron(input)
}
