// basic web crawler
// https://books.toscrape.com/ or https://quotes.toscrape.com/ for testing
// https://www.hellointerview.com/learn/system-design/problem-breakdowns/web-crawler


// only returns links of original page -> no crawling involved (just surface level)
// this was just a test 
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let url = "https://books.toscrape.com/";

    // fetch the page
    // get request for Future, ? bails and returns error on error.
    let body = reqwest::get(url).await?.text().await?;
    println!("Fetched {} bytes", body.len());

    // parse it and pull out every link
    // doc is an html value now, we pass & body so body is not consumed
    let doc = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("a[href]").unwrap();

    for element in doc.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            println!("{}", href);
        }
    }
    Ok(())
}

