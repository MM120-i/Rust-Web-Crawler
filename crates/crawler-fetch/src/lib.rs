/// FetchOutcome, either returns a html or a reason for a skip (wasnt html)
pub enum FetchOutcome {
    Html(Vec<u8>),
    Skipped(crawler_core::SkipReason),
}
/// stores one reqwest::Client here for reuse, expensive to just keep making
pub struct Fetcher {
    // TODO: store a `reqwest::Client` here.
    // client: reqwest::Client,
    client: reqwest::Client,
}

/// implementing the Fetcher struct
impl Fetcher {
    /// one time setup for a worker, reused for each fetch.
    pub fn new(config: &crawler_core::CrawlConfig) -> Result<Self, reqwest::Error> {
        // clone to have no dangling ref (original will die once new() is done)
        let config_for_redirects = config.clone();
        let policy = reqwest::redirect::Policy::custom(move |attempt| {
            // if we are past 5 hops (arbitrary number), just bail to prevent indinite redirection loops
            if attempt.previous().len() > 5 {
                return attempt.stop();
            }
            // make sure this crawl is within the allowed scope
            if config_for_redirects.is_in_scope(attempt.url()) {
                attempt.follow()
            } else {
                attempt.stop()
            }
        });

        let client = reqwest::Client::builder()
            .user_agent(&config.user_agent) // crawler identifier (rather than browser)
            .connect_timeout(config.connect_timeout) // max time to establish connection
            .timeout(config.request_timeout) // max time for entire request
            .redirect(policy) // checks policy for each url (above)
            .build()?; // build into actual client, and if build fails, bail early with error (?)

        Ok(Fetcher { client })
    }

    /// fetches a page, either returns the html or a skipped outcome
    pub async fn fetch(
        &self,
        url: reqwest::Url,
        byte_limit: usize,
    ) -> Result<FetchOutcome, reqwest::Error> {
        let mut response = self.client.get(url).send().await?; // uses client from new()
        // checks if its an html, if not, skip
        let is_html = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|s| s.starts_with("text/html"));

        if !is_html {
            return Ok(FetchOutcome::Skipped(
                crawler_core::SkipReason::UnsupportedMediaType,
            ));
        }

        // otherwise, copy with chunks rather than at once, until we hit our byte limit
        let mut body: Vec<u8> = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if body.len() + chunk.len() > byte_limit {
                let remainder = byte_limit - body.len();
                body.extend_from_slice(&chunk[..remainder]);
                break;
            } else {
                body.extend_from_slice(&chunk);
            }
        }
        Ok(FetchOutcome::Html(body))
    }
}
