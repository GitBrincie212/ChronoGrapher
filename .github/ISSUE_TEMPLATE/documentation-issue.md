---
name: Documentation Issue
about: Something that has to do with the current documentation of the project
title: "[DOCS] My Title For Documentation Issue"
labels: 'Guidebook Documentation'
assignees: ''

---

> IMPORTANT: Do NOT remove headers (unless marked optional) and follow rigorously the structure outlined. Additionally,
if it is related to API docs, append the "API Documentation" label whereas if it is related to guidebook docs then 
"Guidebook Documentation" (if both, then use both). **(REMOVE ME)**

## High-Level Overview
Briefly describe what area of the documentation suffers from that problem, what's wrong about it and how to fix it.
This should act as a summary of the entire issue to get an idea of what to do, the main talking points will be below.
The summary should be short around ~3 lines MAX but clearly describe intent, this message has the ideal length.

## Documentation Problem
Where does the problem resides in documentation, there are two main systems:
- **API Documentation:** Basically the docstrings of various components (functions, structs... etc.). Focuses on how to
use that specific component in code, various errors it can produce, constructors if it is a struct / enum... etc. Doesn't 
describe overlapping systems unless necessary (and even then, briefly) and isn't meant to be narrative.

- **Guidebook Documentation:** Resides in ChronoGrapher's website and explains things narratively and progressively, focusing
on patterns and building knowledge on how to use this system with other systems to do something else. Doesn't describe any
deep internal details but only architecture, patterns and systems.

Additionally explain what kind of problem is it, for example if the documentation poorly explains a topic or doesn't cover 
the entire area, a typo? Or even if the topic's documentation is outdated / incorrect with what currently is in the codebase?

## The Solution (OPTIONAL)
Explains how it could be solved, if it is too basic such as a typo then omit it entirely, it is recommended to suggest
how you would rewrite part of the documentation to best explain that topic.