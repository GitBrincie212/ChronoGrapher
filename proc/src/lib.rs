mod entry;
mod every;
mod cron;
mod utils;
mod task;
mod taskframe;
mod workflow;
mod hook;

use proc_macro::TokenStream;

/// The [`every`] proc-macro is an alternative ergonomic way to write an interval-based schedule as
/// opposed to manually constructing the [`TaskScheduleInterval`](chronographer::prelude::TaskScheduleInterval)
/// object from the ground up.
///
/// # Expansion Semantics
/// It uses under the hood [`TaskScheduleInterval`](chronographer::prelude::TaskScheduleInterval)
/// and calculates the appropriate time from the time-literal expression at compile-time.
///
/// The translated / expanded version typically looks like this:
/// ```ignore
/// TaskScheduleInterval::from_secs_f64(...).unwrap()
/// ```
///
/// # Invocation Syntax
/// This macro uses its own syntax to form an interval via multiple **Time Literals**. The
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
/// The [`every`] macro allows defining more specific times via multiple time literals sorted from most
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
/// Finally, the [`every`] macro additionally supports the use of decimals. Though you can only use it in
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
/// the [`every`] macro, though usually it isn't particularly necessary.
///
/// The same thing applies with higher-order time units (above days, such as weeks, months, years, decades... etc.) do
/// NOT include a time literal. Though a workaround of this issue is using higher values for days such as:
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
/// Though it allows for an escape hatch to override names with two respective attribute parameters
/// (see below for more info about).
///
/// Everything else is almost identical to the [`taskframe`] attribute macro as such it's recommended
/// to read more about it
///
/// # Valid Targets
/// The [`task`] macro is applied primarily to async functions, these functions cannot be methods (include
/// &self or &mut self as the first argument in a struct / enum / trait).
///
/// # Attributes & Parameters
/// The [`task`] macro contains 4 attribute parameters, one of which is required while the other three
/// are optional, and one out of these is an attribute flag:
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
/// For the former where the [`Task`](chronographer::prelude::Task) is a singleton, it is similar to:
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
/// error type to use respectively. The latter, on the other hand, typically takes the form of:
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
/// works as it borrows the same syntax with a minor issue (see the limitations below).
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
/// While [`taskframe`] generics work mostly out of the box, there is an issue for [`task`].
/// Due to static-based limitations, there can be no singleton Task with generics. As such, either remove
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

/// The [`taskframe`] attribute macro is an alternative, more ergonomic way to write
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
/// When it comes to creating full Tasks objects, it is recommended to check the [`task`] attribute macro,
/// its interface is almost identical, and in fact the [`taskframe`] macro is used under the hood.
///
/// # Valid Targets
/// The [`taskframe`] macro is applied primarily async functions, these functions cannot be methods (include
/// &self or &mut self as the first argument in a struct / enum / trait).
///
/// # Attributes & Parameters
/// The [`taskframe`] contains no attribute parameters (apart from an internal one which under any
/// circumstances should **NOT** be used due to being an antipattern).
///
/// # Expansion Semanticse
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
/// Then the user must extract those values and name them themselves, which is slightly cumbersome and
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
/// async limitations. *Should be noted generics aren't limited to either arguments or the return type;
/// They work the same as Rust generics*.
///
/// Additionally, we have one constant parameter ``N`` that is a type of usize, this isn't used anywhere
/// obvious but could theoritically be used in our code. Moreover, we also have a ``where`` clause which is
/// supported. Alternatively, we can specify the trait bounds directly if we want to.
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

/// The [`workflow`] attribute macro is a special macro from the rest macro; Just like with [`hooks`], it
/// behaves as an annotation working alongside [`task`] and [`taskframe`] rather than a macro which
/// transforms directly the input provided into something new.
///
/// With that said, it allows users to write ergonomically workflows ([`TaskFrames`](chronographer::prelude::TaskFrame)
/// stacking on top of each other) which works on top of the function / code they have already written.
///
/// Users can write any kind of workflow via the provided built-in workflow primitives,
/// from the simplest (one workflow primitive) to the most complex (with basically an infinite number of these).
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
/// named (with the argument's name) or on special occasions contain a variable number of arguments.
///
/// In essence, workflow primitive arguments behave similarly to Python's ``args`` / ``kwargs``, which
/// means positional arguments should **NOT** be followed after named arguments.
///
/// As previously said, there can be any number of workflow primitives. However, the [`workflow`] macro
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
/// it allows retrying the workflow up to a specified number of times with a configurable delay and error filter.
///
/// ### Arguments
/// - ``max`` The upper bound of times to retry a workflow until it succeeds. Unlike other arguments,
/// this one is required to be specified; Additionally, it can be any source of an integer as long as it
/// can be converted internally to a ``NonZeroU32``.
///
/// - ``delay`` The delay in-between every retry, this can be as simple as ``immediate``, providing a
/// constant time / duration literal or even a backoff strategy. It's fully optional to specify and
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
/// - ``when`` The error filter is composed of a list of patterns encapsulated in brackets (``[...]``)
/// with optionally an exclamation mark (``!``) as a prefix. When used without any exclamation marks, it's
/// a whitelist (one of the patterns must match), whereas with one it turns into a blacklist (none of the
/// patterns must match). Patterns match based on the error's structure, it's fully optional to specify,
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
///     retry(11, delay = jitter(equal, 2s)), // ... with an equally jittered delay of 2 seconds
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
/// These [`TaskFrames`](chronographer::prelude::TaskFrame) "expressions" must be a simple constructor
/// method that fully exposes the type directly, for example, plain ``MyType`` or ``MyType::new()``.
///
/// Generics are also supported as you can specify ``MyType::<T>::new()`` but no construction, in
/// addition to type aliases. What is not supported though are constants and macros.
///
/// ChronoGrapher's [`workflow`] macro is clever in that it can recognize when users call the ``workflow``
/// method and automatically inherit the workflow (more formerly known as **Workflow Inheritance**).
///
/// While using any plain constructor only includes the target [`TaskFrame's`](chronographer::prelude::TaskFrame) raw code,
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
/// The workflow primitive accepts only argument that being ``delay`` which is a duration-based
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
/// The workflow primitive accepts only argument that being ``duration`` which is a duration-based
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
/// based on a criteria. When the workflow is tempted to run again, it can skip, fail with an error, or
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
/// it allows the specification of a required dependency to be resolved before ultimately running the workflow.
/// The shape of the dependency can be as simple as a flag to as complex as a boolean expression with Tasks.
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
/// dependency by using the following "atomic" expressions:
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
/// default runs nothing. Just like the fallback this follows the exact same expression syntax
///
/// - ``on_false`` A configuration for the workflow primitive to act in case the predicate returns false.
/// This can either be ``error`` for erroring out or ``success`` to simply skip. **It is important to know**
/// when a secondary TaskFrame runs and fails, its error will be prioritized, if it succeeds, then the condition
/// errors out regardless of its own error.
///
/// ### Examples:
/// ```rust
/// use chronographer::prelude::*;
///
/// #[taskframe]
/// #[workflow(
///     condition(MY_PREDICATE), // Run MY_PREDICATE if it returns true -> run the workflow
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
/// When attaching it anywhere else, it infamously produces a compile-time error:
/// ```ignore
/// "Workflow attribute is unsupported outside of Tasks and TaskFrames (via the respective macros)"
/// ```
///
/// # Limitations
/// Due to Rust's macro limitations imposed, the [`workflow`] macro cannot support any type of expression
/// that can produce a non-obvious type in some specific scenarios such as [`TaskFrames`](chronographer::prelude::TaskFrame).
///
/// When it comes to [`TaskFrame`](chronographer::prelude::TaskFrame) "expressions", as stated before,
/// they must not have any indirections such as constants and macros that obfuscate the type, however,
/// it should be noted generics in the method's constructors are unsupported (i.e. ``MyType::new::<T>()``).
///
/// Additionally, due to the way the workflow annotation macro is set up, some IDEs such as RustRover
/// may not display the color of the [`workflow`] macro nicely or rarely provide false positive
/// errors (in this case run ``cargo clean``).
///
/// Finally, while every workflow primitive defined in the core crate is supported through the
/// [`workflow`] macro interface. Any third party crates defining their own [`TaskFrames`](chronographer::prelude::TaskFrame)
/// will not work no matter what and as such require the switch to the base API.
///
/// # See Also
/// - [`task`] - A macro for defining Tasks can also consume the workflow annotation.
/// - [`taskframe`] - A macro for defining TaskFrames can also consume the workflow annotation.
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

/// The [`hook`] attribute macro is a special macro from most macros; It is similar in spirit to [`workflow`]
/// but unlike the [`workflow`] macro annotation. This macro can act both as a macro annotation and as
/// atypical attribute macro depending on the context.
///
/// With that said, it allows users to write ergonomically [`TaskHooks`](chronographer::prelude::TaskHook),
/// both implementing / defining them and even registering / attaching them.
///
/// The system allows for almost identical levels of flexibility just like the Base API but achieved with
/// less boilerplate and more readability in both implementation and attachment phases.
///
/// The bare minimal of both implementation and attachment of a TaskHook looks as follows:
/// ```rust
/// use chronographer::prelude;
///
/// // ============== [IMPLEMENTATION PHASE (START)] ==============
///
/// #[derive(Default)]
/// struct MyTaskHook;
///
/// #[hook]
/// impl MyTaskHook {
///    async fn OnTaskStart(&self, ctx: &TaskHookContext) { /* ... */ }
///    async fn OnTaskEnd(&self, ctx: &TaskHookContext, error: Option<&dyn TaskError>) { /* ... */ }
///    async fn OnRetryAttemptEnd(&self, ctx: &TaskHookContext, _retry: u32, _error: Option<&dyn TaskError>) { /* ... */ }
/// }
///
/// // ============== [IMPLEMENTATION PHASE (END)] ==============
/// //
/// // ============== [ATTACHMENT PHASE (START)] ================
///
/// #[task(schedule = every!(2s))]
/// #[hook(
///     // Auto-attachment
///     auto(MyTaskHook),
///
///     // Manual Listening (Same code as above basically)
///     my_inst = MyTaskHook::default(),
///     OnTaskStart: my_inst,
///     OnTaskEnd: my_inst,
///     OnRetryAttemptEnd: my_inst
/// )]
/// pub async fn MyCoolTaskFrame(ctx: TaskFrameContext) -> Result<(), String> {
///     todo!()
/// }
///
/// // ============== [ATTACHMENT PHASE (END)] ==================
/// ```
///
/// # Valid Targets
/// The [`hook`] macro can either be applied as an attribute macro to a simple ``impl`` block containing
/// one identifier that being the TaskHook to generate the implementation-phase. Or alternatively as
/// a macro annotation in an async function utilizing the [`task`] macro to generate the attachment-phase.
///
/// # Attributes & Parameters
/// Depending on where the macro is used, a different syntax is available for use. The most simple phase
/// of the two in terms of attributes & parameters is the implementation phase.
///
/// It contains only one parameter that being ``auto_attach``. It can either be explicitly specified
/// by itself, be assigned a value (an identifier) or have as prefix an exclamation mark (!), by default,
/// it is enabled with the usual method name of "auto_attach".
///
/// Explicitly enabling it is the same as the default whereas with assignment an identifier it both enables
/// the option and overrides the method name of "auto_attach" to a different one. Whereas the third option
/// disables the option altogether.
///
/// The purpose of ``auto_attach`` when it is enabled as an option (either overriding the name or not) is
/// to provide a method to allow automatically-attaching default events of a TaskHook onto a Task.
///
/// The macro can also be used in an async function inside the ``impl`` block with the attribute macro
/// ``#[hook]`` as a macro annotation. It provides a ``default`` on/off toggle and a ``listen`` parameter
/// which are explained below.
///
/// ---
///
/// While in the attachment-phase, traditional parameters in the sense of a static config are non-existent.
/// The macro embeds a full mini DSL to express connections between TaskHook instances and events to listen
/// to that specific Task.
///
/// The DSL consists of statements which are separated by commas, each statement has the following grammar:
///
/// - ``auto(<TYPE> | <TYPE>::<IDENT>)``: Attaches automatically the default events that ``<TYPE>`` TaskHook
/// provides assuming of course the TaskHook has the auto-attach method (default name is "auto_attach").
/// If the name is overridden, the alternative syntax may be used via ``<IDENT>`` to specify the method name.
///
/// - ``<IDENT> = <VALUE>``: Assigns a TaskHook instance with ``<VALUE>`` as a constructor and the
/// ``<IDENT>`` refers to the instance just like atypical variable assignment.
///
/// - ``<TYPE>: <VALUE>``: Listens to the specific event ``<TYPE>`` with the instance ``<VALUE>`` either
/// being an identifier which references an existing instance or some other expression that creates a new
/// instance only for that event.
///
/// # Expansion Semantics
/// The expansion of the [`hooks`] macro heavily depends on the context it is used in. However, for the
/// implementation phase it typically looks something like:
/// ```rust
/// use chronographer::prelude::*;
///
/// /* Input:
/// #[hook]
/// impl MyTaskHook {
///    async fn OnTaskStart(&self, ctx: &TaskHookContext) { /* <...> */ }
///    async fn OnTaskEnd(&self, ctx: &TaskHookContext, error: Option<&dyn TaskError>) { /* <...> */ }
///    async fn OnRetryAttemptEnd(&self, ctx: &TaskHookContext, retry: u32, error: Option<&dyn TaskError>) { /* <...> */ }
///    async fn OnMyCustomEvent(&self, ctx: &TaskHookContext, param1: String, param2: Vec<u8>) { /* <...> */ }
/// }
/// */
///
/// impl MyTaskHook {
///     pub async fn auto_attach(hooks_layer: &impl TaskHookLayer, instance: impl Deref<Target=Arc<Self>>) {
///         hooks_layer.attach::<OnTaskStart>(instance.clone()).await;
///         hooks_layer.attach::<OnTaskEnd>(instance.clone()).await;
///         hooks_layer.attach::<OnRetryAttemptEnd>(instance.clone()).await;
///     }
/// }
///
/// #[async_trait]
/// impl TaskHook<OnTaskStart> for MyTaskHook {
///     async fn on_event(&self, ctx: &TaskHookContext, _payload: &<OnTaskStart as TaskHookEvent>::Payload<'_>) {
///         /* <...> */
///     }
/// }
///
/// #[async_trait]
/// impl TaskHook<OnTaskEnd> for MyTaskHook {
///     async fn on_event(&self, ctx: &TaskHookContext, payload: &<OnTaskEnd as TaskHookEvent>::Payload<'_>) {
///         let error: &Option<&dyn TaskError> = payload;
///         /* <...> */
///     }
/// }
///
/// #[async_trait]
/// impl TaskHook<OnRetryAttemptEnd> for MyTaskHook {
///     async fn on_event(&self, ctx: &TaskHookContext, payload: &<OnRetryAttemptEnd as TaskHookEvent>::Payload<'_>) {
///         let (retry, error): &(u32, Option<&dyn TaskError>) = payload;
///         /* <...> */
///     }
/// }
///
/// #[async_trait]
/// impl TaskHook<OnMyCustomEvent> for MyTaskHook {
///     async fn on_event(&self, ctx: &TaskHookContext, payload: &<OnMyCustomEvent as TaskHookEvent>::Payload<'_>) {
///         let (param1, param2): &(String, Vec<u8>) = payload;
///         /* <...> */
///     }
/// }
/// ```
///
/// Each async method corresponds to a trait implementation of ``TaskHook<E>`` where ``E`` is the method name
/// or the event you are listening to. Do note that visibility modifiers (``pub``, ``pub(crate)``... etc.)
/// are fully optional.
///
/// In our example we have basic names but the macro also allows to listen to generic-based events:
/// ```rust
/// #[hook]
/// impl MyTaskHook {
///    async fn OnHookAttach<OnTaskStart>(&self, ctx: &TaskHookContext, hook: &dyn TaskHook<OnTaskStart>) { /* <...> */ }
///    async fn OnHookDetach<E: TaskHookEvent>(&self, ctx: &TaskHookContext, hook: &dyn TaskHook<E>) { /* <...> */ }
///    async fn __anonymous__<E: TaskLifecycleEvents>(&self, ctx: &TaskHookContext) { /* <...> */ }
/// }
/// ```
///
/// There are three variations for listening to generic-based events, each with their own syntax:
/// - ``MyGenericEvent<MySpecificEvent>`` Narrows the generic the ``MyGenericEvent<T>`` has to only be ``MySpecificEvent``
/// (while obviously following the bounds set by the generic event), apart from that, it acts identically to ``OnTaskStart``,
/// ``OnTaskEnd, ``OnMyCustomEvent``... etc.
///
/// - ``MyGenericEvent<E: Bound1 + Bound2 ...>``Expresses any kind of event inside ``MyGenericEvent<T>`` as long as the
/// event parameter has the bounds. The event generic must also follow the given bounds set by ``MyGenericEvent<T>``.
///
/// - ``__anonymous__<E: Bound1 + Bound2 ...>`` Unlike the above case which narrows to ``MyGenericEvent<T>``. This form
/// freely expresses any kind of event as long as it's in the bounds you set.
///
/// > **Note 1#:** The bottom two variants have a key limitation regarding the ``auto_attach``. This topic
/// is explained below and summarized in the limitations section
///
/// > **Note 2#:** The first case may look like an unbounded generic of any kind of event, but it behaves
/// completely different. If you need to represent every single type of event use ``E: TaskHookEvent`` instead.
///
/// With the trait implementations for the specific events out, the macro also generates the auto_attach
/// method as seen above with every event we've written attached by default. We can specify our own defaults,
/// however, by embedding the ``#[hook(...)]`` macro annotation and using the ``default`` boolean parameter.
///
/// Rewriting our previous simple code with this mind, it transforms to:
/// ```rust
/// #[hook]
/// impl MyTaskHook {
///
///    #[hook(default)]
///    async fn OnTaskStart(&self, ctx: &TaskHookContext) { /* <...> */ }
///
///    #[hook(default)]
///    async fn OnTaskEnd(&self, ctx: &TaskHookContext, error: Option<&dyn TaskError>) { /* <...> */ }
///
///    async fn OnRetryAttemptEnd(&self, ctx: &TaskHookContext, retry: u32, error: Option<&dyn TaskError>) { /* <...> */ }
///
///    #[hook(default)]
///    async fn OnMyCustomEvent(&self, ctx: &TaskHookContext, param1: String, param2: Vec<u8>) { /* <...> */ }
///
/// }
/// ```
/// Now our auto-attach method only attaches ``OnTaskStart``, ``OnTaskEnd`` and ``OnMyCustomEvent`` and not
/// ``OnRetryAttemptEnd``. Currently, our generic-based events (except the first case) disallow auto-attachement,
/// the ``default`` parameter solves this elegantly via:
/// ```rust
/// #[hook]
/// impl MyTaskHook {
///    #[hook(default)]
///    async fn OnHookAttach<OnTaskStart>(&self, ctx: &TaskHookContext, hook: &dyn TaskHook<OnTaskStart>) { /* <...> */ }
///
///    #[hook(default = [OnMyCustomEvent, OnTaskStart, OnTaskEnd])]
///    async fn OnHookDetach<E: TaskHookEvent>(&self, ctx: &TaskHookContext, hook: &dyn TaskHook<E>) { /* <...> */ }
///
///    #[hook(default = [OnRetryAttemptStart, OnRetryAttemptEnd])]
///    async fn __anonymous__<E: TaskLifecycleEvents>(&self, ctx: &TaskHookContext) { /* <...> */ }
/// }
/// ```
/// In the above example, the first case doesn't need additional parameters as there is one singular event
/// to take care of. Whereas in the other two, there can be an unknown number of events and thus needs
/// specification for which are the defaults.
///
/// Though there is a specific edge-case where there is a multitude of basic events but only a very few
/// number of ambigious generic-based events. Providing defaults might be impossible, and thus you may need
/// to provide defaults for the rest to discard it. To combat this, use ``#[hooks(!default)]``
///
/// An additional parameter which can be used is the ``listen``, unlike ``default`` it can be assigned a value.
/// Do note, this value denotes the event to listen to and is prioritized over the method's name when it exists,
/// which allows for better self-documenting code in some cases:
/// ```rust
/// #[hook]
/// impl MyTaskHook {
///
///    #[hook(listen = OnTaskStart)]
///    async fn initialization_phase(&self, ctx: &TaskHookContext) { /* <...> */ }
///
///    #[hook(default, listen = OnTaskEnd)]
///    async fn shutdown_phase(&self, ctx: &TaskHookContext, error: Option<&dyn TaskError>) { /* <...> */ }
///
///    async fn OnRetryAttemptEnd(&self, ctx: &TaskHookContext, retry: u32, error: Option<&dyn TaskError>) { /* <...> */ }
///
///    #[hook(default)]
///    async fn OnMyCustomEvent(&self, ctx: &TaskHookContext, param1: String, param2: Vec<u8>) { /* <...> */ }
///
/// }
/// ```
///
/// ---
///
/// In terms of the attacement-phase. The expansion essentially involves variables for the instances,
/// attaching the corresponding instances to their specified events along with the handling auto-attachement.
///
/// # Limitations
/// The [`hook`] macro cannot provide an auto-attach if the ``impl`` block contains ambigious generic-based
/// methods with an unknown number of defaults. The solution is either to disable it fully via ``#[hook(!auto_attach)]``
/// at the top of the ``impl`` block.
///
/// Or alternatively specify defaults for those generic-based methods via the macro annotation ``#[hooks(default = [...])]``
/// which resides at the top of the function that hosts the ambigious generic-based event.
///
/// Another limitation is the fact the macro cannot represent stateful container based [`TaskHooks`](chronographer::prelude::TaskHook)
/// which can be easily solved via implementing the [`NonObserverTaskHook`](chronographer::prelude::NonObserverTaskHook) trait.
///
/// # See Also
/// - [`TaskHook`](chronographer::prelude::TaskHook) - One of the base API building block for this macro.
/// - [`TaskHookEvent`](chronographer::prelude::TaskHookEvent) - Another base API building block for this macro.
/// - [`NonObserverTaskHook`](chronographer::prelude::NonObserverTaskHook) - For specifying stateful-container based TaskHooks.
/// - [`task`] - Works with this macro to allow for the attachement phase.
/// - [`workflow`] - A closely-related "cousin" to this macro but for describing workflows.
#[proc_macro_attribute]
pub fn hook(attrs: TokenStream, item: TokenStream) -> TokenStream {
    hook::hook(attrs, item)
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

/// The [`cron`] proc-macro is an alternative ergonomic way to write CRON-based schedules as opposed
/// to manually constructing the [`TaskScheduleCron`](chronographer::prelude::TaskScheduleCron)
/// object from the ground up.
///
/// # Expansion Semantics
/// It utilizes under the hood [`TaskScheduleCron`](chronographer::prelude::TaskScheduleCron) and
/// parses, validates and lowers the CRON expression into the appropriate
/// [`CronField`](chronographer::task::schedule::CronField) values at compile-time. Any malformed
/// expression is reported as a compile-time error pointing at the offending field.
///
/// The expanded version typically looks this:
/// ```ignore
/// TaskScheduleCron::new([/* CronField per field */])
/// ```
///
/// A side effect of compile-time checking is it doesn't affect runtime at all which means a slight
/// performance boost if creating multiple constant CRON schedules. Even without the performance boost,
/// the [`cron`] proc-macro **should be preferred in all cases when dealing with a constant CRON expression.**
///
/// Alternatively for the rare cases which a runtime-based cron expression is needed, switch to
/// [`TaskScheduleCron::from_str`](chronographer::prelude::TaskScheduleCron::from_str) to dynamically
/// construct one at runtime.
///
/// # Invocation Syntax
/// This macro uses standard CRON syntax composed of **6 whitespace-separated fields**, sorted from
/// most significant / shortest period to least significant.
///
/// The fields and their valid ranges are listed below in order:
/// - Second = ``0-59``
/// - Minute = ``0-59``
/// - Hour = ``0-23``
/// - Day of Month = ``1-31``
/// - Month = ``1-12`` or names (``JAN``, ``FEB``, ``MAR``, ..., ``DEC`` where ``JAN`` = 1)
/// - Day of Week = ``1-7`` or names (``SUN``, ``MON``, ``TUE``, ...,``SAT``, where ``SUN`` = 1)
///
/// Month and day-of-week names are case-insensitive, so ``JAN``, ``jan`` and ``Jan`` are all accepted.
/// It should also be mentioned that months and day-of-week constants are **ONLY** allowed on their
/// respective fields.
///
/// ```rust
/// use chronographer::cron;
///
/// cron!(0 0 * * * *) // Every hour, on the hour
/// cron!(0 30 9 * * *) // Every day at 09:30:00
/// cron!(0 0 12 * JAN MON) // At noon, on Mondays in January
/// ```
///
/// # Field Expressions
/// The supported expressions map directly onto the [`CronField`](chronographer::task::schedule::CronField) variants:
/// - ``*`` A **Wildcard**, matches every value of the field.
/// - ``?`` An **Unspecified** value, used for the day-of-month / day-of-week fields.
/// - ``<N>`` An **Exact** integer value (e.g. ``5``).
/// - ``<A>-<B>`` A **Range** from integer ``A`` to integer ``B`` inclusive (e.g. ``9-17``).
/// - ``<EXPR>/<N>`` A **Step**, matches every integer ``N``th value across any expression ``EXPR`` (e.g. ``*/15`` or ``0-30/5``).
/// - ``<EXPR1>,<EXPR2>,<EXPR3>`` A **list**, matches any of the comma-separated sub-expressions.
/// - ``L`` The **Last** value of the field (last day of month, or last day of week).
/// - ``<N>L`` The last given weekday (e.g. ``5L`` for the last occurrence of that weekday).
/// - ``<N>W`` The **Nearest Weekday** to day-of-month ``N``.
/// - ``<A>#<B>`` The **Nth Weekday**, the integer ``B``-th occurrence of weekday integer ``A`` in the month.
///
/// A couple of combined examples are demonstrated below:
/// ```rust
/// use chronographer::cron;
///
/// cron!(0 */15 * * * *) // Every 15 minutes
/// cron!(0 0 9-17 * * MON-FRI) // Hourly from 09:00 to 17:00 on weekdays
/// cron!(0 0 0 L * ?) // At midnight on the last day of the month
/// cron!(0 0 12 ? * 6#3) // At noon on the third Friday of every month
/// cron!(0 0 8 15W * ?) // At 08:00 on the nearest weekday to the 15th
/// cron!(0 0,30 * * * *) // On the hour and the half-hour
/// ```
///
/// It is highly recommended to read in detail [Quartz's CRON Trigger Documentation](https://www.quartz-scheduler.org/documentation/quartz-2.3.0/tutorials/crontrigger.html)
/// as the proc-macro is heavily based on the same syntax structure.
///
/// # Limitations
/// Since the [`cron`] macro is based on [Quartz's CRON Trigger](https://www.quartz-scheduler.org/documentation/quartz-2.3.0/tutorials/crontrigger.html),
/// it is subject to the same limitation as it, including the limitation to not specify anything below
/// a second (milliseconds, nanoseconds... etc.).
///
/// To circumvent this issue, it is recommended to switch schedules. For basic intervals it is best to use
/// the [`every`] macro and for even more complex schedules, use [`calendar`] macro.
///
/// # See Also
/// - [`TaskScheduleCron`](chronographer::prelude::TaskScheduleCron) - The base API equivalent
/// - [`CronField`](chronographer::task::schedule::CronField) - The lowered representation of each field
/// - [`every`] - For simpler interval-based schedules, including sub-second granularity
/// - [`calendar`] - For making schedules with a human-readable calendar expression
/// - [`TaskSchedule`](chronographer::prelude::TaskSchedule) - The trait that makes schedules possible
#[proc_macro]
pub fn cron(input: TokenStream) -> TokenStream {
    cron::cron(input)
}
