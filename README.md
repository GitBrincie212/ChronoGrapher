<h1 align="center">ChronoGrapher</h1>
<img src="./assets/Chronographer Banner.png" alt="Chronographer Banner" />

<img align="center" src="assets/Chronographer Divider.png" />
<h1 align="center">One Job Scheduler To Rule Them All</h1>

ChronoGrapher is a **Job Scheduler And Workflow Orchestration Platform** that brings unified scheduling to your entire stack.

1. **Unified Multi-Language API:** Coordinate workflows across Python, TypeScript/JavaScript, Rust, and Java with a single, 
beautiful API, no more glue code needed.

2. **Unopinionated by Design:** ChronoGrapher provides core scheduling without forcing features. Major capabilities 
offered as optional extensions.

3. **Hyper-Extensible Architecture:** Built for customization with numerous integrations and extension points.

4. **Scale Effortlessly:** Rust-powered engine handles from single machines to distributed clusters seamlessly.

5. **Crash-Resistant Durability:** Never lose task progress again—state persistence ensures continuity through failures.

**Get started in 30 seconds**, here's a simple "Hello World" example in ChronoGrapher written in Rust 
(other languages look similar):
```rust
use chronographer::prelude::*;

#[tokio::main]
async fn main() {
  let task = Task::define(
    TaskScheduleInterval::from_secs(4),
    |_ctx| async {
      println!("Hello World");
      Ok(())
    }
  );

  CHRONOGRAPHER_SCHEDULER.schedule_owned(task).await;
  loop {} // (Optional) keeps alive the program
}
```

<img align="center" src="assets/Chronographer Divider.png" />
<h1 align="center">Solving Modern Scheduling Challenges</h1>
Today's applications are inherently polyglot, yet scheduling solutions remain language-bound,
ChronoGrapher rethinks scheduling for modern distributed systems and general use. 

A typical modern stack leverages multiple programming languages. Each requiring its own scheduling ecosystem:

- **Python:** Apache Airflow, Celery, Prefect, APScheduler
- **TypeScript/Javascript** Agenda, Bree, BullMQ, Bottleneck
- **Rust:** cron_tab, tokio_task_scheduler, tokio-cron
- **Java:** Quartz, Spring Scheduler
- **Misc:** Temporal, CRON

**The Current Challenge:**
Most solutions face fundamental limitations:
- **Language Isolation:** Bound to single ecosystems, requiring complex glue code for cross-language workflows
- **Scalability Constraints:** Difficult to extend beyond initial design parameters without significant re-engineering
- **Inconsistent Developer Experience:** Varying documentation quality, opinionated patterns, and steep learning curves across tools

**The ChronoGrapher Approach:**
We believe developers deserve better than fragmented scheduling experiences. While no solution is perfect, 
ChronoGrapher's polyglot architecture, performance-first design, and extensibility focus represent significant 
breakthroughs in scheduler design, eliminating the need to master multiple disjointed systems.
<img align="center" src="assets/Chronographer Divider.png" />
<h1 align="center">Core Capabilities / Key Features</h1>
There are many features that make up ChronoGrapher. We will focus on a few key features such as:

### Composable Task Architecture

ChronoGrapher's power comes from its modular task system. Build complex workflows by composing simple, reusable components:

```rust
let payment_process = TaskFrameBuilder::builder(handle_payment)
    .with_timeout(Duration::from_secs(30)) // It cannot exceed 30 seconds
    .with_fallback(handle_payment_failure) // If it does or fails, execute this
    .with_instant_retry(NonZeroU32::new(3).unwrap()) // If all fail, retry 3 times
    .with_dependencies(vec![
        LogicalDependency::or(
          system_notif_dependency,
          cleanup_notif_dependency
        ),
        data_extraction_task_dependency,
        validation_task_dependency,
        // ...
    ]) // Run the workflow only if dependencies are resolved
    .build();

let payment_task = Task::builder()
    .frame(payment_process)
    .schedule(TaskScheduleInterval::from_secs(3))
    .schedule_strategy(CancelCurrentSchedulingPolicy)
    .build();
```

Some of the available task frame types are:

- **🔄 RetriableFrame:** Automatic retries for a TaskFrame with configurable backoff strategies
- **⏱️ TimeoutFrame:** Enforce execution time limits on a TaskFrame (otherwise a timeout error is thrown if exceeded)
- **🔄 FallbackFrame:** If the primary TaskFrame fails, switch to a secondary TaskFrame
- **🎯 ConditionalFrame:** Conditional execution of a TaskFrame via an outside predicate
- **📋 SequentialFrame:** Executes multiple TaskFrames sequentially
- **⚡ ParallelFrame:** Executes multiple TaskFrames in parallel
- **🔗 DependencyFrame:** Executes a TaskFrame if its dependencies are resolved (can depend on other Tasks)
- **💤 DelayFrame:** Delays the execution of a TaskFrame

### Powerful Hook-Based System
Monitor tasks at a deep level by reacting to relevant events emitted:

```rust
// A basic example for integration with Prometheus
struct PrometheusMetricsHook;

/*
    Defines what kind of event the PrometheusMetricsHook TaskHook implementation supports,
    if you don't care which event the TaskHook is being used and the code is the same,
    you can do:
    
    impl<E: TaskHookEvent> TaskHook<E> for PrometheusMetricsHook {
        ...
    }
*/

impl TaskHook<OnTaskStart> for PrometheusMetricsHook {
  async fn on_event(&self, event: OnTaskStart, ctx: Arc<TaskContext>, payload: &OnTaskStart::Payload) {
    metrics::counter!("tasks_started_total", "task_id" => payload.task_id.to_string()).increment(1);
    metrics::gauge!("running_tasks").increment(1.0);
  }
}

impl TaskHook<OnTaskEnd> for PrometheusMetricsHook {
  async fn on_event(&self, event: OnTaskEnd, ctx: Arc<TaskContext>, payload: &OnTaskEnd::Payload) {
    metrics::gauge!("running_tasks").decrement(1.0);

    if payload.error.is_some() {
      metrics::counter!("tasks_failed_total", "task_id" => payload.task_id.to_string()).increment(1);
    } else {
      metrics::counter!("tasks_completed_total", "task_id" => payload.task_id.to_string()).increment(1);
    }
  }
}

// Attach to any task
task.attach_hook(Arc::new(PrometheusMetricsHook)).await;
```
TaskHook Events Include:
- TaskHook attach/detach events
- Task start and end events 
- Retries starting and finishing events
- Timeout events
- Dependency resolution status events
- Conditional branching decisions

### Millisecond Calendar-Based Schedules
Finite control over how a Task executes via a ``TaskScheduleCalendar``

```rust
let schedule = TaskScheduleCalendar::builder()
    .millisecond(Arc::new(TaskCalendarFieldExact::new(0))) // At millisecond 0
    .second(Arc::new(TaskCalendarFieldInterval::new(30)))  // Every 30 seconds
    .minute(Arc::new(TaskCalendarFieldExact::new(0)))      // At minute 0
    .hour(Arc::new(TaskCalendarFieldRange::new(9..=17)))   // Business hours only
    .build();
```

### Priority-Aware Execution
Ensure critical tasks get resources when needed:
```rust
let critical_task = Task::builder()
    .frame(alert_system)
    .priority(Priority::CRITICAL)  // Jumps to front of queue
    .schedule(...)
    .build();
```

### Creating Custom Schedulers
The composition-based architecture of ChronoGrapher also applies to Schedulers!
```rust
struct MyCoolScheduler(Scheduler);

impl MyCoolScheduler {
  pub fn new(clock: impl SchedulerClock) -> Self {
    MyCoolScheduler(
      Scheduler::builder()
              .store(...)
              .clock(clock)
              .dispatcher(...)
              .build()
    )
  }
}

// Testing the scheduler with a virtual clock
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_scheduled_task() {
        let virtual_clock = Arc::new(VirtualClock::new());
        let test_scheduler = MyCoolScheduler::new(virtual_clock.clone());
        
        // Fast-forward time to test scheduling
        virtual_clock.advance(Duration::from_hours(24)).await;
        
        // ...
    }
}
```

<img align="center" src="assets/Chronographer Divider.png" />
<h1 align="center">Getting Started</h1>
One can install the package via:

```bash
cargo add chronographer # Rust
pip install chronographer # Python
npm install chronographer # JS/TS (also available in yarn, bun, pnpm...)
```

Then ChronoGrapher is configured for one machine!
To scale it more, it is advised to check the multiple integrations 
and extensions offered by us or other third-parties

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
