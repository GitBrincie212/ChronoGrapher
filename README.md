<h1 align="center">Chronographer (Scheduling & Orchestration Library)</h1>
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

<img align="center" src="assets/Chronographer Divider.png" />
<h1 align="center">What Is ChronoGrapher?</h1>

Dreaming of a **powerful, unopinionated polyglot scheduler**? ChronoGrapher delivers, achieve 
**Rust-Level Efficiency** while scheduling thousands of tasks across all your projects. 
With native bindings for Python, Rust, JavaScript/TypeScript, and Java, 
it combines raw power with incredible ergonomics and flexibility via a composition-based architecture

<img align="center" src="assets/Chronographer Divider.png" />
<h1 align="center">The Architecture Of ChronoGrapher</h1>
Since Chronographer is a fully featured scheduling / orchestration workflow library, it provides many 
features out of the box by default:

## 🧩 Task Composition
Instead of thinking a task is just some executable. Chronographer thinks of tasks as components in a group instead, allowing 
the expression and reuse of complex logic easily while also separating concerns and giving overall flexibility, tasks 
consist of the core components:
  - ***Task Frame:*** The Task Frame is the core embodiment of a task. It defines <ins>What</ins> needs to be done. Think of it 
  as the immutable recipe or the instruction set for a specific unit of work. Task frames can access the metadata of the
  task, task frames can also be decorated / wrapped, allowing for flexibility with minimal boilerplate footprint and code
  <br /> <br />
  - **Task Schedule:** This defines <ins>When</ins> the task needs to be executed, schedules can be as simple as an
  interval, to a cron expression and even a calendar. Given a specific time, they calculate based on the time provided, when
  the task will execute again (they can fail depending on the scenario)
  <br /> <br />
  - **Task Scheduling Strategy:** Defines when the task reschedules (how the task overlaps with others), 
  should it reschedule the same instance now? Should it do it once the instance finishes execution? Should it cancel
  the previous running task? All these questions are answered by the policy, by default it uses the sequential policy
  <br /> <br />
  - **Task Priority:** Defines the importance of a task. ChronoGrapher offers 5 levels of priority which are
  ``LOW``, ``MEDIUM``, ``HIGH``, ``IMPORTANT``, ``CRITICAL``. These priority levels make Chronographer responsive even under
  heavy workflow, as it optimizes the execution of tasks, as low priority tasks may execute a bit later, whereas critical
  tasks in most scenarios will immediately execute
  <br /> <br />
  - **TaskHook System:** If the core components are not enough. ChronoGrapher includes an **Extremely Powerful** system
  called ``TaskHooks``, these have the ability to observe various events and react to them, hold state and even interact
  with other task hooks. The real power comes from task hooks **being independent of the task's business logic** and
  can act as an optional enhancement as opposed to a strict requirement, in addition to this, ``TaskFrames`` can also
  attach onto their respective Task, their **OWN** TaskHooks, get task hook instance and detach TaskHooks as they please

That is it! with these 4 core components (plus priority, tho mostly serves as metadata) of Task, you can shape
your own tasks, no more boilerplate, no more complexity and no more barricading yourself in unscalable & unmaintainable 
systems. Your imagination is truly the only barrier

## 📋 Scheduler Composition
Just like tasks. Chronographer gives the ability to also restructure schedulers to fit your needs, no need to depend
on the default scheduler implementation, if you need. You can also implement your own, or even use existing components
defined by the default scheduler, here are the composites a scheduler requires:
- **Clock:** This is a mechanism for tracking time, while by default it uses the system clock, one can also use a virtual
clock for simulating scenarios, such as unit testing, benchmarking or stress-testing
<br /> <br />
- **Task Store:** It stores a task in some form (either be in-memory or persist them), the scheduler may interact with
the task store via getting the earliest task, rescheduling the same task instance or from methods from the scheduler which 
act as wrappers around the task store mechanism
<br /> <br />
- **Task Dispatcher:** It is the main logic block that handles execution logic, typically when the scheduler hands out a
task to the dispatcher, it tries to find the worker (which are units that concurrently execute) with the most minimum work 
it has based on priority level, once it finds it, that is where the task's schedule strategy is executed

## 📡 Language Agnostic Communication
Emit a task in python, listen to task events in JavaScript, write task logic in rust. No more doing trickery to
work around the limitation of a library/framework being trapped in one specific programming language, one specific
ecosystem. Chronographer is the central hub for scheduling, no more glue code, no more anything that harms productivity.
Ensuring the smoothest developer experience

<img align="center" src="assets/Chronographer Divider.png" />
<h1 align="center">Why Should I Use ChronoGrapher?</h1>
Why use Chronographer when other scheduling libraries exist in other programming languages? Some of the highlights
/ strength points which you might consider to use Chronographer over other scheduling libraries are:
<br /> <br />

1. **🌐 Multi-language Support:** Chronographer is available in Python, Rust, JavaScript/TypeScript, and Java. 
Switch between languages without rewriting scheduling logic and learning a new framework. No more trying to combat the limitations of different 
schedulers, one universal scheduler library to rule them all
<br /> <br />
2. **🛠️ Immeasurable Extensible:** Chronographer's architecture has extensibility in mind, as such you are not restricted to 
using the default implementation of the scheduler, task frames, and even schedules. You can build extensions 
for Chronographer in your favourite programming language ecosystem, you can write your own TaskHooks... etc.
<br /> <br />
3. **↔️ Horizontal Scaling** Chronographer makes it easy and intuitive to scale the scheduling infrastructure horizontally,
across multiple servers located in multiple regions. Chronographer handles multiple timezones and converting them in-between,
while also supporting Kubernetes and other widely used tools in distributed systems 
<br /> <br />
4. **🚀 Performance To The Moon:** The core as well as the core extensions are severely optimized to handle multiple
tasks concurrently. Some optimizations include tickless scheduling, persisting state lazily, keeping an in-memory copy
of the current program's state, no **D**ependency **A**cyclic **G**raph overhead for tasks that don't need it, and so on so fourth
<br /> <br />
5. **💾 Undeterred Durability:** ChronoGrapher offers near deterministic durability. The way the state is managed, the way
the execution state is kept and the persistence system as a whole makes it possible to store on disk the entire state
of the scheduler and restore it after a shutdown as if the program simply paused
<br /> <br />
6. **🔧 Developer-Friendly:** Clear API, intuitive task registration, vast documentation. The design of ChronoGrapher is 
simplicity and minimalism above all, life shouldn't be harder than it needs to be. No complications, no trickery, what you 
write in code is what you will get in the production environment
<br /><br />
7. **⏰ Second & Millisecond Precision:** Chronographer is also designed to be not only second but also millisecond precise,
which makes it ideal for scheduled tasks that frequently execute, it attempts to maintain this precision even when clogged 
by multiple tasks (tho no guarantees of fetching exactly at the specified millisecond under heavy workload)
<br /> <br />
8. **📦 Tiny But Mighty** Tired of large sized packages, taking forever to compile, consuming disk space and so on? We too,
as such, Chronographer is tiny about **~1MB** in size
<br /> <br />
<img align="center" src="assets/Chronographer Divider.png" />
<h1 align="center">Contributing & License Of ChronoGrapher?</h1>

When it comes to contributing and forking. Chronographer is free and open source to use, only restricted by the lightweight
<strong>MIT License (this license only applies to when the project enters beta)</strong>. 
Contributions are welcome with wide open arms as Chronographer is looking to foster a community, proceed to take a look at 
[CONTRIBUTING.md](./CONTRIBUTING.md), for more information on how to get started as well as the codebase to learn
from it. We sincerely and deeply are grateful and thankful for the efforts
