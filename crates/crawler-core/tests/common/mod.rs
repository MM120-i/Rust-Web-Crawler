use std::time::Duration;
use crawler_core::CrawlConfig;

pub fn test_config(allowed_hosts: Vec<&str>) -> CrawlConfig {
    CrawlConfig {
        seeds: vec![],
        allowed_hosts: allowed_hosts.into_iter().map(String::from).collect(),
        allowed_path_prefixes: vec![],
        max_pages: 100,
        max_depth: 3,
        global_concurrency: 5,
        per_origin_delay: Duration::from_millis(1000),
        request_timeout: Duration::from_secs(30),
        max_body_bytes: 10 * 1024 * 1024,
        user_agent: "crawler-test/0.1".into(),
    }
}

pub fn test_config_with_paths(allowed_hosts: Vec<&str>, paths: Vec<&str>) -> CrawlConfig {
    CrawlConfig {
        seeds: vec![],
        allowed_hosts: allowed_hosts.into_iter().map(String::from).collect(),
        allowed_path_prefixes: paths.into_iter().map(String::from).collect(),
        max_pages: 100,
        max_depth: 3,
        global_concurrency: 5,
        per_origin_delay: Duration::from_millis(1000),
        request_timeout: Duration::from_secs(30),
        max_body_bytes: 10 * 1024 * 1024,
        user_agent: "crawler-test/0.1".into(),
    }
}