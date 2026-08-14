# ADR-001: Modular monolith before microservices

- **Status:** Accepted
- **Date:** Phase 0
- **Related:** ADR-002 (PostgreSQL frontier)

## Context

A web crawler is often drawn toward a distributed design (multiple services, a message broker, a fleet of fetchers) because "crawlers scale out." Premature distribution adds operational burden — network boundaries, consistency handling, failing deployments, observability duplication — before the crawl loop even works. This is a single-operator, v1, site indexer (see `product-contract.md`); its immediate challenge is correctness (scope, politeness, resumability), not horizontal scale.

## Decision

Build v1 as a **modular monolith**: one deployable process and one codebase, composed of clearly separated in-code modules (`crawler-core`, `crawler-engine`, `crawler-cli`), communicating via in-process interfaces rather than network APIs. No separate microservices in v1.

The modular boundaries anticipate, but do not require, later extraction into services if measurement shows a single process cannot meet a target.

## Consequences

**Positive**

- One binary to deploy, debug, and reason about; local development is cheap.
- No distributed-consistency problems for the frontier.
- Module boundaries (`core`/`engine`/`cli`) keep testability and ownership clean without service overhead.
- The politeness and scope invariants are enforced in one place and are easy to test end-to-end.

**Negative**

- Scaling is CPU/core-bound to one process; if a future gate requires distributing fetchers, extraction work is needed.
- A single process must coexist with rate limits and politeness delays, so concurrency tuning is within-process.

**Tradeoff accepted:** v1 optimizes for correctness and operability over vertical horizontal scaling, which is untested territory anyway at Phase 0.