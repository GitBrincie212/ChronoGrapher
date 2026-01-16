# General Module Structure
When making new modules or modifying existing ones, one should be aware of these guidelines:
1. Modules must not exceed **600 lines of code**, otherwise split it into a submodule, this does
   include documentation, imports, submodules... etc. In the mix as well. Exceeding this limit requires 
   a strong cohesion argument and a comment explaining why splitting would harm clarity.
2. Modules group stuff in one category. As such when deciding on a submodule, you should always
   consider what category it belongs in, if it is a new one then make a separate module. If it is
   known one then stick with it (example is ``task.rs``, this gets its own category but ``frames.rs`` is
   below ``task.rs`` because it is related to it)
3. If submodules don't fit elsewhere, then one should consider making a ``misc`` or ``utils`` module for anything that
   is closely shared and re-export this module on the two or more modules that depend on it. Should rarely be used

# ChronoGrapher's Goals And Vision
When writing or rewriting a portion of code in ChronoGrapher, it is best to stay inline with ChronoGrapher's
design philosophy "Minimalism over Bloat, Emergent over Predefined and Simplicity Over Complexity". In addition to its
goals, which are:

### High Performance
ChronoGrapher has to remain intact and operational even under heavy load (millions to billions of Tasks). Due to this,
contributors must optimize their code as much as they can (if their architecture doesn't inline with performance, then
they may have to pivot depending on the circumstances, it is better to discuss it with the GitBrincie212). However,
developers should not micro-optimize (https://wiki.c2.com/?PrematureOptimization) every corner unnecessarily.

Some examples include persistence (where its different compared to the atypical event log based solutions), dependency
management (there is no global DAG in place)... etc. These required architecture shifts

### Extensibility / Flexibility
ChronoGrapher aims to be a highly extensible system. Providing emergent features, i.e. Features that can combine with others
to form complex systems, the features in question are broad, general but adaptable to any sub-niche 
(targeting the general demographic). ChronoGrapher encourages heavy use of traits over enums (enums are rarely used, only for
static things such as the types of time fields, use them when the domain is closed and unlikely to be extended externally). 
ChronoGrapher encourages heavily composition as well as principles of single responsibility per system.

In regard to features, ChronoGrapher follows the "less is more" strategy, minimize any unnecessary features, but also
follows the guideline of keeping features as unopinionated as possible, users should be able to decide themselves how
to use the feature. Features shouldn't be very coupled, depending on the feature in question

Examples of this are **TaskMetadata, TaskErrorHandler and Fragmented Event Listeners**. There were three separate
entire systems which forced <u>Opinionated Design</u> for how to handle state, errors and observability respectively.
These systems also violated the "less is more" principle, in particular the event listeners, they were many of them and
disjointed from one and the other, in addition to the emergency being minimal. This is why ``TaskHooks`` were made,
to address this issue.

### Ergonomics And Simplicity
ChronoGrapher minimizes complexity and thrives on its simplicity model. When a feature is made, one should always have
in mind "minimum path of friction" examples, these examples use the feature in question to accomplish something simple.
If its complex to do one of these examples, this calls for a pivot. Developers should also have in mind the cognitive
load required to decipher the feature, its use in a code piece... etc.

Examples include but not limited to. **Complex TaskFrame Workflows** which had a toll on readability, to solve this,
a ``TaskFrameBuilder`` was introduced. **Boilerplate DynamicTaskFrame Code**, users would have to constantly create the
dynamic taskframe, then return always a ``Ok(())`` and so on, as such a ``dynamic_taskframe`` macro was made for simple
examples.

# Design Tradeoff Priority
- Correctness & Safety
- Scalability & Architectural Performance
- Extensibility / Composability
- Ergonomics
- Local Optimizations

A change that improves a lower-priority axis must not significantly harm a higher-priority one without strong justification.

# Name Conventions
This project does not have any other naming convention requirements apart from the standard rust
conventions listed in this [Guide](https://rust-lang.github.io/api-guidelines/naming.html)

# Before Pushing Changes 
1. make sure to run ``cargo test`` and see if any tests fail (our CI/CD pipeline will also check this)
2. make sure to use ``cargo clippy`` and ensure there is no warning, the exception is loop {}
   statements for testing to keep the script awake at all times (again our CI/CD pipeline will catch this)
3. Group and sort imports consistently. We recommend using rust-analyzer's "Organize Imports" 
   feature to ensure consistency Once no warnings are shown, use ``cargo fmt`` and commit the 
   changes (not a big issue but very recommended still)

# API Documentation
Document (in the API) every public thing without being too brief or too descriptive, explain things mostly in a simple manner
and attach only relevant information that should be documented. The goal is for the API docs to be skim friendly, easily disect
information needed, for this reason **include a brief description on the top about the component**.

There are multiple groups of sections. Each with their own subgroups and best suited for one specific thing, use them wisely
for documentation. Each section is rated out of 5 based on the importance and how recommended it is to include it:

---
This has to do with ChronoGrapher-specific components and systems. These include but are not limited to ``TaskFrames``,
``TaskTriggers``, ``SchedulerEngine``, ``SchedulerTaskStore``, ``PersistenceBackend``... etc. Sections include:

## TaskFrame Sections
- ``# Events`` Describes the events which the ``TaskFrame`` fires, when they are fired and what payload they have. **(5/5)**
- ``# Decorating / Wrapping Behaivior`` Documents how ``TaskFrames`` behave when wrapping inner ``TaskFrame(s)``. **(5/5)**
- ``# Execution Error(s)`` Document what kind of errors may appear whilst the TaskFrame is executing, under what conditions. **(5/5)**
- ``# Supported TaskHook(s)`` Describes what kind of TaskHooks are supported which change the behavior of the TaskFrame in some way. **(4/5)**

## TaskHook Sections
- ``# Supported Event(s) / THEG(s)`` Describes the events and THEGs which the ``TaskHook`` supports. **(5/5)**
- ``# TaskHook Type`` Describes the type of the ``TaskHook``, a marker? An observer? An error handler? A state manager? **(5/5)**
- ``# How To Attach`` Documents how to attach this TaskHook to tasks, does it need manual ``attach_hook`` or are there methods
  that do it automatically? **(4/5)**
- ``# Supported TaskHook(s)`` Describes what kind of TaskHooks are supported which change the behavior of this TaskHook in some way. **(4/5)**

## TaskHook Event Sections (NOT THEGs)
- ``# Payload Type`` Lists the payload type, what it represents, what information it has...etc. **(5/5)**
- ``# Is Emittable`` Explains if the event emittable, if not then why isn't it. **(4/5)**

## THEGs Sections
- ``# Common Payload Type`` Lists the payload type that is present in all ``TaskHookEvents`` which implement this THEG. **(5/5)**
- ``# Is Emittable`` Explains if the events of this THEG are emittable, if not then why isn't it. **(4/5)**
- ``# Supported Events`` Lists the events which are supported by this THEG (the implementors). **(4/5)**

## SchedulingStrategy Sections
- ``# Policy Semantics`` Describes the semantics of the strategy / policy, how rescheduling and overlapping tasks are handled. **(5/5)**
- ``# Supported TaskHook(s)`` Describes what kind of TaskHooks are supported which change the behavior of the ScheduleStrategy in some way. **(4/5)**

## TaskTrigger Sections (NOT TaskSchedules)
- ``# Computation Errors`` Describes the various errors which may appear during computation, this is when alerting. **(5/5)**
- ``# Triggering Errors`` Describes the various errors which may appear during triggering, this is when the trigger is called. **(5/5)**
- ``# Waiting Semantics`` Informs the user what kind of thing (computation, outside event... etc.) does the TaskTrigger wait for. **(5/5)**

## TaskSchedule Sections
- ``# Schedule Errors`` Describes the various errors which may appear during the TaskSchedule computing. **(5/5)**
- ``# Scheduling Semantics`` Informs the user about the TaskSchedule's behavior of computing the timings. **(5/5)**

## FrameDependency Sections
- ``# Resolving & Unresolving`` Describes if the FrameDependency can be resolved and unresolved or not. **(5/5)**
- ``# Dependency Semantics`` Explains the dependency's semantics, what does it wait for. **(5/5)**

## SchedulerEngine Sections
- ``# Supported Config Shape`` Lists the various configuration shapes which the SchedulerEngine supports. **(5/5)**
- ``# Supported TaskHook(s)`` Describes what kind of TaskHooks are supported which change the behavior of the SchedulerEngine in some way. **(4/5)**

## SchedulerClock Sections
- ``# Supported Config Shape`` Lists the various configuration shapes which the SchedulerClock supports. **(5/5)**
- ``# Timing Semantics`` Describes how the SchedulerClock works, what time it represents, how it idles and so on. **(5/5)**
- ``# Is Advancable`` Explains if the SchedulerClock can be manually advanced or not. **(5/5)**

## SchedulerTaskDispatcher Sections
- ``# Supported Config Shape`` Lists the various configuration shapes which the SchedulerTaskDispatcher supports. **(5/5)**
- ``# Dispatching Semantics`` Describes how the SchedulerTaskDispatcher works, how does it exactly dispatch tasks and so on. **(5/5)**

## SchedulerTaskStore Section
- ``# Supported Config Shape`` Lists the various configuration shapes which the SchedulerTaskStore supports. **(5/5)**
- ``# Store Semantics`` Explains how the SchedulerTaskStore works, how does it sort earliest and so on. **(5/5)**
- ``# Supported PersistenceBackends`` Describes ift he SchedulerTaskStore is persistent and what kind of backends it supports. **(5/5)**

## SchedulerConfig Sections
- ``# Component Compatibility`` Which composites work together and if there are any restrictions. **(5/5)**
- ``# Default Implementations`` Lists the default composites if not specified. **(4/5)**


## PersistenceBackend Sections
- ``# Storage Format`` Describes the data format used for persistence (JSON, binary, database... etc.). **(5/5)**
- ``# Recovery Semantics`` Explains how state is recovered after crashes/restarts. **(4/5)**
- ``# Checkpoint Behaivior`` Describes how checkpoints are stored during TaskFrame execution. **(4/5)**

---
This has to do with Rust's general things and patterns, such as consts, traits, builders, structs, enums... etc. Section include:
## Struct Sections
- ``# Method(s)`` Lists and briefly describes the methods (only functions, no getters/setters). **(5/5)**
- ``# Semantics`` Lists how the struct is meant to be used and what it does. **(5/5)**
- ``# Constructor(s)`` Lists the various constructors you can use (including builders, normal rust initialization and via the Default trait). **(5/5)**
- ``# Accessing/Modifying Field(s)`` Lists the fields which you can access and/or modify (via setters/getters or via just normal Rust fields). **(5/5)**
- ``# Trait Implementation(s)`` Lists the various implementations the struct implements (not including blanket implementations). **(5/5)**
- ``# Generic(s)`` Describes the generics which the struct may have (don't include it if there aren't any). **(4/5)**

## Enum Sections
- ``# Variant(s)`` Lists and briefly describes the variants of this enum. **(5/5)**
- ``# Method(s)`` Lists and briefly describes the methods (only functions, no getters/setters). **(5/5)**
- ``# Semantics`` Lists how the enum is meant, what each variant is best suited to and what it does. **(5/5)**
- ``# Constructor(s)`` Lists the various constructors you can use (including builders, normal rust initialization and via the Default trait) for each variant. **(5/5)**
- ``# Accessing/Modifying Field(s)`` Lists the fields which you can access and/or modify (via setters/getters or via just normal Rust fields). **(5/5)**
- ``# Trait Implementation(s)`` Lists the various implementations the enum implements (not including blanket implementations). **(5/5)**
- ``# Generic(s)`` Describes the generics which the enum may have (don't include it if there aren't any). **(4/5)**

## Trait Sections
- ``# Required Method(s)`` Lists the various required methods and what they are meant to do, if any. **(5/5)**
- ``# Required Subtrait(s)`` Lists the various subtraits required for this trait to be implemented, if any. **(5/5)**
- ``# Supertrait(s)`` Lists the various supertraits which require this trait and extend on top of, if any. **(5/5)**
- ``# Semantics`` Lists how the trait is meant to be implemented, used and what it does. **(5/5)**
- ``# Implementation(s)`` Lists the various implementations of this trait and briefly describes them (not blanket implementations), if any. **(5/5)**
- ``# Object Safety / Dynamic Dispatching`` Describes if the trait is object safe / dynamic dispatchable, if not why. **(5/5)**
- ``# Blanket Implementation(s)`` Lists the blanket implementations of this trait, if any. **(4/5)**
- ``# Optional Method(s)`` Lists the various optional methods, what they are meant to do and their default behavior (ideally follow it after required). **(4/5)**
- ``# Generic(s)`` Describes the generics which the trait may have (don't include it if there aren't any). **(4/5)**

## Builder Sections (NOT Structs)
- ``# Required Builder Methods`` Lists the various required builder methods, what they require as arguments and their semantics. **(5/5)**
- ``# Builder Initializer(s)`` Lists the various ways to produce this builder and briefly describes the semantics for each construction method. **(5/5)**
- ``# Optional Builder Methods`` Lists the various optional builder methods, what they require as arguments, their semantics, the defaults. **(4/5)**

## Builder Method Sections (INCLUDED Method Section)
- ``# Default Value`` Informs the user about the default value of the builder value, if there is any. **(5/5)**
- ``# Builder Method Chaining`` Informs the user if multiple identical builder methods are allowed, if yes what do they do, how they modify. **(5/5)**

## Method Sections
- ``# Argument(s)`` Lists the various arguments the method accepts, their type and the semantics for each argument, if any. **(5/5)**
- ``# Returns`` Lists what the method returns back to the developer, the type and the semantics, if these are constant return values describe each return clause, if any. **(5/5)**
- ``# Error(s)`` Lists the various errors which may appear, under what conditions each error appears in. **(5/5)**
- ``# Panics`` Lists the various reasons the method may panic, under what circumstances and why. **(5/5)**
- ``# Semantics`` Lists how the method is meant to be used and what it does. **(5/5)**

---

This has to do with any miscellaneous sections, they are universal and  apply to both ChronoGrapher-specific 
component and general Rust stuff. Sections include:
- ``# Example(s)`` Lists various examples on how to use this system in practice. **(5/5)**
- ``# See Also`` Lists any relevant ``struct``, ``enum``, ``trait``, ``methods``, ``type-alias``, ``constants``, in addition 
  to explaining how they relate briefly. That are either mentioned on the documentation or are recommended to be seen, 
  for using this on methods, list as well the parent ``struct``, ``enum`` or ``trait``. **(5/5)**
- ``# FAQ & Troubleshooting`` Common issues when using this thing as well as answering frequently asked questions **(4/5)**

---

Depending on the circumstance however, trivial methods such as getters / setters which aren't doing much may benefit from
omitting some of the clauses (including some 5/5 clauses). The structure of the API docs should be:
```text
[COMPONENT SUMMARY]
...Should be no more than 6 lines maximum...

[ANY CHRONOGRAPHER SPECIFIC HEADERS]
...List any ChronoGrapher specific headers, try to keep consistent ordering...

[ANY RUST SPECIFIC HEADERS]
...List any Rust specific headers, try to keep consistent ordering...

[Example(s)]
...List at least 1 example and a maximum of 4 examples, while explaining them below...

[FAQ & Troubleshooting]
...List general questions users may ask and/or common mistakes, explaining why they happen and how to fix...

[SEE ALSO]
...List relevant similar components (including traits implemented, what methods belong in, super traits) while briefly explaining them...
```

# Writing Unit Tests
When writing unit tests, one should do tests for regular values and mostly edge case values, when considering edge cases,
it boils down to experience mostly, as such it cannot be described clearly in this document. A couple of edge cases
to be aware are:
- Double or triple calling a method with different values
- Values which are out of bounds
- Ways of turning a conditional to false/true
- Stopping earlier loops than expected
- Race conditions & Concurrency

In terms of writing tests themselves after considering the edge cases, there are specific style guidelines, those being:
1. Create separate modules under ``tests/``, each specializing on testing one component in isolation
2. Create separate modules under ``tests/integrated`` each specialized on testing how one or multiple 
components work together in unison
3. Every edge case, method testing... etc. Should be its own method for testing
4. Follow the [General Module Structure Guidelines](#general-module-structure)
5. Constant values that are used commonly (such as for approximate equality) must be top 
level constants, if these are used throughout other modules then there should be a module
for holding all related constants
6. Any common method used in the testing not part of the core system should live in a ``utils`` module
