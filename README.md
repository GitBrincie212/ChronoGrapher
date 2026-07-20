<h1 align="center">ChronoGrapher</h1>
<img src="./assets/ChronoGrapher Logo.png" alt="Chronographer Banner" />

<img align="center" src="assets/Chronographer Divider.png" />
<h1 align="center">One Unified Scheduling Kernel, Unlimited Power</h1>

ChronoGrapher is the **Scheduling / Orchestration Kernel** (related to job scheduling and workflow orchestration but more
generalized and adaptive) that brings unified scheduling to your entire stack.

1. **Unified Multi-Language API:** Coordinate workflows across Python, TypeScript/JavaScript, Rust, and Java with a
   single and beautiful API tailor-made for each programming language, no more awkward glue code needed.

2. **Unopinionated by Design:** ChronoGrapher provides core scheduling without forcing features. Major capabilities
   offered as optional extensions such as **Cloud Infrastructure Support**, **Distributed Systems**, **Markers API** which
   build on top of the core.

3. **Hyper-Extensible Architecture:** The best in market when it comes to extensibility, as demonstrated with its
   extensions / integrations, its core is exceptionally powerful with its design philosophy being
   "Minimalism over Bloat, Emergent over Predefined and Simplicity over Complexity".

4. **Scale Effortlessly:** Rust-powered engine scales from a single machine to cloud infrastructure and finally to
   distributed clusters seamlessly, ChronoGrapher scales with your ever-growing ambitions.

5. **Adaptive Crash-Resistant Durability:** Never lose task progress and state, with its revolutionary persistence
   model. Guaranteeing high durability and low overhead in performance, developers should never worry about failures.
   <u>(COMING SOON)</u>

**Get started in 30 seconds**, with a simple "Hello World" example in ChronoGrapher written in Rust (≥ 1.92v). Other
languages look similar:
```rust
use chronographer::prelude::*;

#[task(schedule = every!(4s))]
async fn HelloWorldTask(ctx: &TaskFrameContext) -> Result<(), String> {
  println!("Hello World");
  Ok(())
}

#[chronographer::main]
async fn main(scheduler: DefaultLiveScheduler<String>) {
  let task = HelloWorldTask::instance();
  scheduler.schedule(&task).await;
}
```

<img align="center" src="assets/Chronographer Divider.png" />
<h1 align="center">Solving Modern Scheduling Challenges</h1>
Today's applications are inherently ambitious, polyglot and complex. Yet job scheduling and workflow orchestration for the 
most part remain language-bound, constraint and sometimes with inconsistent developer experience.

A typical modern stack leverages multiple programming languages. Each requiring its own scheduling ecosystem:

- **Python:** Apache Airflow, Celery, Prefect, APScheduler
- **TypeScript/JavaScript** Agenda, Bree, BullMQ, Bottleneck
- **Rust:** cron_tab, tokio_task_scheduler, tokio-cron
- **Java:** Quartz, Spring Scheduler
- **Misc:** Temporal, Cadence, CRON

**The Current Challenge:**
Most solutions face one or more fundamental limitations:
- **Language Isolation:** Bound to single ecosystems, either leading to the use of different solutions for every programming
  language with their own APIs to learn, their own tradeoffs... etc. Or alternatively requiring complex and awkward glue-code.
- **Scalability Issues:**  Difficult to scale and extend beyond what the solution was initially intended for, without
  requiring significant re-engineering or weird hacky solutions to common problems. Both approaches are a maintenance nightmare to
  developers.
- **Inconsistent Developer Experience:** Inconsistent poorly-maintained documentation quality and at worst outright outdated,
  opinionated patterns with different systems doing the same thing, and an overall steep learning curve across tools.
- **Highly Niche:** Some job schedulers and workflow orchestrations are built for one thing in mind, while its good for
  solving that particular problem pretty well, there is no generalized blueprint which can be adapted across industries
  freely.

**The ChronoGrapher Approach:**
We believe developers deserve better than fragmented scheduling experiences. While no solution is perfect,
ChronoGrapher's polyglot architecture, performance-first design, and extensibility focus represent significant
breakthroughs in scheduler design, eliminating the need to master multiple disjointed systems.
<img align="center" src="assets/Chronographer Divider.png" />
<h1 align="center">Core Capabilities / Key Features</h1>

There are many features that make up ChronoGrapher (some are accessed via first-party extensions). Extension-provided
features are opt-in and never enforced, we will focus on a few key features defined in the core such as:

### Composable Workflow Architecture

ChronoGrapher's power comes from its modular task system. Build complex workflows by composing simple, reusable components:

```rust
#[task(schedule = every!(4s))]
#[workflow(
  dependency(
        HealthCheckTask && (DatabaseCheckTask || SystemUpdateTask)
  ),  // Run workflow only if dependencies are resolved
  retry(3, delay = 1s), // If everything fails retry with a delay up to 3 times
  fallback(HandlePaymentFailure), // If the below "sub-workflow" fails, execute this
  timeout(30s), // Cannot exceed more than 30 seconds
)]
async fn HandleCleanupTask(ctx: &TaskFrameContext) -> Result<(), String> {
  // <...>
}
```

Some of the available workflow primitive types are:

- **🔄 RetriableFrame:** Automatic retries for a TaskFrame with configurable backoff strategies
- **⏱️ TimeoutFrame:** Enforce execution time limits on a TaskFrame (otherwise a timeout error is thrown if exceeded)
- **🚫 FallbackFrame:** If the primary TaskFrame fails, switch to a secondary TaskFrame
- **🎯 ConditionalFrame:** Conditional execution of a TaskFrame via an outside predicate
- **📋 ThresholdTaskFrame:** Similar to ``TimeoutFrame`` but for enforcing a threshold for run count.
- **🔗 DependencyFrame:** Executes a TaskFrame if its dependencies are resolved (can depend on other Tasks)
- **💤 DelayFrame:** Delays the execution of a TaskFrame

### Powerful Hook-Based System

Fine-grain reactivity for Tasks at a deep level, or append your own stateful containers, or even mix both! Unlimited freedom
as ``TaskHooks`` are the backbone of extensibility in ChronoGrapher:
```rust
// A basic example for "integration" with Prometheus
struct PrometheusMetricsHook;

#[hook]
impl PrometheusMetricsHook {
  async fn OnTaskStart(&self, ctx: &TaskHookContext) {
        // ...Increment the number of running Tasks and update metrics...
  }

  async fn OnTaskEnd(&self, ctx: &TaskHookContext, error: Option<&dyn TaskError>) {
        // ...Decrement the number of running Tasks and update metrics...
  }

  async fn OnTimeout(&self, ctx: &TaskHookContext, duration: Duration) {
        // ...Executes when a TimeoutTaskFrame throws a timeout...
  }

  async fn OnHookAttach<OnTaskStart>(ctx: &TaskHookContext, hook: Option<&dyn TaskHook<OnTaskStart>>) {
        // ...You can initialize logic for when it is attached to a OnTaskStart event...
  }
}

// Defining an arbitrary Task and attaching our hook
#[task(schedule = every!(1s))]
#[hooks(auto(PrometheusMetricsHook::default()))]
async fn MyTask(ctx: &TaskFrameContext) -> Result<(), String> {
    todo!()
}
```

TaskHook Events Include:
- TaskHook attach/detach events
- Task start and end events
- Retries starting and finishing events
- Timeout events
- Dependency resolution status events
- Conditional branching decisions

...

Various patterns can be achieved such as TaskHooks communicating with each other via their own set of events
(Hook-To-Hook Communication), registering internal TaskHooks from TaskFrames or elsewhere, and even running conditional
code based on the presence of a TaskHook.

### Millisecond Calendar-Based Schedules

Fine-grain control over when a Task executes via ``TaskSchedule``. Build your own complex schedules or use pre-existing
ones such as ``TaskScheduleCalendar`` to satisfy your time critical needs:

```rust
#[task(schedule = calendar!{
    year: interval(1), // Can be omitted as well
    month: 1, // Same as using january or exact(1)
    day: 1..=10, // The first 10 days
    hour: +3, // Every 3 hours, same as interval(3)
    minute: jitter(+2, 2), // Jitter with a factor of 2
    second: bounded(0, +10, 20), // Bounded interval
    millisecond: identity // Can be omitted as well
})]
async fn CalendarBasedTask(ctx: &TaskContext) -> Result<(), String> {
  // <...>
}
```

> Temporary Note: Everything up until the calendar proc-macro is considered finished, while the rest is being worked
on and thus needs more time to materialize. These examples most likely won't work until the macro extension is done,
for now simply map the syntax to the base API (the individual structs, traits, enums, etc.). This syntax is purely a vision
and thus may change over time.

### Creating Custom Schedulers

The composition-based architecture of ChronoGrapher also applies to Schedulers!

```rust
#[derive(Scheduler(
    clock!, // Required during construction
    dispatcher = MySchedulerTaskDispatcher,
    engine = MySchedulerEngine,
    store = MySchedulerTaskStore
))]
struct MyCoolScheduler<C: SchedulerConfig>(Scheduler<C>);

// Testing the scheduler with a virtual clock
#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_scheduled_task() {
    let test_scheduler = MyCoolScheduler::<MyCoolSchedulerConfig>::new(
      VirtualClock::default()
    );

    // Fast-forward time to test scheduling
    virtual_clock.advance(Duration::from_hours(24)).await;

    // <...>
  }
}
```

<img align="center" src="assets/Chronographer Divider.png" />
<h1 align="center">Getting Started</h1>
One can install the package via <b>(CURRENTLY NOT AVAILABLE NOW)</b>:

```bash
cargo add chronographer # Rust
pip install chronographer # Python
npm install chronographer # JS/TS (also available in yarn, bun, pnpm...)
```

Just like that. ChronoGrapher is configured for one machine! To scale it more, it is advised to check the
multiple integrations and extensions offered by us or other third-parties

With that said, the next steps are:
- Full Documentation (Coming Soon)
- API Reference (Coming Soon)
- Examples Gallery (Coming Soon)
  <img align="center" src="assets/Chronographer Divider.png" />
<h1 align="center">Contributing And License</h1>

> [!IMPORTANT]  
> The project is in its infancy, it is not out there, as its being worked on and crafted meticulously. **If you plan to
> contribute to the project, Now is the time to provide a good helping hand for the hardworking team**. When it comes to
integrating with other programming languages, we mainly focus on rust then slowly make the features available in other
languages
>
> In addition, the project uses temporary license: **[BSL Business Source License](LICENSE)**, once beta versions roll out,
this is when Chronographer will switch to [MIT License](https://opensource.org/license/mit), in the meantime,
the license in a nutshell says:
> - You can view the source, learn from it, and use it for testing and development.
> - You cannot use this software to run a competing service or product.
> - The license will automatically convert to the [MIT License](https://opensource.org/license/mit) on
    > the date of the first official beta announcement (made by the owner, GitBrincie212)

When it comes to contributing and forking. Chronographer is free and open source to use, only restricted by the lightweight
<strong>MIT License (this license only applies to when the project enters beta)</strong>.
Contributions are welcome with wide open arms as Chronographer is looking to foster a community, proceed to take a look at
[CONTRIBUTING.md](./CONTRIBUTING.md), for more information on how to get started as well as the codebase to learn
from it. We sincerely and deeply are grateful and thankful for the efforts
