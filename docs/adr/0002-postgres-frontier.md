# ADR-002: PostgreSQL frontier before a message broker

- **Status:** Accepted
- **Date:** Phase 0
- **Related:** ADR-001 (modular monolith), `data-policy.md` (durable state)

## Context

The crawler's work queue (the "frontier") is the set of URLs waiting to be fetched. Competing designs are a message broker (Kafka, RabbitMQ, SQS) or a database. The product contract requires **durable, resumable** crawling: "safely resume after interruption" (`gates.md`, Durable MVP). A broker adds an infrastructure dependency, at-least-once / exactly-once ambiguity, and operational tooling — all before the frontier even proves it can store a million URLs.

A database is already justified: the crawler persists metadata, text, and links. Using a single PostgreSQL database for both the frontier and the results removes an entire class of integration problems and makes interrupted-crawl recovery a transactional property rather than a glue layer.

## Decision

Use **PostgreSQL as the frontier** in v1:
- The frontier is a database table (or small set of tables) with durable, transactional enqueue/dequeue.
- Worker claim via `SELECT ... FOR UPDATE SKIP LOCKED` (or equivalent) so concurrent workers claim disjoint rows and a crash mid-claim releases work.
- Deduplication and state (`pending`, `claimed`, `fetched`, `failed`, `out-of-scope`) live in the database, enabling resump-recovery from the last confirmed state.

No message broker in v1.

## Consequences

**Positive**

- One data store for frontier + results → transactional correctness and simpler ops.
- Interruption recovery is exact: the frontier reflects the last durable acknowledgement, not broker offsets.
- Observability inherited from the results store; easy to query "what is pending" for status.
- Fewer moving parts than a broker (ADR-001 alignment).

**Negative**

- DB polling/claims are less "pushy" than a broker; throughput depends on efficient claim queries.
- High-frequency enqueue/dequeue is a DB write path and must be indexed and tuned for the Beta-scale gate (1,000,000 URLs).

**Tradeoff accepted:** throughput tuning is deferred until the Beta-scale gate provides measurements; correctness and durability are prioritized now. Re-evaluate a broker only if a measured gate shows the DB cannot meet a stated target.