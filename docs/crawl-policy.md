# Crawl Policy

Status: Draft — Phase 0, Step 2. Defines the crawler's behavior toward origin servers and its obligations under the product contract.

## User agent

- `User-Agent` header: `crawler-rs/0.1 (+[contact-email])`
- The contact email is configured in the environment (`CRAWLER_CONTACT`) and defaults to the operator-provided value in `.env.example`.
- Every request carries this UA. No UA spoofing, no omission, no per-site negotiation.

## Contact information

- Contact is a single mailbox/mailing list owned by the operator, surfaced in the UA string and on the takedown path below. It is required for receiving robots objections and takedown requests.

## Robots (robots.txt) behavior

robots.txt handling is a **protocol implementation**, specified by [RFC 9309](https://datatracker.ietf.org/doc/html/rfc9309). The crawler implements the protocol, not a vague courtesy layer.

### Parsing and matching

- RFC 9309 group and record parsing: `User-agent`, `Allow`, `Disallow` directives per the RFC grammar.
- Group selection: the group whose `User-agent` line most specifically matches this crawler's UA token (case-insensitive substring match, longest match wins, as the RFC requires).
- Matching: **longest-match rule** against the request URL path. `Allow` and `Disallow` are order-independent; ties broken by directive specificity. `*` and `$` pattern tokens are supported per the RFC.
- A URL not matching any directive is **allowed**.
- A `Disallow:` with an empty value allows everything under that group; `Allow:` with an empty value is ignored per RFC 9309.

### Error handling and caching

- **2xx with content:** parse and honor; cache the parsed result for up to 24 hours.
- **2xx with empty/non-robots body:** treat as allow-all; cache briefly (1 hour).
- **4xx (including 404):** treat as allow-all per RFC 9309; cache 1 hour.
- **5xx / network error / timeout:** treat as **disallow-all (temporarily unavailable)** for the origin; cache the temporary failure no longer than 1 hour, after which re-fetch.
- Malformed robots bodies must not panic the parser; parse failures degrade to the above 4xx-style allow-all with an error log.
- Respect the RFC's requirement to treat `User-agent: *` appropriately for unlisted agents and to ignore unknown directives.

### Crawl-delay extension

`Crawl-delay` is a common **non-standard extension** and is **not honored in v1**. The project's own politeness delay below is authoritative. If `Crawl-delay` is ever enabled, it must be implemented, tested, and versioned deliberately — not silently honored.

## Politeness

- Default per-origin delay: **1,000 ms** between requests to the same host, configurable per job (`CRAWLER_POLITENESS_DELAY_MS`).
- The delay is enforced per host (all pages of a host serialized by the politeness gate), not globally, so distinct origins are not throttled by each other.
- No burst mode: the delay is the floor between requests, including across redirects and retries to the same host.
- Politeness is measured by the test harness: zero known per-origin politeness violations is a Phase 0 exit-gate-adjacent correctness target (see `gates.md`).

## Retry policy

- Maximum **3 attempts** per URL.
- Backoff between retries: 1s, 2s, 4s (exponential).
- Retriable failures: connection errors, timeouts, and 5xx responses.
- **Never retry 4xx** responses or failures classified as protocol/user errors.
- Retries to the same host must respect the politeness delay.
- Redirect handling uses separate limits (`manual` redirects, ≤5 hops, re-scoped each hop — see `threat-model.md`).

## Supported status codes

- `200` (and 2xx) → parse and extract.
- `301 / 302 / 303 / 307 / 308` → follow redirects (≤5 hops, each hop re-scoped to the configured hosts; otherwise dropped and recorded).
- `404 / 410` → record as gone; link recorded but not parsed.
- Other 4xx → record with status, no body persisted, not retried.
- 5xx → record, subject to retry policy above.
- Redirect loops (more than 5 hops) → record, stop, log.

## Takedown process

1. A takedown/robots objection arrives at the contact mailbox, or the operator issues one.
2. The operator adds the offending host to the block/deny list via configuration (`BLOCKED_HOSTS`).
3. Blocked hosts are excluded at enqueue time and at fetch time (defense in depth), and any in-flight job entries for those hosts are re-scoped out.
4. Deletion of already-fetched data for the host follows `data-policy.md` (delete by domain).
5. Target SLA for honoring a takedown: **24 hours** from receipt to blocked host.

## Status-code and scope bookkeeping

All statuses above are recorded per URL in the data store so that post-crawl audit ("what did we fetch, from where, when") is possible without inspecting logs. This is required, not optional, for a polite crawler that carries a UA with contact information.