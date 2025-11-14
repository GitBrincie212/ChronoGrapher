# The Future Of ChronoGrapher
The vision for ChronoGrapher is to make it a fully featured platform of extension libraries (directly supported by us,
while also containing some within the community) built on top of a core library. We have various ambitious
plans, which can be summarized in this checklist (from highest to lowest priority):
- [ ] **Core Library** - A minimal base library (foundation) that provides core functionalities and abstractions
- [ ] **Distributed Systems Library** - An extension to the core, making it fully ready for distributed systems 
- [ ] **Web Dashboard** - Extends both core and distributed sys lib, adding a website for visual feedback
- [ ] SDKs
  - [ ] Python
  - [ ] Java
  - [ ] PHP
  - [ ] Rust
  - [ ] JavaScript/TypeScript
- [ ] Integrations
  - [ ] RocksDB (default, used in core for performance)
  - [ ] Redis
  - [ ] Apache Kafka (probably will be used by default)
  - [ ] RabbitMQ
  - [ ] Apache Cassandra
  - [ ] PostgreSQL
  - [ ] MySQL
  - [ ] SQLite
  - [ ] MongoDB
  - [ ] Amazon DynamoDB
  - [ ] Grafana
  - [ ] Sentry
  - [ ] Datadog
  - [ ] Celery
- [ ] Website
  - [ ] Documentation
    - [ ] Getting Started
    - [ ] General Concepts
    - [ ] Advanced Concepts
    - [ ] Common Patterns And Practices
    - [ ] List Of Integrations / Extensions (with docs about)
  - [ ] Landing Page
  - [ ] Info Page
  - [ ] Integrations / Extensions Page

## Distributed Systems Library
The goal of this library is simple, bridge the gap between the in-process core and enterprise-level 
distributed systems used in various applications. The features of the library are as follows:
- Provide distributed task dispatching and fault-tolerant scheduling
- Support leader election and task coordination (e.g., via Kafka or Redis)
- Offer hooks for custom load-balancing, horizontal scaling... etc.
- Maintain core architectural principles (composition, modularity, and low coupling)

## Web Dashboard Library
The goal for this library is to be able to start up a server with the ability to preview information regarding the
current process of scheduling in a modern easy to read interface, with visualizations and metrics. The features of the
library are as follows:
- Visualization of TaskFrame structure
- Metrics regarding which tasks started, finished... etc.
- Configuration over the dashboard's features
- (And other features not considered as of now)

## Future Considerations
- [ ] **Plugin / Extension Marketplace** A dedicated section to ChronoGrapher's website for community plugins

*This roadmap will evolve as ChronoGrapher grows. Community suggestions and contributions are always welcome.*