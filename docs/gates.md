# Measurable Gates

Status: Draft — Phase 0, Step 4. These are the acceptance gates that Phase 1 (the crawl loop) and later phases must meet. They are measurable, deterministic, and act as the team-wide definition of "done" at each maturity level.

## Gates

### MVP demo

Deterministic crawl of **100 linked local pages**. The crawler must visit exactly the in-scope pages, in a reproducible order, extracting metadata/text/links, with full result persistence.

### Durable MVP

Deterministic crawl of **10,000 local pages**, interrupted at least **three times** during the run, with **correct recovery** — the resumed crawl continues where it stopped and produces the identical final result set as an un-interrupted run. This exercises the durable frontier ("safely resume after interruption").

### Beta scale

Frontier containing **1,000,000 synthetic URLs** with **measured enqueue/dequeue performance**. This validates the frontier data structure at scale, not end-to-end HTTP throughput. Performance numbers are recorded, not promised ahead of time.

### Production candidate

**72-hour controlled soak test** with **worker restarts** and **injected HTTP and database faults**. Demonstrates the crawler survives long-running operation, identifies and recovers from component failure, and keeps the politeness and scope guarantees intact.

### Correctness target

**Zero out-of-scope fetches** and **zero known per-origin politeness violations** in the test harness. These are hard correctness invariants (see `threat-model.md` and `crawl-policy.md`), enforced by a harness that deliberately serves private-IP and out-of-scope URLs and asserts they are never requested.

## Explicit non-goal for now: no pages-per-second promise

We do **not** commit to a throughput number at Phase 0. External websites, actual bandwidth, response sizes, politeness delays, and parsing cost dominate end-to-end throughput — none of which are under our control or measurable from a blank slate.

Approach instead:

1. Build the architecture and the durable frontier.
2. **Benchmark** enqueue/dequeue and local end-to-end crawl performance (gates: Durable MVP, Beta scale) to collect real numbers.
3. Only then, and only if useful, record a target pages-per-second figure for a *specific, stated* environment.

Until step 3, any claims of "X pages/sec" are out of scope and should be treated as speculation.

## Gate ownership

- Each gate maps to at least one automated harness or test suite; a gate is not "passed" based on manual observation alone.
- The correctness target (zero out-of-scope / zero politeness violations) is a regression harness that must stay green from its introduction onward.