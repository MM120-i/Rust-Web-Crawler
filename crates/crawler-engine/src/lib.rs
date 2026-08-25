use std::collections::{HashSet, VecDeque};

use crawler_core::url::{AdmittedUrl, CrawlKey};
use crawler_core::{CrawlJobId, UrlId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnqueueResult {
    Admitted { url_id: UrlId },
    Duplicate,
    AtCapacity,
}

#[derive(Debug, Clone)]
pub struct FrontierStats {
    pub seen_count: usize,
    pub queued_count: usize,
    pub in_flight_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
    pub total_enqueued: usize,
}

pub struct Frontier {
    #[allow(dead_code)]
    job_id: CrawlJobId,
    seen: HashSet<CrawlKey>,
    queue: VecDeque<AdmittedUrl>,
    in_flight: HashSet<CrawlKey>,
    pages_fetched: u32,
    max_pages: u32,
    next_url_id: u64,
    completed_count: u64,
    failed_count: u64,
    total_enqueued: u64,
}

impl Frontier {
    pub fn new(job_id: CrawlJobId, max_pages: u32) -> Self {
        Self {
            job_id,
            seen: HashSet::new(),
            queue: VecDeque::new(),
            in_flight: HashSet::new(),
            pages_fetched: 0,
            max_pages,
            next_url_id: 1,
            completed_count: 0,
            failed_count: 0,
            total_enqueued: 0,
        }
    }

    pub fn enqueue(&mut self, admitted: AdmittedUrl) -> EnqueueResult {
        if self.pages_fetched + (self.queue.len() as u32) >= self.max_pages {
            return EnqueueResult::AtCapacity;
        }

        if self.seen.contains(&admitted.crawl_key)
            || self.in_flight.contains(&admitted.crawl_key)
        {
            return EnqueueResult::Duplicate;
        }

        let url_id = UrlId(self.next_url_id);
        self.next_url_id += 1;

        self.seen.insert(admitted.crawl_key.clone());
        self.queue.push_back(admitted);
        self.total_enqueued += 1;

        EnqueueResult::Admitted { url_id }
    }

    pub fn dequeue(&mut self) -> Option<(UrlId, AdmittedUrl)> {
        let admitted = self.queue.pop_front()?;
        let url_id = UrlId(self.next_url_id - self.queue.len() as u64 - 1);

        self.in_flight.insert(admitted.crawl_key.clone());

        Some((url_id, admitted))
    }

    pub fn mark_complete(&mut self, crawl_key: &CrawlKey) {
        self.in_flight.remove(crawl_key);
        self.pages_fetched += 1;
        self.completed_count += 1;
    }

    pub fn mark_failed(&mut self, crawl_key: &CrawlKey) {
        self.in_flight.remove(crawl_key);
        self.failed_count += 1;
    }

    pub fn stats(&self) -> FrontierStats {
        FrontierStats {
            seen_count: self.seen.len(),
            queued_count: self.queue.len(),
            in_flight_count: self.in_flight.len(),
            completed_count: self.completed_count as usize,
            failed_count: self.failed_count as usize,
            total_enqueued: self.total_enqueued as usize,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty() && self.in_flight.is_empty()
    }

    pub fn pages_fetched(&self) -> u32 {
        self.pages_fetched
    }
}
