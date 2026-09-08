//! crawler that puts everything (all the different crates) together into one

use std::time::{Duration, Instant};

use crawler_core::url::AdmissionError;
use crawler_core::{CrawlConfig, CrawlJobId};
use crawler_engine::{EnqueueResult, Frontier};
use crawler_fetch::{FetchOutcome, Fetcher};

use crate::cli::CrawlArgs;

#[derive(Debug, Default)]
pub struct CrawlSummary {
    pub unique_urls_discovered: usize,
    pub attempted: usize,
    pub successful: usize,
    pub skipped: usize,
    pub failed: usize,
    pub total_bytes: usize,
    pub duplicate_links_rejected: usize,
    pub out_of_scope_rejected: usize,
    pub elapsed: Duration,
}

/// Builds the crawl's settings from the CLI args and seed URL.
fn build_config(
    args: &CrawlArgs,
    seed: &url::Url,
) -> Result<CrawlConfig, Box<dyn std::error::Error>> {
    let allowed_host = crawler_core::Origin::from_url(seed)
        .ok_or("seed URL should have a host")?
        .host;

    Ok(CrawlConfig {
        seeds: vec![seed.clone()],         // one starting url in a list
        allowed_hosts: vec![allowed_host], // pulls hostname out of seed URL
        allowed_path_prefixes: Vec::new(), // no path restriction atm
        max_pages: args.max_pages,         // these are copied from the cmd
        max_depth: args.max_depth,
        global_concurrency: 1,            // (single worker so unused atm)
        per_origin_delay: Duration::ZERO, // how long to wait and politeness delay (0 since local)
        request_timeout: Duration::from_secs(10),
        connect_timeout: Duration::from_secs(5),
        max_body_bytes: 1024 * 1024, // how much of a pages body gets downloaded (1mb?)
        user_agent: "rust-web-crawler/0.1".to_string(), // identifier sent for every http req
    })
}

/// runs the actual crawler (put everything together.) using bfs, starting at seed
/// goes until queue is empty, or the max pages we set is hit.
pub async fn run_crawl(args: CrawlArgs) -> Result<CrawlSummary, Box<dyn std::error::Error>> {
    let started_at = Instant::now();
    let mut summary = CrawlSummary::default(); // empty result tracker
    let seed = url::Url::parse(&args.url)?; // turned input text into real URL
    let config = build_config(&args, &seed)?; // build settings! (function above)
    let fetcher = Fetcher::new(&config)?; // make the downloader (fetcher)
    let mut frontier = Frontier::new(CrawlJobId(1), config.max_pages); // create queue
    let admitted_seed = crawler_core::url::admit(&config, &seed, 0, None)?; // check if starting url allowed
    if let EnqueueResult::Admitted { .. } = frontier.enqueue(admitted_seed) {
        summary.unique_urls_discovered += 1; // add to queue and count it
    }
    // keep pulling URL's from queue until empty. count each as attempted.
    // then try to download each url
    while let Some((_url_id, admitted)) = frontier.dequeue() {
        summary.attempted += 1;
        match fetcher
            .fetch(admitted.url.clone(), config.max_body_bytes)
            .await
        {
            // in this case, download failed (network error or timeout, etc). counts it, moves on
            Err(_) => {
                frontier.mark_failed(&admitted.crawl_key);
                summary.failed += 1;
            }
            // downloaded, but it was not an html (like, an image or smtng)
            Ok(FetchOutcome::Skipped(_reason)) => {
                frontier.mark_complete(&admitted.crawl_key);
                summary.skipped += 1;
            }
            // otherwise good, got html and downloaded
            Ok(FetchOutcome::Html { body, .. }) => {
                frontier.mark_complete(&admitted.crawl_key);
                summary.successful += 1;
                summary.total_bytes += body.len();

                let html = String::from_utf8_lossy(&body);
                let doc = crawler_parser::parse_document(&html);
                let links = crawler_parser::extract_links(&doc, &admitted.url);

                // for each link found, try to queue (check new, dupe, queue size, or in scope, or skipped.)
                for link in links {
                    match crawler_core::url::admit(&config, &link.url, admitted.depth + 1, None) {
                        Ok(new_admitted) => match frontier.enqueue(new_admitted) {
                            EnqueueResult::Admitted { .. } => summary.unique_urls_discovered += 1,
                            EnqueueResult::Duplicate => summary.duplicate_links_rejected += 1,
                            EnqueueResult::AtCapacity => break,
                        },
                        Err(AdmissionError::OutOfScope) => summary.out_of_scope_rejected += 1,
                        Err(_) => summary.skipped += 1,
                    }
                }
            }
        }
    }
    summary.elapsed = started_at.elapsed();
    Ok(summary)
}

/// just a printed summary of everything
pub fn print_summary(summary: &CrawlSummary) {
    println!("Unique URLs discovered: {}", summary.unique_urls_discovered);
    println!("Attempted fetches:      {}", summary.attempted);
    println!("Successful fetches:     {}", summary.successful);
    println!("Skipped fetches:        {}", summary.skipped);
    println!("Failed fetches:         {}", summary.failed);
    println!("Total bytes downloaded: {}", summary.total_bytes);
    println!(
        "Duplicate links rejected:  {}",
        summary.duplicate_links_rejected
    );
    println!(
        "Out-of-scope rejected:     {}",
        summary.out_of_scope_rejected
    );
    println!("Elapsed time:           {:?}", summary.elapsed);
}
