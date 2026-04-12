---
name: Bug Report
about: A report for unintended effects
title: "[BUG] My Title For A Bug Report "
labels: 'Unintended / Bug'
assignees: ''

---

> IMPORTANT: Do NOT remove headers (unless marked optional) and follow rigorously the structure outlined. **(REMOVE ME)**

## High-Level Overview
Provide a concise description of the issue, state what is wrong and where it occurs. This should be 2–4 lines MAX
and allow someone to immediately understand the problem scope.

## Expected Behavior
Describe what *SHOULD* happen under normal conditions. Be precise and unambiguous, list an expected output that demonstrates
the correct behavior under normal conditions.

## Actual Behavior
Describe what *ACTUALLY* happened. Include any incorrect outputs, any panic messages, any potential deadlocks / freezes... etc.
That can help narrow down where the bug resides in.

## Steps To Reproduce
Provide a deterministic sequence of steps to reproduce the exact bug / unintended issue in a bullet-styled format as shown
in the example below:
1. **Create XYZ:** Describe briefly create XYZ in this step.
2. **Change XYZ's Parameters:** Explain what parameters and how to change them for XYZ and in what order.
3. **Destroy XYZ:** Elaborate on what "destroy" would mean (calling drop? A method?)... etc.

It is highly recommend if possible to include one or more minimal reproducible code snippet and specific configuration used.

## The Environment
Provide relevant information on what environment the issue was caused:
- **OS:** The OS the program ran in
- **Rust Version:** The Rust version used
- **ChronoGrapher Version / Commit:** The ChronoGrapher version / commit used (provide a link ideally)
- **Enabled Cargo Features:** Which cargo features were enabled when running
- **Additional Hardware (OPTIONAL):** Any additional hardware that may aid in narrowing down the bug such as the CPU model, RAM size... etc.
  <...>

## Bug Impact
Explain the severity and scope of this bug:
- Does it affect correctness, performance, memory or something else?
- Is it deterministic or does it randomly occur every once in a while?
- Are there any workarounds / hacks to this issue?

## Suspected Cause (OPTIONAL)
If you have any insight, describe what might be causing this bug, reference specific components and where in their code
this bug appears in, to help narrow further the area.

## Additional Context (OPTIONAL)
Add any logs (or debug prints but specify where they are), screenshots (or videos demonstrating in action), benchmarks, or
external references that help with diagnosing this bug.