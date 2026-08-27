// crawler-fetch: the "delivery driver" crate.
// Job: given a URL, go get the raw bytes off the network — safely, with limits.
// It should NOT know anything about parsing HTML — that's crawler-parser's job.
// This struct is the "one reusable reqwest::Client per worker" from the checklist.
// Think of `reqwest::Client` as a phone line: expensive to set up (DNS lookup,
// TLS handshake setup, connection pool), cheap to reuse. So we build ONE and
// keep it alive for the whole life of the worker, instead of making a new
// phone line for every single URL we fetch.
pub struct Fetcher {
    // TODO: store a `reqwest::Client` here.
    // client: reqwest::Client,
    client: reqwest::Client,
}

impl Fetcher {
    // This is where checklist items 1-3 all come together:
    //   1. build ONE client (this whole function only runs once per worker)
    //   2. set a custom User-Agent so we identify ourselves honestly
    //      (do NOT copy a real browser's user agent string — that's imitating
    //      a browser, which the checklist explicitly says not to do)
    //   3. set a connect timeout (max time to establish the connection) AND
    //      a total/overall timeout (max time for the whole request,
    //      including downloading the body) — these are two different knobs
    //      on reqwest::ClientBuilder, both matter
    //
    // Rough shape of what needs to happen here (pseudocode, not real syntax):
    //
    //   let client = reqwest::Client::builder()
    //       .user_agent("your-product-name/0.1 (+contact-or-url)")
    //       .connect_timeout(...)   // e.g. a few seconds
    //       .timeout(...)           // overall request deadline
    //       .build()?;
    //
    //   Fetcher { client }
    //
    // Question to think about while writing this: where should the timeout
    // durations and user-agent string come from? (Hint: crawler-core already
    // has a `CrawlConfig` struct with `request_timeout` and `user_agent`
    // fields — this constructor probably wants to take a `&CrawlConfig`.)
    pub fn new(config: &crawler_core::CrawlConfig) -> Result<Self, reqwest::Error> {
        // Item 4: redirects.
        //
        // By default reqwest silently auto-follows up to 10 redirects for you.
        // That's the problem: page A might be in scope, but a 302 could send
        // you to a completely different host, a weird scheme, or loop forever
        // — and reqwest would just go along with it before you ever see it.
        //
        // Fix: reqwest::redirect::Policy::custom(closure) lets you intercept
        // EVERY hop. reqwest calls your closure with an `Attempt` before it
        // follows a redirect, and you return one of:
        //   attempt.follow()        -> ok, go ahead
        //   attempt.stop()          -> stop here, treat current response as final
        //   attempt.error(some_err) -> abort the whole request as an error
        //
        // `Attempt` gives you `.url()` (where it wants to redirect to) and
        // `.previous()` (the chain of URLs already visited this request, so
        // you can also cap redirect depth yourself, e.g. previous().len() > 5).
        //
        // The tricky Rust part: this closure has to be `'static + Send + Sync`
        // (reqwest may reuse it across requests/threads), so it CANNOT borrow
        // `config: &CrawlConfig` by reference — that reference only lives as
        // long as this `new()` call. You need to move an *owned* value in
        // instead. CrawlConfig already derives Clone, so something like
        // `let config_for_redirects = config.clone();` before the builder,
        // then use `config_for_redirects` inside the closure, would work.
        //
        // Rough shape (pseudocode):
        //
        //   let policy = reqwest::redirect::Policy::custom(move |attempt| {
        //       if config_for_redirects.is_in_scope(attempt.url()) {
        //           attempt.follow()
        //       } else {
        //           attempt.stop()
        //       }
        //   });
        //
        // Question to think about: CrawlConfig::is_in_scope already exists
        // (crawler-core/src/lib.rs) — does checking scope alone cover
        // "safety" too, or is there something else worth rejecting here
        // (e.g. non-http(s) schemes, redirect chains that are too long)?

        let config_for_redirects = config.clone();
        let policy = reqwest::redirect::Policy::custom(move |attempt| {
            if config_for_redirects.is_in_scope(attempt.url()) {
                attempt.follow()
            } else {
                attempt.stop()
            }
        });

        let client = reqwest::Client::builder()
            .user_agent(&config.user_agent)
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .redirect(policy)
            .build()?;

        Ok(Fetcher { client })
    }
}
