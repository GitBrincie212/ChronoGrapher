# Overview & Philosophy
Chronographer is an unopinionated composable scheduling library built in Rust. Its core philosophy is 
to provide fundamental scheduling primitives that users can compose into complex workflows, 
rather than follow the typical path of a monolithic, opinionated framework.

Core Tenets:
- **Composability:** Complex behavior is built by combining simple, single-responsibility TaskFrames (for tasks) or
by replacing various components with your own implementations of them (for schedulers and tasks).
- **Ergonomics:** Common patterns are easy to express via builders and default implementations, and can also be re-used
throughout other similar tasks, minimizing boilerplate code.
- **Extensibility:** Most core components are defined by traits (except for components which aren't meant to
be extensible such as priorities and schedulers which their core loop remains the same no matter what), allowing
users to provide their own implementations.
- **Efficiency:** Leverages Rust's ownership, various optimized crates and Rust's async models to build a robust 
and concurrent core.
- **Language Agnostic:** The core is designed to be the backbone for future language SDKs and distributed systems.

# Core Abstractions
There are 2 main systems at play for Chronographer, those being **Tasks** and **Schedulers**, they are broken down
into multiple sub parts which are used in combination to create them. Both are ``struct`` and provide methods to
use the underlying composites

## Task Hierarchy
![Task Abstraction Map](assets/Task%20Abstraction.png) <br />
Task is the most in-depth compared to the scheduler and is broken down to several parts to ensure
extensibility. Mainly being:
- **TaskFrame** This is a trait for the main unit of execution, it is also the second most flexible out of all other composites
for tasks. The power comes from composing **TaskFrame Chains** where TaskFrames wrap other TaskFrames, creating a tree 
hierarchy form, each TaskFrame influences the wrapped result depending on its order, configuration... etc. TaskFrames can
work with another system called TaskHooks
<br /> <br />
- **TaskHooks** This is also a trait, by far the most extensible composite Task has to offer, while not necessarily 
required, it enhances a Task by observing behavior / events from one or multiple of these categories:
    1. **Lifecycle Task Events** For when a task starts and when it ends
    2. **TaskHook Events** For when a task hook is attached and detached 
    3. **Local TaskFrame Events** For when a retry attempt starts, a timeout occurs... etc.

    The main gimmick of TaskHooks is they are independent entities from TaskFrame, they are not required,
    in fact, they are optional enhancements. TaskFrames can work with the system too, by attaching/detaching their own
    hooks and even getting a specific TaskHook instance. TaskHooks can also work in harmony 
<br /> <br />
- **TaskSchedule** This is a trait which computes the next time the task shall be executed, it is called when the scheduler
requests a <u>reschedule</u> and it can be non-deterministic (via an implementation of the trait)
<br /> <br />
- **TaskPriority** It is a simple enum dictating the importance of the task, the more importance, the greater the chances
for it to execute exactly on time (of course, under heavy workflow shall be used). This is the only composite which cannot
be extended
<br /> <br />
- **TaskScheduleStrategy** This trait tells how to handle rescheduling and tasks of the same instance being overlapped, 
should it be rescheduled when completed? Should it cancel the previous running task then run this current?

## Scheduler Hierarchy
![Task Abstraction Map](assets/Scheduler%20Abstraction.png) <br />
Scheduler is the brain of managing when and how the task is executed, it is more simple than the task struct but still
flexible enough. There are 3 composites:
- **SchedulerClock** This trait defines when is "now" and how to idle (sleep). An extension trait called 
``AdvancableSchedulerClock`` also allows for advancing time by a duration or to a specific point of time.
<br /> <br />
- **SchedulerTaskDispatcher** This trait controls how to execute tasks, the scheduler hands off a task that wants to be
executed, it is the dispatcher's job to balance the various task executions to ensure responsiveness even under heavy
workflows.
<br /> <br />
- **SchedulerTaskStore** This trait is the mechanism that stores the tasks which are scheduled, tasks can be retrieved
by earliest, they can be canceled, they can be scheduled... etc. This can be as simple as in-memory store to persistent
store.

The loop of the scheduler is simple:
- Retrieve earliest task
- Idle the clock til the point where the task wants to execute is reached
- Dispatch the task
- Reschedule the task when requested
- Repeat this process for every other task

# TaskFrame Chains
The ``TaskFrame`` trait is the second most flexible composite, one of its killer features is the wrapping of multiple task frames
to create complex execution mechanisms and reuse them throughout other tasks. There are 2 approaches to building a chain:

**TaskFrameBuilder:** By far the simplest way but limited to default task frame implementations and can't
be customized easily apart from the templates provided (the builder can be extended by utilizing the new-type pattern 
which wraps the builder inside a new struct)

**TaskFrame Manual Construction:** A bit tedious, but you can still do it this way since some task frames
may require more customization which may not be possible with the ``TaskFrameBuilder``'s builder templates

Here is an example to show the strength of the ``TaskFrameBuilder``:
```rust
TaskFrameBuilder::new(MY_PRIMARY_TASK_FRAME)
    .with_timeout(Duration::from_secs_f64(2.35))
    .with_instant_retry(NonZeroU32::new(3).unwrap())
    .with_dependency(MY_DEPENDENCY)
    .with_fallback(MY_SECONDARY_TASK_FRAME)
    .build();

// This translates to (more complex): 
FallbackTaskFrame::new(
    DependencyTaskFrame::builder()
    .task(
        RetriableTaskFrame::new_instant(
            TimeoutTaskFrame::new(
                MY_PRIMARY_TASK_FRAME,
                Duration::from_secs_f64(2.35)
            ),
            3
        )
    )
    .dependencies(vec![MY_DEPENDENCY])
    .build(),

    MY_SECONDARY_TASK_FRAME
);
```

Now say we change the dependency's behavior to return a success when dependencies aren't resolved, 
this would then become to:
```rust
FallbackTaskFrame::new(
    DependencyTaskFrame::builder()
    .task(
        RetriableTaskFrame::new_instant(
            TimeoutTaskFrame::new(
                MY_PRIMARY_TASK_FRAME,
                Duration::from_secs_f64(2.35)
            ),
            3
        )
    )
    .dependencies(vec![MY_DEPENDENCY])
    .dependent_behaviour(DependentSuccessOnFail)
    .build(),

    MY_SECONDARY_TASK_FRAME
);

// For the builder pattern, you would have to make use of the 
// new-type pattern and provide a method yourself
```

# Library Splitting
Instead of forcing everyone to download one single monolithic library, the project is split into
multiple libraries which all use the ``core``. the ``core`` contains the main traits, type aliases, 
implementation defined for Chronographer. Other programming language SDKs use the core to provide
a thin wrapper around the programming language, same goes for the distributed version of Chronographer (multiple
machines) and integrations
