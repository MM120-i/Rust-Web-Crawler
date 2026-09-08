use clap::Parser;
use crawler_cli::cli::{Cli, Commands};
use crawler_cli::crawl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // reads cmdline
    let cli = Cli::parse();

    // matches based on option (right now, crawl is the only option lol, cli.rs) and then runs
    match cli.command {
        Commands::Crawl(args) => {
            let summary = crawl::run_crawl(args).await?;
            crawl::print_summary(&summary);
        }
    }

    Ok(())
}
