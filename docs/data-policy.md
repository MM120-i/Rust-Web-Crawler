# Data Policy

Status: Draft — Phase 0, Step 2. What the crawler retains, for how long, and how it can be destroyed.

## What is retained

Per job, the crawler persists the four data classes below. Retention is scoped to the job that produced it and the job's operator.

| Class | Contents | Stored by default |
|---|---|---|
| Metadata | URL, canonical URL, HTTP status, Content-Type, size, fetch timestamp, ETag/checksum | Yes |
| Extracted text | Cleaned body text (markup removed) | Yes |
| Links | Source → target edges with anchor text and target scope classification | Yes |
| Raw bodies | The raw HTTP response body (HTML) | **No — opt-in per job** |

- **Raw bodies are disabled by default.** They are enabled only by an explicit per-job flag.
- Content types outside `text/html` are not extracted and their bodies are not stored unless raw-body mode is on, subject to size caps.
- No cookies, no `Authorization`/API credentials, no headers, and no request parameters are ever persisted or logged. Credential hygiene is enforced in `threat-model.md`.

## Default retention

- **No automatic expiry in v1.** Data lives until the operator deletes it; the crawler never silences data silently. Auto-TTL is a deliberate future feature, not a default.

## Deletion controls

- **Delete by job:** removes all data (metadata, text, links, raw bodies) created by that job, including enqueued work already fetched and recovered from interruption.
- **Delete by domain:** removes all data associated with a host across all jobs. This is the primary takedown/withdrawal control (see `crawl-policy.md`).
- Deletion is cascade and complete: removing a URL removes its metadata, text, raw body, and link edges in both directions.
- All deletion operations are deterministic, restartable on error, and logged. There is no grace window; deletion is immediate.

## Raw bodies: opt-in flag

- Per-job flag `store_raw_bodies` (default `false`).
- When `true`, raw bodies are subject to the same retention/deletion rules as everything else (delete by job, delete by domain).
- Raw bodies are capped by the same maximum body size as the fetch pipeline (see `threat-model.md`).

## Sizing caps

- Maximum single-body size for storage: equal to the fetch cap (default 10 MiB, configurable). Bodies over the cap are truncated/recorded as oversized and not stored.
- These caps prevent unbounded downloads, which the product contract explicitly prohibits.

## Privacy and re-use

- Data is a derived corpus for the crawling project. Not to be re-served publicly or resold beyond the operator's stated purpose.
- Credentials encountered in crawled content (e.g. secrets pasted in HTML) are in-scope for the threat model's leakage section and must be detected/redacted from stored text, not propagated.

## Out of scope (v1)

- Backup/restore tooling and long-term archive tiering.
- Multi-tenant data isolation (single-operator deployment).
- Automatic anonymization or PII scrubbing pipelines (note in threat model as future work).