use crawler_core::CrawlJobId;
use crawler_core::url::AdmittedUrl;
use crawler_engine::{EnqueueResult, Frontier};
use url::Url;

fn make_admitted(path: &str, depth: u32) -> AdmittedUrl {
    let url: Url = Url::parse(&format!("https://example.com{path}")).unwrap();
    let crawl_key: crawler_core::CrawlKey = crawler_core::url::normalize_for_key(&url).unwrap();
    
    AdmittedUrl {
        crawl_key,
        url,
        depth,
        source_url_id: None,
    }
}

#[test]
fn enqueue_returns_admitted_for_new_url() {
    let mut frontier: Frontier = Frontier::new(CrawlJobId(1), 10);
    let admitted: AdmittedUrl = make_admitted("/page1", 0);
    let result: EnqueueResult = frontier.enqueue(admitted);
    assert!(matches!(result, EnqueueResult::Admitted { .. }));
}

#[test]
fn enqueue_returns_duplicate_for_same_url() {
    let mut frontier: Frontier = Frontier::new(CrawlJobId(1), 10);
    let a: AdmittedUrl = make_admitted("/page1", 0);
    let b: AdmittedUrl = make_admitted("/page1", 1);
    frontier.enqueue(a);
    let result: EnqueueResult = frontier.enqueue(b);
    assert!(matches!(result, EnqueueResult::Duplicate));
}

#[test]
fn enqueue_returns_at_capacity_when_max_pages_reached() {
    let mut frontier = Frontier::new(CrawlJobId(1), 2);
    frontier.enqueue(make_admitted("/a", 0));
    frontier.enqueue(make_admitted("/b", 0));
    let result: EnqueueResult = frontier.enqueue(make_admitted("/c", 0));
    assert!(matches!(result, EnqueueResult::AtCapacity));
}

#[test]
fn dequeue_returns_url_in_bfs_order() {
    let mut frontier: Frontier = Frontier::new(CrawlJobId(1), 10);
    frontier.enqueue(make_admitted("/first", 0));
    frontier.enqueue(make_admitted("/second", 0));
    let (_, first) = frontier.dequeue().unwrap();
    assert_eq!(first.url.path(), "/first");
    let (_, second) = frontier.dequeue().unwrap();
    assert_eq!(second.url.path(), "/second");
}

#[test]
fn dequeue_returns_none_when_empty() {
    let mut frontier: Frontier = Frontier::new(CrawlJobId(1), 10);
    assert!(frontier.dequeue().is_none());
}

#[test]
fn mark_complete_removes_from_in_flight() {
    let mut frontier: Frontier = Frontier::new(CrawlJobId(1), 10);
    let admitted: AdmittedUrl = make_admitted("/page1", 0);
    let key: crawler_core::CrawlKey = admitted.crawl_key.clone();

    frontier.enqueue(admitted);
    frontier.dequeue();
    assert_eq!(frontier.stats().in_flight_count, 1);

    frontier.mark_complete(&key);
    assert_eq!(frontier.stats().in_flight_count, 0);
    assert_eq!(frontier.stats().completed_count, 1);
}

#[test]
fn mark_failed_removes_from_in_flight() {
    let mut frontier: Frontier = Frontier::new(CrawlJobId(1), 10);
    let admitted: AdmittedUrl = make_admitted("/page1", 0);
    let key: crawler_core::CrawlKey = admitted.crawl_key.clone();

    frontier.enqueue(admitted);
    frontier.dequeue();

    frontier.mark_failed(&key);
    assert_eq!(frontier.stats().in_flight_count, 0);
    assert_eq!(frontier.stats().failed_count, 1);
}

#[test]
fn stats_tracks_counts() {
    let mut frontier: Frontier = Frontier::new(CrawlJobId(1), 10);
    let a: AdmittedUrl = make_admitted("/a", 0);
    let b: AdmittedUrl = make_admitted("/b", 0);
    let key_a: crawler_core::CrawlKey = a.crawl_key.clone();

    frontier.enqueue(a);
    frontier.enqueue(b);

    frontier.dequeue();
    frontier.dequeue();

    frontier.mark_complete(&key_a);

    let stats = frontier.stats();
    assert_eq!(stats.seen_count, 2);
    assert_eq!(stats.queued_count, 0);
    assert_eq!(stats.in_flight_count, 1);
    assert_eq!(stats.completed_count, 1);
    assert_eq!(stats.total_enqueued, 2);
}

#[test]
fn duplicate_detection_persists_after_complete() {
    let mut frontier: Frontier = Frontier::new(CrawlJobId(1), 10);
    let admitted: AdmittedUrl = make_admitted("/page1", 0);
    let key: crawler_core::CrawlKey = admitted.crawl_key.clone();

    frontier.enqueue(admitted);
    frontier.dequeue();
    frontier.mark_complete(&key);

    let result: EnqueueResult = frontier.enqueue(make_admitted("/page1", 0));
    assert!(matches!(result, EnqueueResult::Duplicate));
}

#[test]
fn different_urls_are_not_duplicates() {
    let mut frontier: Frontier = Frontier::new(CrawlJobId(1), 10);
    frontier.enqueue(make_admitted("/a", 0));
    let result: EnqueueResult = frontier.enqueue(make_admitted("/b", 0));
    assert!(matches!(result, EnqueueResult::Admitted { .. }));
}

#[test]
fn empty_frontier_is_empty() {
    let frontier: Frontier = Frontier::new(CrawlJobId(1), 10);
    assert!(frontier.is_empty());
    assert_eq!(frontier.pages_fetched(), 0);
}
