use crawler_core::url::{AdmissionError, normalize_for_key, resolve_dot_segments};
use url::Url;

#[test]
fn crawl_key_strips_fragment() {
    let url: Url = Url::parse("https://example.com/page#section").unwrap();
    let key: crawler_core::CrawlKey = normalize_for_key(&url).unwrap();
    assert_eq!(key.as_str(), "https://example.com/page");
}

#[test]
fn crawl_key_lowercases_scheme_and_host() {
    let url: Url = Url::parse("HTTPS://EXAMPLE.COM/").unwrap();
    let key: crawler_core::CrawlKey = normalize_for_key(&url).unwrap();
    assert_eq!(key.as_str(), "https://example.com/");
}

#[test]
fn crawl_key_strips_default_port() {
    let url: Url = Url::parse("https://example.com:443/").unwrap();
    let key: crawler_core::CrawlKey = normalize_for_key(&url).unwrap();
    assert_eq!(key.as_str(), "https://example.com/");
}

#[test]
fn crawl_key_keeps_non_default_port() {
    let url: Url = Url::parse("https://example.com:8443/").unwrap();
    let key: crawler_core::CrawlKey = normalize_for_key(&url).unwrap();
    assert_eq!(key.as_str(), "https://example.com:8443/");
}

#[test]
fn crawl_key_resolves_dot_segments() {
    let url: Url = Url::parse("https://example.com/a/../b").unwrap();
    let key: crawler_core::CrawlKey = normalize_for_key(&url).unwrap();
    assert_eq!(key.as_str(), "https://example.com/b");
}

#[test]
fn crawl_key_preserves_query_order() {
    let url: Url = Url::parse("https://example.com/?b=2&a=1").unwrap();
    let key: crawler_core::CrawlKey = normalize_for_key(&url).unwrap();
    assert_eq!(key.as_str(), "https://example.com/?b=2&a=1");
}

#[test]
fn crawl_key_preserves_duplicate_query_keys() {
    let url: Url = Url::parse("https://example.com/?b=1&b=2").unwrap();
    let key: crawler_core::CrawlKey = normalize_for_key(&url).unwrap();
    assert_eq!(key.as_str(), "https://example.com/?b=1&b=2");
}

#[test]
fn crawl_key_rejects_userinfo() {
    let url: Url = Url::parse("https://user:pass@example.com/").unwrap();
    assert!(normalize_for_key(&url).is_err());
}

#[test]
fn crawl_key_rejects_non_http() {
    let url: Url = Url::parse("ftp://example.com/").unwrap();
    assert!(matches!(
        normalize_for_key(&url),
        Err(AdmissionError::UnsupportedScheme(_))
    ));
}

#[test]
fn resolve_dot_segments_basic() {
    assert_eq!(resolve_dot_segments("/a/b/c"), "/a/b/c");
    assert_eq!(resolve_dot_segments("/a/../b"), "/b");
    assert_eq!(resolve_dot_segments("/a/./b"), "/a/b");
    assert_eq!(resolve_dot_segments("/a/b/../c"), "/a/c");
}

#[test]
fn resolve_dot_segments_does_not_pop_past_root() {
    assert_eq!(resolve_dot_segments("/../a"), "/a");
    assert_eq!(resolve_dot_segments("/a/../../b"), "/b");
}

#[test]
fn resolve_dot_segments_preserves_trailing_slash() {
    assert_eq!(resolve_dot_segments("/a/b/"), "/a/b/");
    assert_eq!(resolve_dot_segments("/a/../b/"), "/b/");
}
