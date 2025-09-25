<h1 align="center">ChronoGrapher Core</h1>
<img src="./assets/Chronographer Core Banner.png" alt="Chronographer Core Banner" />

---
This is the **core of the library**, the foundation of **<u>ChronoGrapher</u>** resides here, it contains essential scheduling
and orchestration systems. The core is deliberately kept lean and minimal, as such it does not include the full powers of ChronoGrapher 
but rather it is used throughout official extensions for ChronoGrapher. This separation of core and extensions allows:
1. **🗂 Modularization**️ The entire library is split into multiple parts making it easier to maintain
2. **📏 Fine-Grain Feature Control** The user can choose what integrations to use, what features to use... etc.,
3. **💪 Clear Showcase Of Power** It demonstrates how effective the core is and the library as a whole

The core's philosophy is to stay minimal but meaningful:
- Only implement **common, worthwhile, and universally useful** features that **every workload benefits from**.
- Stay **Rust-First** and **Single-Node** focused on the core, ensuring maximum performance and idiomatic design.
- For multi-language SDKs, the core is wrapped and exposed in a way that **feels natural to each language**.
- For distributed scheduling, a similar wrapping strategy is applied, where the distributed crate layers 
on top of the same minimal foundation.

In short, the core acts as the spine of ChronoGrapher: simple, performant, extensible and flexible. 
While leaving room for more advanced feature-sets to grow around the core.