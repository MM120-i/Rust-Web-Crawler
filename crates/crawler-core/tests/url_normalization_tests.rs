use crawler_core::url::normalize_for_key;
use url::Url;

struct NormalizeCase {
    name: &'static str,
    input: &'static str,
    expected: Result<&'static str, &'static str>,
}

#[test]
fn table_driven_normalization() {
    let cases = vec![
        // Basic scheme/host lowercasing
        NormalizeCase {
            name: "lowercases HTTP scheme",
            input: "HTTP://EXAMPLE.COM/",
            expected: Ok("http://example.com/"),
        },
        NormalizeCase {
            name: "lowercases host",
            input: "http://EXAMPLE.COM/page",
            expected: Ok("http://example.com/page"),
        },
        // Default port stripping
        NormalizeCase {
            name: "strips :80 from http",
            input: "http://example.com:80/",
            expected: Ok("http://example.com/"),
        },
        NormalizeCase {
            name: "strips :443 from https",
            input: "https://example.com:443/",
            expected: Ok("https://example.com/"),
        },
        NormalizeCase {
            name: "keeps non-default port",
            input: "https://example.com:8443/",
            expected: Ok("https://example.com:8443/"),
        },
        NormalizeCase {
            name: "keeps :8080 on http",
            input: "http://example.com:8080/",
            expected: Ok("http://example.com:8080/"),
        },
        // Fragment stripping
        NormalizeCase {
            name: "strips fragment",
            input: "https://example.com/page#section",
            expected: Ok("https://example.com/page"),
        },
        NormalizeCase {
            name: "strips empty fragment",
            input: "https://example.com/page#",
            expected: Ok("https://example.com/page"),
        },
        // Dot segment resolution
        NormalizeCase {
            name: "resolves single dot",
            input: "https://example.com/a/./b",
            expected: Ok("https://example.com/a/b"),
        },
        NormalizeCase {
            name: "resolves double dot",
            input: "https://example.com/a/../b",
            expected: Ok("https://example.com/b"),
        },
        NormalizeCase {
            name: "resolves multiple dots",
            input: "https://example.com/a/b/../c/../d",
            expected: Ok("https://example.com/a/d"),
        },
        NormalizeCase {
            name: "does not pop past root",
            input: "https://example.com/../a",
            expected: Ok("https://example.com/a"),
        },
        // Query preservation
        NormalizeCase {
            name: "preserves query order",
            input: "https://example.com/?b=2&a=1",
            expected: Ok("https://example.com/?b=2&a=1"),
        },
        NormalizeCase {
            name: "preserves duplicate query keys",
            input: "https://example.com/?x=1&x=2",
            expected: Ok("https://example.com/?x=1&x=2"),
        },
        NormalizeCase {
            name: "preserves empty query",
            input: "https://example.com/?",
            expected: Ok("https://example.com/?"),
        },
        // Empty path
        NormalizeCase {
            name: "empty path becomes /",
            input: "http://example.com",
            expected: Ok("http://example.com/"),
        },
        // Trailing slash
        NormalizeCase {
            name: "preserves trailing slash",
            input: "https://example.com/path/",
            expected: Ok("https://example.com/path/"),
        },
        // Rejection cases
        NormalizeCase {
            name: "rejects userinfo",
            input: "https://user:pass@example.com/",
            expected: Err("user_info"),
        },
        NormalizeCase {
            name: "rejects ftp scheme",
            input: "ftp://example.com/",
            expected: Err("unsupported"),
        },
        NormalizeCase {
            name: "rejects file scheme",
            input: "file:///etc/passwd",
            expected: Err("unsupported"),
        },
        NormalizeCase {
            name: "rejects javascript scheme",
            input: "javascript:alert(1)",
            expected: Err("unsupported"),
        },
        // Unicode host
        NormalizeCase {
            name: "lowercases unicode host",
            input: "http://MÜNCHEN.DE/",
            expected: Ok("http://xn--mnchen-3ya.de/"),
        },
        // Percent encoding preserved
        NormalizeCase {
            name: "preserves percent encoding",
            input: "https://example.com/path%20with%20spaces",
            expected: Ok("https://example.com/path%20with%20spaces"),
        },
    ];

    for case in cases {
        let url = Url::parse(case.input).unwrap_or_else(|_| panic!("parse failed for: {}", case.input));
        let result = normalize_for_key(&url);

        match case.expected {
            Ok(expected_key) => {
                let key = result.unwrap_or_else(|e| panic!("expected Ok for '{}', got Err: {e}", case.name));
                assert_eq!(key.as_str(), expected_key, "failed: {}", case.name);
            }
            Err(_err_type) => {
                assert!(result.is_err(), "expected Err for '{}', got Ok", case.name);
            }
        }
    }
}
