use std::fmt;

use url::Url;

use crate::CrawlConfig;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CrawlKey(String);

impl CrawlKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CrawlKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum AdmissionError {
    #[error("unsupported scheme: {0}")]
    UnsupportedScheme(String),

    #[error("URL contains userinfo (username/password)")]
    UserinfoRejected,

    #[error("URL parse failed: {0}")]
    ParseFailed(String),

    #[error("URL has no host")]
    NoHost,

    #[error("URL is out of scope")]
    OutOfScope,

    #[error("max depth {max} exceeded at depth {depth}")]
    MaxDepthExceeded { max: u32, depth: u32 },
}

#[derive(Debug, Clone)]
pub struct AdmittedUrl {
    pub crawl_key: CrawlKey,
    pub url: Url,
    pub depth: u32,
    pub source_url_id: Option<super::UrlId>,
}

pub fn normalize_for_key(url: &Url) -> Result<CrawlKey, AdmissionError> {
    let scheme = url.scheme().to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err(AdmissionError::UnsupportedScheme(scheme));
    }

    if url.username() != "" || url.password().is_some() {
        return Err(AdmissionError::UserinfoRejected);
    }

    let host = url.host_str().ok_or(AdmissionError::NoHost)?;
    let host = host.trim_end_matches('.').to_ascii_lowercase();

    let default_port = match scheme.as_str() {
        "http" => Some(80u16),
        "https" => Some(443u16),
        _ => None,
    };

    let port_part = match (url.port(), default_port) {
        (Some(p), Some(dp)) if p == dp => String::new(),
        (Some(p), _) => format!(":{p}"),
        (None, _) => String::new(),
    };

    let mut path = url.path().to_string();
    path = resolve_dot_segments(&path);

    let query_part = match url.query() {
        Some(q) => format!("?{q}"),
        None => String::new(),
    };

    let key_str = format!("{scheme}://{host}{port_part}{path}{query_part}");
    Ok(CrawlKey(key_str))
}

pub fn resolve_dot_segments(path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();
    let mut leading_slash = false;

    if path.starts_with('/') {
        leading_slash = true;
    }

    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            s => segments.push(s),
        }
    }

    let mut result: String = String::new();

    if leading_slash {
        result.push('/');
    }

    result.push_str(&segments.join("/"));

    if path.ends_with('/') && !result.ends_with('/') {
        result.push('/');
    }

    result
}

pub fn admit(
    config: &CrawlConfig,
    url: &Url,
    depth: u32,
    source_url_id: Option<super::UrlId>,
) -> Result<AdmittedUrl, AdmissionError> {
    if !config.is_in_scope(url) {
        let scheme: String = url.scheme().to_string();

        if scheme != "http" && scheme != "https" {
            return Err(AdmissionError::UnsupportedScheme(scheme));
        }

        return Err(AdmissionError::OutOfScope);
    }

    if depth > config.max_depth {
        return Err(AdmissionError::MaxDepthExceeded {
            max: config.max_depth,
            depth,
        });
    }

    if url.username() != "" || url.password().is_some() {
        return Err(AdmissionError::UserinfoRejected);
    }

    let crawl_key: CrawlKey = normalize_for_key(url)?;

    Ok(AdmittedUrl {
        crawl_key,
        url: url.clone(),
        depth,
        source_url_id,
    })
}

pub fn resolve_relative(base: &Url, reference: &str) -> Result<Url, AdmissionError> {
    base.join(reference)
        .map_err(|e| AdmissionError::ParseFailed(e.to_string()))
}
