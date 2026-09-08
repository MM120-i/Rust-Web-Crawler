//! testing for crawl command

use crawler_cli::cli::CrawlArgs;
use crawler_cli::crawl;

#[tokio::test]
async fn crawls_the_full_fixture_graph_end_to_end() {
    let site = crawler_test_site::spawn().await;
    let seed = site.base_url.join("root").unwrap();

    let args = CrawlArgs {
        // setup
        url: seed.to_string(),
        max_pages: 100,
        max_depth: 5,
    };

    // run the actual crawl!!!!!
    let summary = crawl::run_crawl(args)
        .await
        .expect("crawl should complete without erroring");

    // basic santity, like max pages respected,
    assert!(summary.attempted <= 100);
    // every page fetched fell under one of the buckets, either success, skipped, failed or attempted
    assert_eq!(
        summary.successful + summary.skipped + summary.failed,
        summary.attempted
    );
    // crawler saw link out of scope, and refused to fetch
    assert!(summary.out_of_scope_rejected >= 1);
    // crawler saw dupe on root page, and didnt go to crawl it twice
    assert!(summary.duplicate_links_rejected >= 1);
    // all the little edge cases like redirect and oversized or malformed were handled if we get here

    println!("{summary:?}");
}

// potential test case for later: nothing verifies that each url was fetched exactly once (only by crawlers seen logic)
// can add counting requests later and test if we want
