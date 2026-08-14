# Threat Model

Status: Draft — Phase 0, Step 2. This document treats the crawler as an **SSRF-capable system** and hardens every stage of the fetch pipeline accordingly. Severity: fetch-time inputs are **hostile** — never assume seed URLs, DNS answers, redirects, or bodies are benign.

## Trust boundaries

```
[operator config] ─ trusted
[seed URLs]       ─ semi-trusted (validated at ingestion)
[DNS answers]     ─ hostile
[redirect target] ─ hostile
[response body]   ─ hostile
[robots.txt]      ─ hostile
[storage]         ─ trusted once written, but content derived from hostile input
```

The invariants below must hold regardless of attacker control over any hostile input.

## 1. SSRF — the crawler can reach things it should not

The crawler voluntarily makes outbound HTTP connections from URLs it is told to fetch. An attacker who controls a seed URL, a link on a crawled page, a redirect target, or a DNS answer can otherwise aim the crawler at the operator's own infrastructure.

Mitigations (all enforced in the engine's scope/safety layer, before any connection):

- **Allow-listed scope, validated at ingestion:** each job's hosts are exact host plus explicitly listed subdomains. Anything else is out of scope and never fetched.
- **Scheme allow-list:** `http:` and `https:` only. All other schemes are rejected at ingestion and again at enqueue.
- **Host/DNS validation per resolution:** resolve the hostname, then verify **every** resolved address is public before connecting. Reject if any answer is private — do not just pick the first.
- **Private/reserved address rejection:** loopback (`127.0.0.0/8`, `::1`), RFC 1918 (`10/8`, `172.16/12`, `192.168/16`), link-local (`169.254/16`, `fe80::/10`), ULA (`fc00::/7`), unspecified/broadcast, and cloud metadata (`169.254.169.254`) are always rejected.
- **Pin the resolved address to the request:** the HTTP client must connect to the validated address, not re-resolve (DNS-rebinding defense). Re-validate on redirect hops.
- **Redirect re-scoping:** every redirect hop is re-scoped against the job allow-list *before* following. Out-of-scope redirect → dropped and recorded. Max 5 hops.

## 2. Hostile / rebinding DNS

- Resolve → validate all answers → connect with the validated IP pinned to the request. A resolver that returns different answers between validation and connect is the core DNS-rebinding attack and is neutralized by the pin.
- Time-to-live checks are not a substitute for the pin.

## 3. Redirect attacks

- Redirect loops and out-of-scope redirects are dropped (≤5 hops, re-scoped each hop).
- Redirect responses may carry hostile headers; never forward request headers/cookies across hops.
- A redirect to a non-HTTP scheme is treated as invalid.

## 4. Huge bodies and decompression bombs

- **Hard cap on downloaded bytes:** default 10 MiB (configurable per job). The body is read as a *bounded stream* and truncated/aborted at the cap; it is never buffered unbounded.
- **Decompression-bomb defense:** gorilla/zip-bomb style bodies that decompress far beyond their advertised size are bounded by a *decompressed-size cap* enforced while streaming (e.g. limited reader around the inflate stream). The decompressed cap must be <= a small multiple of the on-wire cap.
- Content-Length is advisory; the streaming cap is authoritative.

## 5. Malformed markup

- HTML parsing is delegated to a spec-compliant, panic-free parser (html5ever lineage). Malformed/truncated/untrusted markup must not panic, hang, or exhaust memory in the parser or extraction layers.
- Bodies that abort mid-stream (cap exceeded) are still parsed defensively as truncated documents or discarded by policy (`data-policy.md`).

## 6. Credential leakage

- **Never** log or persist: headers, `Cookie`, `Authorization`, session state, or request parameters.
- Structured tracing redacts `Authorization`/`Cookie` and error messages that embed URLs with userinfo (`user:pass@host`).
- URLs with embedded userinfo are rejected at ingestion (credential-smuggling / history-poisoning vector).
- Bodies are never logged even at debug level. Stored extracted text is scanned for accidental secrets (env-var style, `-----BEGIN` blocks, cloud keys) and such spans are redacted from storage.
- `.env` and `.env.*` are excluded from the repository (`gitignore`), so operational credentials cannot enter version control.

## 7. Resource exhaustion / availability

- Per-origin politeness delay (see `crawl-policy.md`) doubles as a natural rate limit against hostile hosts.
- Concurrency is bounded (semaphore), bodies are bounded, redirects are bounded, and the URL frontier is deduplicated and normalized to prevent infinite or near-infinite queues (`Infinite URLs` prohibition in `product-contract.md`).

## Future work (not v1)

- Fuzzing harness for URL normalization + robots parser.
- PII scrubbing as a first-class pipeline component (currently manual redaction rules only).
- Content-type sniffing of byte streams before full parse.
- TLS certificate pinning options for operator-provided internal test infrastructure.

## Test assertions this drives

The correctness gate in `gates.md` ("zero out-of-scope fetches") is realized as a test harness that serves private-IP and out-of-scope URLs and asserts they are never requested.