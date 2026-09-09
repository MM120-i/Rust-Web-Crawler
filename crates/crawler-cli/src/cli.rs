//! command line stuff for putting everything together
//! for example: crawler crawl http://crawler-test-site:PORT/root --max-pages 100 --max-depth 5

use clap::{Args, Parser, Subcommand};
// clap = rust library for cmd line input https://docs.rs/clap/latest/clap/

// entry point of cli command for program. crawler <something>
#[derive(Parser)]
#[command(name = "crawler", version, about = "Rust web crawler CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

// list of commands crawler can have (atm, only crawl)
#[derive(Subcommand)]
pub enum Commands {
    /// Crawl a site starting from a seed URL and print a summary.
    Crawl(CrawlArgs),
}

#[derive(Args, Debug)]
pub struct CrawlArgs {
    /// seed URL to start crawling from, e.g. http://crawler-test-site:PORT/root
    pub url: String,

    /// stop once this many pages have been fetched, default 100
    /// --max-pages argument
    #[arg(long, default_value_t = 100)]
    pub max_pages: u32,

    /// don't follow links deeper than this many hops from the seed, default 5
    /// --max-depth argument
    #[arg(long, default_value_t = 5)]
    pub max_depth: u32,
}
