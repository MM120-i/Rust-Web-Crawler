//! makes cli.rs and crawl.rs usable from outside this crate for testing
//! (crawler-cli/tests/) can call `crawl::run_crawl` directly

pub mod cli;
pub mod crawl;
