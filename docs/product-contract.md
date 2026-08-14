# Product Contract (v1)

Status: Draft — Phase 0, Step 1. This is the single source of truth for what the crawler is for.

## Initial v1 product statement

> Given public HTTP/HTTPS seed URLs and an explicit host/path scope, crawl allowed static HTML pages politely, extract metadata/text/links, persist results, and safely resume after interruption.

## Is the initial product a site indexer, change monitor, search corpus builder, archive, or general crawl platform?

**Site indexer.** v1 is a scoped, polite crawler that visits allowed static HTML pages under an explicit host/path scope, extracts metadata and text, records links, and persists structured results. Change monitoring, a search corpus, and archival are explicitly deferred: they place different requirements on storage, diffing, and rendering that v1 does not own. A general crawl platform (multi-tenant, arbitrary inputs, distributed workers) is the eventual shape but explicitly not v1.

## Who creates jobs?

**Operators, via a CLI.** A single operator defines a job: one or more seed URLs plus an explicit host/path scope. There is no SaaS, no self-service portal, no multi-tenant API in v1. Job creation is a local, auditable action.

## Are only public, unauthenticated pages allowed initially?

**Yes.** v1 crawls only public, unauthenticated HTTP/HTTPS pages. No logins, no session cookies, no credential-based access, no CAPTCHA solving. If a page requires authentication, it is out of scope and is recorded as not-fetched.

## Is v1 restricted to exact hosts and explicit subdomains?

**Yes.** v1 only follows links whose target stays within the configured exact host and its explicitly listed subdomains. Any link leaving this scope is recorded as out-of-scope and never fetched. Wildcard "crawl the whole internet" behavior is prohibited. Path prefixes narrow the scope further when configured.

## What is stored: metadata, extracted text, links, raw HTML, or all four?

Consistent with the v1 statement, v1 stores:

- **Metadata** — URL, canonical URL, HTTP status, `Content-Type`, page size, fetch timestamp, check-sum/ETag.
- **Extracted text** — body text cleaned of markup.
- **Links** — source → target edges with anchor text and target scope classification.

**Raw HTML is off by default.** It can be enabled per job via an explicit flag, subject to the retention and size limits in `data-policy.md`. Storing raw bodies by default is a storage-cost and privacy risk that v1 avoids.

## What retention and deletion controls are required?

- **Retention:** v1 does not auto-expire data. Nothing is deleted silently.
- **Deletion by job:** deleting a job removes all data created by that job (metadata, text, links, and any raw bodies).
- **Deletion by domain:** deleting a domain/host removes all data associated with that host across jobs.
- Destruction is operator-initiated via documented CLI/storage commands, never implicit.

## What is explicitly prohibited?

- **Login bypasses** — no credentials, sessions, or cookie injection.
- **CAPTCHA bypasses** — no solver integration, no evasion of access controls.
- **Private networks** — no loopback, link-local, private (RFC 1918), ULA, or cloud metadata addresses (e.g. `169.254.169.254`) at fetch time. DNS answers are validated before any connection is made.
- **Non-HTTP protocols** — HTTP and HTTPS only. `ftp:`, `file:`, `gopher:`, `javascript:`, `data:`, and every other scheme are rejected.
- **Infinite URLs** — URL normalization (protocol-relative, default ports, fragments, host casing, trailing slash) plus a per-URL and per-request budget to prevent infinite or near-duplicate loops.
- **Unbounded downloads** — hard caps on request body size and on decompressed size (decompression-bomb defense). See `threat-model.md`.

## Non-goals (v1)

- Change monitoring and diffing.
- Search indexing/corpus assembly.
- Archival / full-page rendering.
- JavaScript rendering (only static HTML).
- Multi-tenant web UI / SaaS.
- Distributed workers across machines.
- Non-HTML content types (PDFs, images, JSON APIs).

Scope rules and these prohibitions carry directly into `crawl-policy.md` and `threat-model.md`. The crawl loop must not start until these boundaries are reflected in the engine's scope and safety types.