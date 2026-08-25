use crawler_core::Origin;
use url::Url;

#[test]
fn origin_from_url_extracts_host() {
    let url = Url::parse("https://blog.example.com/page/1").unwrap();
    let origin = Origin::from_url(&url).unwrap();
    assert_eq!(origin.host, "blog.example.com");
}

#[test]
fn origin_from_url_lowercases_host() {
    let url = Url::parse("https://EXAMPLE.COM/").unwrap();
    let origin = Origin::from_url(&url).unwrap();
    assert_eq!(origin.host, "example.com");
}

#[test]
fn origin_from_url_strips_trailing_dot() {
    let url = Url::parse("https://example.com./page").unwrap();
    let origin = Origin::from_url(&url).unwrap();
    assert_eq!(origin.host, "example.com");
}

#[test]
fn origin_from_url_works_with_any_scheme() {
    let ftp = Url::parse("ftp://example.com/").unwrap();
    let origin = Origin::from_url(&ftp).unwrap();
    assert_eq!(origin.host, "example.com");
}

#[test]
fn origin_from_url_returns_none_when_url_parse_fails() {
    let result = Url::parse("http://");
    assert!(result.is_err());
}

#[test]
fn origin_equality_ignores_case() {
    let a = Origin::from_url(&Url::parse("https://Example.com/").unwrap()).unwrap();
    let b = Origin::from_url(&Url::parse("https://example.com/").unwrap()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn origin_hash_is_consistent_with_equality() {
    use std::collections::HashMap;

    let a = Origin::from_url(&Url::parse("https://Example.com/").unwrap()).unwrap();
    let b = Origin::from_url(&Url::parse("https://example.com/").unwrap()).unwrap();

    let mut map = HashMap::new();
    map.insert(a, "first");
    map.insert(b, "second");

    assert_eq!(map.len(), 1);
    assert_eq!(map.values().next().unwrap(), &"second");
}
