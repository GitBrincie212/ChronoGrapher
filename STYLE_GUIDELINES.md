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
When writing or rewriting a portion of code in ChronoGrapher, it is best to stay inline with ChronoGrapher's goals which are:

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

# Documentation
Document every public method without being too brief or too descriptive, explain things mostly in a simple manner
and attach only relevant information that should be documented, there are multiple groups of sections, each suited for
one specific thing. Use the special sections wisely for documentation:

## Struct Documentation Sections
This has to do with documentation on a ``struct`` specifically (**NOT** its methods):
- ``# Implementation Detail(s)``  In case it is needed to explain the way the struct is implemented. This should be
  used sparingly and shouldn't be the main focus **(OPTIONAL)**
- ``# Struct Field(s)`` It is used to describe some of the non-internal fields (non-internal fields can count
if they are accessed via accessor methods, not just have the public visibility flag) **(MUST)**
- ``# Trait Implementation(s)`` Lists important noteworthy traits that have implemented this struct **(HIGH RECOMMEND)**
- ``# Constructor(s)`` Lists relevant constructor methods and explains briefly their purpose, this does include
  typical construction from a struct (without a method) **(MUST)**
- ``# Cloning Semantics`` Explains what clone does if it isn't obvious enough **(OPTIONAL)**

## Builder Parameter Documentation Sections
This has to do with documentation on builder pattern, <u>specifically the builder methods themselves</u>:
- ``# Default Value`` Describes what is the default value of the builder parameter, if it has one **(HIGH RECOMMEND)**
- ``# Validation Rules`` Describes the constraints the builder parameter has, ranges, non-zero... etc. As well as
  what happens if these validation rules fail for a value **(HIGH RECOMMEND)**
- ``# Method Behaivior`` Describes how the builder parameter method works under the hood as well as if it can
  be chained multiple times **(HIGH RECOMMEND)**

## Enum Documentation Sections
This has to do with documentation on ``enum`` specifically (**NOT** its methods):
- ``# Variants`` Demonstrates the various variants this enum has, what they are, how do they function, what they
    represent and so on so fourth **(MUST)**
- ``# Implementation Detail(s)``  In case it is needed to explain the way the struct is implemented. This should be
  used sparingly and shouldn't be the main focus **(OPTIONAL)**
- ``# Trait Implementation(s)`` Lists important noteworthy traits that have implemented this struct **(HIGH RECOMMEND)**
- ``# Constructor(s)`` Lists relevant constructor methods and explains briefly their purpose, this does include
typical construction from an enum (without a method) **(MUST)**
- ``# Cloning Semantics`` Explains what clone does if it isn't obvious enough **(OPTIONAL)**

## Methods Documentation Sections
This has to do with the documentation on methods (present in either ``struct/trait/enum``):
- ``# Panics`` if a method can panic with an error, document how it can panic and optionally how it could be prevented **(MUST)**
- ``# Error(s)`` if a method can return one or more error(s), it should be documented on what conditions it causes a
method to fail and optionally how it could be prevented **(MUST)**
- ``# Safety`` if a method is unsafe, explain why a function is unsafe and the 
invariants callers must uphold **(MUST)**
- ``# Argument(s)`` if the method has argument(s), they should be explained what they are for **(MUST)**
- ``# Returns`` if the method has a returned value, it should be explained what does it actually return **(MUST)**
- ``# Performance`` for documenting how performant or slow the method is, ideally there should be recommendations
for how to avoid the performance drop off. Suggest whenever other methods or patterns are more performant than this implementation 
as well as explaining why its slow **(OPTIONAL)**

## Trait Documentation Sections
This has to do with the documentation on ``trait``:
- ``# Trait Implementation(s)`` Lists important noteworthy ``structs/enums`` that have implemented this trait **(MUST)**
- ``# Required Method(s)`` Lists various method(s) which are required to be implemented by the developer, also briefly 
describes what these methods do **(MUST)**
- ``# Supertrait(s)`` List any super traits that must be implemented when using this trait **(MUST)**
- ``# Object Safety`` State if the trait is object safe, why it is (or why not) **(MUST)**
- ``# Extension Trait(s)`` List specific noteworthy extension traits that base off this trait, what do they 
add onto the existing trait **(HIGH RECOMMEND)**

## Miscellaneous Documentation Sections
This has to do with any miscellaneous sections that fit either in many of them or don't fit, these can be used
anywhere unlike the other documentation sections which are restricted:
- ``# Example`` For example(s) section, list relevant simple examples, it must be used for any complex systems,
  One should always start from simplest to more complex **(HIGH RECOMMEND)**
- ``# See Also`` Lists any relevant ``struct``, ``enum``, ``trait``, ``methods``, ``type-alias``, ``constants``... etc.
That are either mentioned on the documentation or are recommended to be seen, for using this on methods, list as well the
parent ``struct``, ``enum`` or ``trait`` **(MUST)**
- ``# Usage Note(s)`` General guide to when and how to use this ``struct``, ``enum``, ``trait``, ``method``, 
explains also common pitfalls **(OPTIONAL)**

---

Depending on the circumstance however, trivial methods such as getters / setters which aren't doing much may benefit from
omitting some of the clauses (including some MUST clauses)

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
