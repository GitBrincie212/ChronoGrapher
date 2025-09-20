# Overview & Philosophy
Chronolog is an unopinionated composable scheduling library built in Rust. Its core philosophy is 
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
There are 2 main systems at play for Chronolog, those being **Tasks** and **Schedulers**, they are broken down
into multiple sub parts which are used in combination to create them. Both are ``struct`` and provide methods to
use the underlying composites

## Task Hierarchy
![Task Abstraction Map](assets/Task%20Abstraction.png)
Task is the most in-depth compared to the scheduler and is broken down to several parts to ensure
extensibility. Mainly being:
- **TaskMetadata** This is a trait that acts as a container for wrapping relevant state the task needs to track throughout its
lifetime. It is mostly a reactive container where fields are wrapped via ``ObservableField`` which other
code pieces can listen to upcoming changes to the value. One can implement the trait to add more state to be
tracked or use the default metadata container
<br /> <br />
- **TaskFrame** This is a trait for the main unit of execution, it is also one of the most flexible out of all other composites
for tasks. It contains <u>lifecycle task events</u> (start and end more specifically) and <u>local task frame events</u>
(they depend on the type of task frame), task frames can also be wrapped by other task frames (which will be explained soon)
<br /> <br />
- **TaskSchedule** This is a trait which computes the next time the task shall be executed, it is called when the scheduler
requests a <u>reschedule</u> and it can be non-deterministic (via an implementation of the trait)
<br /> <br />
- **TaskErrorHandler** A simple handler that is called when the task fails, it is mainly used to handle stuff such
as closing database connections, cleaning up... etc. By default, it silently ignores the error, which can be changed by
implementing the trait
<br /> <br />
- **TaskPriority** It is a simple enum dictating the importance of the task, the more importance, the greater the chances
for it to execute exactly on time (of course, under heavy workflow shall be used). This is the only composite which cannot
be extended
<br /> <br />
- **TaskExtension** It is a trait which allows the extension of tasks, if the current fields aren't enough for you, you
can always implement this trait to add additional content, by default it doesn't add anything
<br /> <br />
- **SchedulerStrategy** This trait tells how to handle rescheduling and tasks of the same instance being overlapped, 
should it be rescheduled when completed? Should it cancel the previous running task then run this current?

## Scheduler Hierarchy
![Task Abstraction Map](assets/Scheduler%20Abstraction.png)
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
The ``TaskFrame`` trait is the most flexible composite, one of its killer features is the wrapping of multiple task frames
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
implementation defined for Chronolog. Other programming language SDKs use the core to provide
a thin wrapper around the programming language, same goes for the distributed version of chronolog (multiple
machines) and integrations