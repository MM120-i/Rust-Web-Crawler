use crawler_core::url::{admit, AdmissionError};
use crawler_core::{CrawlConfig, UrlId};
use std::time::Duration;
use url::Url;

fn config() -> CrawlConfig {
    CrawlConfig {
        seeds: vec![],
        allowed_hosts: vec!["example.com".to_string(), "other.org".to_string()],
        allowed_path_prefixes: vec![],
        max_pages: 100,
        max_depth: 3,
        global_concurrency: 5,
        per_origin_delay: Duration::from_secs(1),
        request_timeout: Duration::from_secs(10),
        max_body_bytes: 1024 * 1024,
        user_agent: "test-crawler/0.1".to_string(),
    }
}

struct AdmissionCase {
    name: &'static str,
    url: &'static str,
    depth: u32,
    source: Option<u64>,
    expect_ok: bool,
    expect_error_variant: Option<&'static str>,
}

#[test]
fn table_driven_admission() {
    let cfg = config();
    let cases = vec![
        AdmissionCase {
            name: "admits in-scope http URL at depth 0",
            url: "https://example.com/",
            depth: 0,
            source: None,
            expect_ok: true,
            expect_error_variant: None,
        },
        AdmissionCase {
            name: "admits in-scope URL at max depth",
            url: "https://example.com/",
            depth: 3,
            source: None,
            expect_ok: true,
            expect_error_variant: None,
        },
        AdmissionCase {
            name: "rejects URL exceeding max depth",
            url: "https://example.com/",
            depth: 4,
            source: None,
            expect_ok: false,
            expect_error_variant: Some("max_depth"),
        },
        AdmissionCase {
            name: "rejects out-of-scope host",
            url: "https://evil.com/",
            depth: 0,
            source: None,
            expect_ok: false,
            expect_error_variant: Some("scope"),
        },
        AdmissionCase {
            name: "rejects ftp scheme",
            url: "ftp://example.com/",
            depth: 0,
            source: None,
            expect_ok: false,
            expect_error_variant: Some("scheme"),
        },
        AdmissionCase {
            name: "rejects javascript scheme",
            url: "javascript:alert(1)",
            depth: 0,
            source: None,
            expect_ok: false,
            expect_error_variant: Some("scheme"),
        },
        AdmissionCase {
            name: "rejects userinfo",
            url: "https://user:pass@example.com/",
            depth: 0,
            source: None,
            expect_ok: false,
            expect_error_variant: Some("userinfo"),
        },
        AdmissionCase {
            name: "admits second allowed host",
            url: "https://other.org/page",
            depth: 1,
            source: Some(1),
            expect_ok: true,
            expect_error_variant: None,
        },
        AdmissionCase {
            name: "preserves source_url_id",
            url: "https://example.com/page",
            depth: 1,
            source: Some(42),
            expect_ok: true,
            expect_error_variant: None,
        },
        AdmissionCase {
            name: "admits at depth 0 with source",
            url: "https://example.com/",
            depth: 0,
            source: Some(1),
            expect_ok: true,
            expect_error_variant: None,
        },
    ];

    for case in cases {
        let url = Url::parse(case.url)
            .unwrap_or_else(|e| panic!("parse failed for '{}' ({}): {e}", case.name, case.url));
        let source = case.source.map(UrlId);
        let result = admit(&cfg, &url, case.depth, source);

        if case.expect_ok {
            let admitted = result.unwrap_or_else(|e| panic!("expected Ok for '{}', got Err: {e}", case.name));
            assert_eq!(admitted.depth, case.depth, "depth mismatch: {}", case.name);
            assert_eq!(admitted.source_url_id, source, "source mismatch: {}", case.name);
        } else {
            let err = result.unwrap_err();
            match case.expect_error_variant.unwrap() {
                "max_depth" => assert!(
                    matches!(err, AdmissionError::MaxDepthExceeded { .. }),
                    "expected MaxDepthExceeded for '{}', got: {err}",
                    case.name
                ),
                "scope" => assert!(
                    matches!(err, AdmissionError::OutOfScope),
                    "expected OutOfScope for '{}', got: {err}",
                    case.name
                ),
                "scheme" => assert!(
                    matches!(err, AdmissionError::UnsupportedScheme(_)),
                    "expected UnsupportedScheme for '{}', got: {err}",
                    case.name
                ),
                "userinfo" => assert!(
                    matches!(err, AdmissionError::UserinfoRejected),
                    "expected UserinfoRejected for '{}', got: {err}",
                    case.name
                ),
                _ => panic!("unknown error variant"),
            }
        }
    }
}

#[test]
fn admission_produces_correct_crawl_key() {
    let cfg = config();
    let url = Url::parse("https://EXAMPLE.COM:443/page?q=1#frag").unwrap();
    let admitted = admit(&cfg, &url, 0, None).unwrap();
    assert_eq!(admitted.crawl_key.as_str(), "https://example.com/page?q=1");
}
