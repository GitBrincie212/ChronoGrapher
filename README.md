<h1 align="center">Chronographer (Scheduling Library)</h1>
<img src="./assets/Chronographer Banner.png" alt="Chronographer Banner" />

> [!IMPORTANT]  
> The project is in its infancy, it is not out there, as its being worked on and crafted meticulously. If you plan to
> contribute to the project, now is the time to provide a good helping hand for the hardworking team. When it comes to
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

Chronographer is the **ULTIMATE** unopinionated scheduling library which all developers dreamed of.
Schedule thousands of tasks with the efficiency of rust while still using it in multiple programming languages
such as <u>Python</u>, <u>Rust</u>, <u>JavaScript(and TypeScript)</u> and even <u>Java</u>. The library is designed to be
as easy to use as possible while being powerful, flexible and extendable

<img align="center" src="assets/Chronographer Divider.png" />
Since Chronographer is a fully featured scheduling library, it provides many features out of the box by default:

## 🧩 Task Composition
Instead of thinking a task is just some executable, Chronographer thinks of tasks as components in a group instead, allowing 
the expression and reuse of complex logic easily while also separating concerns and giving overall flexibility, tasks 
consist of:
  - ***Task Metadata:*** The <ins>State</ins> of the task, anything data-related. Internally, this data is mutable, metadata
  can also be exposed restricting write access to any data, think of it as a global state for the entire task. In most
  cases, this is optional to provide
  <br /> <br />
  - ***Task Frame*** The Task Frame is the core embodiment of a task. It defines <ins>What</ins> needs to be done. Think of it 
  as the immutable recipe or the instruction set for a specific unit of work. Task frames can access the metadata of the
  task, task frames can also be decorated / wrapped, allowing for flexibility with minimal boilerplate footprint and code
  <br /> <br />
  - **Task Schedule** This defines <ins>When</ins> the task needs to be executed, schedules can be as simple as an
  interval, to a cron expression and even a calendar. Given a specific time, they calculate based on the time provided, when
  the task will execute again (they can fail depending on the scenario)
  <br /> <br />
  - **Task Scheduling Strategy (Policy)** Defines when the task reschedules (how the task overlaps with others), 
  should it reschedule the same instance now? Should it do it once the instance finishes execution? Should it cancel
  the previous running task? All these questions are answered by the policy, by default it uses the sequential policy
  <br /> <br />
  - **Task Error Handler** This handles gracefully errors when a task fails, it is meant as a recovery from any potential 
  error, examples include rollbacks, cleanups, state reset management, and so on. While the default error handler used is
  for silently ignoring them, in production environments it is advised to make your own error handler
  <br /> <br />
  - **Task Extension** Acts as an extension point for defining more complex task types, by default it is not required,
   however, in the future distributed crate, this will be used for defining other fields which do not fit in the core
  crate (single-node use)
  <br /> <br />
  - **Task Priority** Defines the importance of a task. Chronographer offers 5 levels of priority which are
  ``LOW``, ``MEDIUM``, ``HIGH``, ``IMPORTANT``, ``CRITICAL``. These priority levels make Chronographer responsive even under
  heavy workflow, as it optimizes the execution of tasks, as low priority tasks may execute a bit later, whereas critical
  tasks in most scenarios will immediately execute
  <br /> <br />
  - **Task Dependencies** While for task frames, you can use ``ParallelTaskFrame`` and ``SequentialTaskFrame`` together,
  there may be cases where you want Task C to wait for Task A and Task B to finish before scheduling it for execution.
    Chronographer has this area covered as well. By default, there are no task dependencies for tasks
  
## 🔄 Task Behavior And Management
Fine grain control over task behavior, listening to lifecycle or local task frame events, controlling how the task
is rescheduled via scheduling strategies, controlling dependencies of a task... etc. Want to dynamically re-schedule, 
remove or schedule tasks at any point throughout your program? Chronographer has you covered

## 📋 Scheduler Composition
Just like tasks. Chronographer gives the ability to also restructure schedulers to fit your needs, no need to depend
on the default scheduler implementation, if you need. You can also implement your own, or even use existing components
defined by the default scheduler, here are the composites a scheduler requires:
- **Clock** This is a mechanism for tracking time, while by default it uses the system clock, one can also use a virtual
clock for simulating scenarios, such as unit testing, benchmarking or stress-testing
<br /> <br />
- **Scheduler Engine** The actual core that drives the Scheduler, it sleeps til the earliest task is due, then
gets the instance from ``TaskStore``, emits relevant events and hands out the task to ``TaskDispatcher``, then repeats
the process for the next task
<br /> <br />
- **Task Store** It stores a task in some form (either be in-memory or persist them), the scheduler may interact with
the task store via getting the earliest task, rescheduling the same task instance or from methods from the scheduler which 
act as wrappers around the task store mechanism
<br /> <br />
- **Task Dispatcher** It is the main logic block that handles execution logic, typically when the scheduler hands out a
task to the dispatcher, it tries to find the worker (which are units that concurrently execute) with the most minimum work 
it has based on priority level, once it finds it, that is where the task's schedule strategy is executed

## 📡 Language Agnostic Communication
Emit a task in python, listen to task events in JavaScript, write task logic in rust. No more doing trickery to
work around the limitation of a library/framework being limited to one specific programming language. Chronographer is
the central hub for scheduling

<img align="center" src="assets/Chronographer Divider.png" />
Why use Chronographer when other scheduling libraries exist in other programming languages? Some of the highlights / strength points
which you might consider to use Chronographer over other scheduling libraries are:

- **🌐 Multi-language Support:** Chronographer is available in Python, Rust, JavaScript/TypeScript, and Java. 
Switch between languages without rewriting scheduling logic and learning a new framework. No more trying to combat the limitations of different 
schedulers
<br /> <br />
- **🛠️ Extensible:** Chronographer's architecture has extensibility in mind, as such you are not restricted to using the
default implementation of the scheduler, task frames, and even schedules. You can build extensions 
for Chronographer in your favourite programming language ecosystem
<br /> <br />
- **↔️ Horizontal Scaling** Chronographer makes it easy and intuitive to scale the scheduling infrastructure horizontally,
across multiple servers located in multiple regions. Chronographer handles multiple timezones and converting them in-between
<br /> <br />
- **🚀 Lightweight & Efficient:** Minimal overhead ensures it won’t bloat your project, while still providing reliable 
timing and execution for thousands of tasks with the power, flexibility and safety of <u>_Rust_</u> under the hood as
its core.
<br /> <br />
- **🔧 Developer-Friendly:** Clear API, intuitive task registration, vast documentation, life shouldn't be harder than
it needs to be. No complications, no trickery, what you write in code is what you will get in the production environment
<br /><br />
- **⏰ Millisecond Precision:** Chronographer is also designed to be millisecond precise, which makes it very practical for
frequent scheduled tasks, it maintains this precision even when clogged by multiple tasks
<br /> <br />
- **📦 Tiny But Mighty** Tired of large sized packages, taking forever to compile, consuming disk space and so on? We too,
as such, Chronographer is tiny about **~1MB** in size
<br /> <br />
<img align="center" src="assets/Chronographer Divider.png" />
When it comes to contributing and forking. Chronographer is free and open source to use, only restricted by the lightweight
**MIT License** <strong>(this license only applies to when the project enters beta)</strong>, contributions
are welcome with wide open arms as Chronographer is looking to foster a community