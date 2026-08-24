mod common;

use url::Url;

use crate::common::{test_config, test_config_with_paths};

#[test]
fn scope_allows_exact_host() {
    let config: crawler_core::CrawlConfig = test_config(vec!["example.com"]);
    assert!(config.is_in_scope(&Url::parse("https://example.com/").unwrap()));
}

#[test]
fn scope_allows_http_and_https() {
    let config: crawler_core::CrawlConfig = test_config(vec!["example.com"]);
    assert!(config.is_in_scope(&Url::parse("http://example.com/").unwrap()));
    assert!(config.is_in_scope(&Url::parse("https://example.com/").unwrap()));
}

#[test]
fn scope_rejects_unknown_host() {
    let config: crawler_core::CrawlConfig = test_config(vec!["example.com"]);
    assert!(!config.is_in_scope(&Url::parse("https://evil.com/").unwrap()));
}

#[test]
fn scope_is_case_insensitive_for_host() {
    let config: crawler_core::CrawlConfig = test_config(vec!["example.com"]);
    assert!(config.is_in_scope(&Url::parse("https://EXAMPLE.COM/").unwrap()));
}

#[test]
fn scope_rejects_non_http_schemes() {
    let config: crawler_core::CrawlConfig = test_config(vec!["example.com"]);
    assert!(!config.is_in_scope(&Url::parse("ftp://example.com/").unwrap()));
    assert!(!config.is_in_scope(&Url::parse("file:///etc/passwd").unwrap()));
}

#[test]
fn scope_allows_subdomain_when_listed() {
    let config: crawler_core::CrawlConfig = test_config(vec!["example.com", "blog.example.com"]);
    assert!(config.is_in_scope(&Url::parse("https://blog.example.com/").unwrap()));
}

#[test]
fn scope_rejects_unlisted_subdomain() {
    let config: crawler_core::CrawlConfig = test_config(vec!["example.com"]);
    assert!(!config.is_in_scope(&Url::parse("https://blog.example.com/").unwrap()));
}

#[test]
fn scope_rejects_deep_subdomain_when_only_parent_listed() {
    let config: crawler_core::CrawlConfig = test_config(vec!["example.com"]);
    assert!(!config.is_in_scope(&Url::parse("https://a.b.example.com/").unwrap()));
}

#[test]
fn scope_respects_path_prefix() {
    let config: crawler_core::CrawlConfig = test_config_with_paths(vec!["example.com"], vec!["/docs"]);
    assert!(config.is_in_scope(&Url::parse("https://example.com/docs").unwrap()));
    assert!(config.is_in_scope(&Url::parse("https://example.com/docs/intro").unwrap()));
}

#[test]
fn scope_rejects_path_not_matching_prefix() {
    let config: crawler_core::CrawlConfig = test_config_with_paths(vec!["example.com"], vec!["/docs"]);
    assert!(!config.is_in_scope(&Url::parse("https://example.com/blog").unwrap()));
}

#[test]
fn scope_no_path_prefix_allows_all_paths() {
    let config: crawler_core::CrawlConfig = test_config(vec!["example.com"]);
    assert!(config.is_in_scope(&Url::parse("https://example.com/anything/at/all").unwrap()));
}

#[test]
fn scope_rejects_url_with_no_host() {
    let _config: crawler_core::CrawlConfig = test_config(vec!["example.com"]);
    assert!(Url::parse("http://").is_err());
}
