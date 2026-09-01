use crawler_core::{CrawlConfig, SkipReason};
use crawler_fetch::{FetchOutcome, Fetcher};
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_config(allowed_host: &str) -> CrawlConfig {
    CrawlConfig {
        seeds: vec![],
        allowed_hosts: vec![allowed_host.to_string()],
        allowed_path_prefixes: vec![],
        max_pages: 100,
        max_depth: 3,
        global_concurrency: 5,
        per_origin_delay: Duration::from_millis(0),
        request_timeout: Duration::from_secs(5),
        connect_timeout: Duration::from_secs(5),
        max_body_bytes: 10 * 1024 * 1024,
        user_agent: "crawler-fetch-test/0.1".to_string(),
    }
}

fn server_host(server: &MockServer) -> String {
    url::Url::parse(&server.uri())
        .unwrap()
        .host_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn fetch_returns_full_body_under_limit() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/page"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(b"hello world".to_vec(), "text/html"))
        .mount(&server)
        .await;

    let config = test_config(&server_host(&server));
    let fetcher = Fetcher::new(&config).unwrap();
    let url = url::Url::parse(&format!("{}/page", server.uri())).unwrap();

    let outcome = fetcher.fetch(url, 1024).await.unwrap();

    match outcome {
        FetchOutcome::Html { body, truncated } => {
            assert_eq!(body, b"hello world");
            assert!(!truncated);
        }
        FetchOutcome::Skipped(reason) => panic!("expected Html, got Skipped({reason:?})"),
    }
}

#[tokio::test]
async fn fetch_skips_non_html_content_type() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/image.png"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/png")
                .set_body_bytes(vec![0u8, 1, 2, 3]),
        )
        .mount(&server)
        .await;

    let config = test_config(&server_host(&server));
    let fetcher = Fetcher::new(&config).unwrap();
    let url = url::Url::parse(&format!("{}/image.png", server.uri())).unwrap();

    let outcome = fetcher.fetch(url, 1024).await.unwrap();

    assert!(matches!(
        outcome,
        FetchOutcome::Skipped(SkipReason::UnsupportedMediaType)
    ));
}

#[tokio::test]
async fn fetch_truncates_body_at_byte_limit() {
    let server = MockServer::start().await;
    let full_body = "0123456789abcdefghij"; // 20 bytes
    Mock::given(method("GET"))
        .and(path("/big"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(full_body.as_bytes().to_vec(), "text/html"))
        .mount(&server)
        .await;

    let config = test_config(&server_host(&server));
    let fetcher = Fetcher::new(&config).unwrap();
    let url = url::Url::parse(&format!("{}/big", server.uri())).unwrap();

    let outcome = fetcher.fetch(url, 10).await.unwrap();

    match outcome {
        FetchOutcome::Html { body, truncated } => {
            assert_eq!(body, full_body.as_bytes()[..10]);
            assert!(truncated);
        }
        FetchOutcome::Skipped(reason) => panic!("expected Html, got Skipped({reason:?})"),
    }
}

#[tokio::test]
async fn fetch_does_not_follow_redirect_out_of_scope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/redirect"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("location", "http://out-of-scope.invalid/page")
                .set_body_raw(b"stub-body".to_vec(), "text/html"),
        )
        .mount(&server)
        .await;

    let config = test_config(&server_host(&server));
    let fetcher = Fetcher::new(&config).unwrap();
    let url = url::Url::parse(&format!("{}/redirect", server.uri())).unwrap();

    // if the redirect policy incorrectly followed this, reqwest would try to
    // connect to a host that doesn't exist and this would return Err instead.
    let outcome = fetcher.fetch(url, 1024).await.unwrap();

    match outcome {
        FetchOutcome::Html { body, .. } => assert_eq!(body, b"stub-body"),
        FetchOutcome::Skipped(reason) => panic!("expected Html, got Skipped({reason:?})"),
    }
}

#[tokio::test]
async fn fetch_follows_redirect_within_scope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/redirect"))
        .respond_with(ResponseTemplate::new(302).insert_header("location", "/page"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/page"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(b"target-body".to_vec(), "text/html"))
        .mount(&server)
        .await;

    let config = test_config(&server_host(&server));
    let fetcher = Fetcher::new(&config).unwrap();
    let url = url::Url::parse(&format!("{}/redirect", server.uri())).unwrap();

    let outcome = fetcher.fetch(url, 1024).await.unwrap();

    match outcome {
        FetchOutcome::Html { body, .. } => assert_eq!(body, b"target-body"),
        FetchOutcome::Skipped(reason) => panic!("expected Html, got Skipped({reason:?})"),
    }
}
