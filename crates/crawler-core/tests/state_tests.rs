use std::time::Duration;

use crawler_core::{
    CrawlJobId, DiscoveredLink, FetchErrorKind, FetchId, FetchRequest,
    FetchResponseMetadata, OriginId, RetryDecision, SkipReason, UrlId, UrlState,
};

use url::Url;

#[test]
fn url_state_variants_are_distinct() {
    let states: [UrlState; 5] = [
        UrlState::Pending,
        UrlState::Fetching,
        UrlState::Complete,
        UrlState::Failed,
        UrlState::Skipped,
    ];

    for (i, a) in states.iter().enumerate() {
        for (j, b) in states.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } 
            else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn url_state_is_cloneable() {
    let state: UrlState = UrlState::Pending;
    let cloned: UrlState = state.clone();
    assert_eq!(state, cloned);
}

#[test]
fn skip_reason_variants_are_distinct() {
    let reasons: [SkipReason; 5] = [
        SkipReason::OutOfScope,
        SkipReason::Duplicate,
        SkipReason::MaxDepthReached,
        SkipReason::MaxPagesReached,
        SkipReason::InvalidUrl,
    ];

    for (i, a) in reasons.iter().enumerate() {
        for (j, b) in reasons.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } 
            else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn fetch_error_http_status_preserves_code() {
    let err: FetchErrorKind = FetchErrorKind::HttpError(404);

    match err {
        FetchErrorKind::HttpError(code) => assert_eq!(code, 404),
        _ => panic!("expected HttpError(404)"),
    }
}

#[test]
fn fetch_error_variants_are_distinct() {
    let errors: [FetchErrorKind; 6] = [
        FetchErrorKind::Network,
        FetchErrorKind::Timeout,
        FetchErrorKind::HttpError(500),
        FetchErrorKind::BodyTooLarge,
        FetchErrorKind::InvalidRedirect,
        FetchErrorKind::ParseError,
    ];

    for (i, a) in errors.iter().enumerate() {
        for (j, b) in errors.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } 
            else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn retry_after_preserves_duration() {
    let decision: RetryDecision = RetryDecision::RetryAfter(Duration::from_secs(5));

    match decision {
        RetryDecision::RetryAfter(d) => assert_eq!(d, Duration::from_secs(5)),
        _ => panic!("expected RetryAfter(5s)"),
    }
}

#[test]
fn retry_decision_variants_are_distinct() {
    let decisions = [
        RetryDecision::RetryNow,
        RetryDecision::RetryAfter(Duration::from_secs(1)),
        RetryDecision::DoNotRetry,
    ];
    for (i, a) in decisions.iter().enumerate() {
        for (j, b) in decisions.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn ids_are_copy() {
    let job_id: CrawlJobId = CrawlJobId(1);
    let url_id: UrlId = UrlId(2);
    let origin_id: OriginId = OriginId(3);
    let fetch_id: FetchId = FetchId(4);
    let job_id2: CrawlJobId = job_id;
    let url_id2: UrlId = url_id;
    let origin_id2: OriginId = origin_id;
    let fetch_id2: FetchId = fetch_id;

    assert_eq!(job_id, job_id2);
    assert_eq!(url_id, url_id2);
    assert_eq!(origin_id, origin_id2);
    assert_eq!(fetch_id, fetch_id2);
}

#[test]
fn different_id_types_are_not_interchangeable() {
    let url_id: UrlId = UrlId(1);
    let origin_id: OriginId = OriginId(1);

    assert_ne!(format!("{:?}", url_id), format!("{:?}", origin_id));
}

#[test]
fn fetch_request_stores_all_fields() {
    let req: FetchRequest = FetchRequest {
        url: Url::parse("https://example.com/page").unwrap(),
        url_id: UrlId(42),
        origin_id: OriginId(7),
        depth: 3,
        attempt: 2,
    };

    assert_eq!(req.url.as_str(), "https://example.com/page");
    assert_eq!(req.url_id, UrlId(42));
    assert_eq!(req.origin_id, OriginId(7));
    assert_eq!(req.depth, 3);
    assert_eq!(req.attempt, 2);
}

#[test]
fn fetch_response_metadata_stores_all_fields() {
    let meta: FetchResponseMetadata = FetchResponseMetadata {
        status: 200,
        headers: vec![("content-type".into(), "text/html".into())],
        duration: Duration::from_millis(150),
        bytes_download: 4096,
    };

    assert_eq!(meta.status, 200);
    assert_eq!(meta.headers.len(), 1);
    assert_eq!(meta.duration, Duration::from_millis(150));
    assert_eq!(meta.bytes_download, 4096);
}

#[test]
fn discovered_link_stores_all_fields() {
    let link: DiscoveredLink = DiscoveredLink {
        source_url_id: UrlId(1),
        target_url: Url::parse("https://example.com/other").unwrap(),
        anchor_text: "click here".into(),
    };
    
    assert_eq!(link.source_url_id, UrlId(1));
    assert_eq!(link.target_url.as_str(), "https://example.com/other");
    assert_eq!(link.anchor_text, "click here");
}
