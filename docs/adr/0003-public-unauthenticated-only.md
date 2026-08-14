# ADR-003: Public unauthenticated crawling only for v1

- **Status:** Accepted
- **Date:** Phase 0
- **Related:** `product-contract.md` (prohibitions), `threat-model.md` (SSRF), `crawl-policy.md` (robots, take-down)

## Context

The most dangerous attack surface of a crawler is not speed but **reach**: a crawler is an SSRF-capable machine. Allowing authenticated or internal crawling multiplies that reach — the crawler would hold credentials, cross auth boundaries, and touch private infrastructure in ways that turn any input bug into a security incident.

The product contract's v1 statement already says **public HTTP/HTTPS pages only**. This ADR makes it an architectural invariant the code must enforce, not just a documentation preference.

## Decision

v1 crawls **only public, unauthenticated HTTP/HTTPS pages**:

- No credentials, no session cookies, no `Authorization` headers, no CAPTCHA-solving — ever passed to fetch.
- No private/loopback/link-local/ULAs/metadata addresses, per `threat-model.md` (SSRF section).
- No non-HTTP schemes (`ftp:`, `file:`, `gopher:`, `data:`, `javascript:`, etc.).
- These are enforced in the scope/safety layer of the engine before any connection is made and re-validated on every redirect hop.

Test infrastructure (local crawls over 127.0.0.1) is allowed as an **explicitly configured development mode** used by the test harness and `crawler-test-site`, separate from production posture, and disabled by default.

## Consequences

**Positive**

- Drastically reduces the SSRF blast radius: the crawler can never be aimed at internal services, the operator's own infrastructure, or credential-protected resources.
- Simplifies policy: no session management, no credential storage, no secure storage subsystem in v1.
- Clean message to site operators interacting via `crawl-policy.md`: public pages only, UA contact available, take-down honored.

**Negative**

- Cannot crawl authenticated portions of sites (intranet, paywalled/login areas) — explicitly out of scope for v1, per `product-contract.md`.
- Development/test crawling of localhost is restricted to the explicit dev mode, which is a small ergonomic cost.

**Tradeoff accepted:** v1 sacrifices authenticated/internal reach for safety. Future authenticated crawling, if ever needed, will be introduced as a separate, hardened, scoped feature — never by relaxing this invariant.