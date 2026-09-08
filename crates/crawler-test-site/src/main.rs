//! used for testing the crawler manually

#[tokio::main]
async fn main() {
    // start the fake site for crawler
    let site = crawler_test_site::spawn().await;
    // print url its runnign on, and a cmd you can try to point crawler-cli to the test thing
    println!("crawler-test-site listening on {}", site.base_url);
    println!(
        "try: cargo run -p crawler-cli -- crawl {}root --max-pages 100 --max-depth 5",
        site.base_url
    );
    // waits till you hit ctrl c, allows for manual testing and fidgeting
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c");
}
