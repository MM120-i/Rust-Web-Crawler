/// extracted <a href> link, the url, text, and if its marked as a nofollow (shape)
#[derive(Debug, PartialEq)]
pub struct ExtractedLink {
    pub url: url::Url,
    pub anchor_text: String,
    pub no_follow: bool,
}

/// gets html and parses into document, doesnt panic on malformed html
pub fn parse_document(html: &str) -> scraper::Html {
    scraper::Html::parse_document(html)
}

/// extracts the <title> text or None
pub fn extract_title(document: &scraper::Html) -> Option<String> {
    let selector = scraper::Selector::parse("title").ok()?;
    let element = document.select(&selector).next()?;
    Some(element.text().collect()) // collects many strings into one
}

/// extracts the <link rel="canonical" href="..."> or None
pub fn extract_canonical(document: &scraper::Html) -> Option<String> {
    let selector = scraper::Selector::parse(r#"link[rel="canonical"]"#).ok()?;
    let element = document.select(&selector).next()?;
    let href = element.value().attr("href")?;
    Some(href.to_string()) // .attr href and lang already returns one string so we just need to convert it
}

/// extracts <html lang="..."> or None
pub fn extract_language(document: &scraper::Html) -> Option<String> {
    let selector = scraper::Selector::parse("html").ok()?;
    let element = document.select(&selector).next()?;
    let lang = element.value().attr("lang")?;
    Some(lang.to_string())
}

/// extracts pages visible text (no <script>`/`<style>)
pub fn extract_visible_text(document: &scraper::Html) -> String {
    let selector = scraper::Selector::parse("*:not(script):not(style)").unwrap();
    let mut text = String::new();
    // goes through every non script/style element (ie, <p>, <div>, <body>)
    for element in document.select(&selector) {
        for child in element.children() {
            // if its a text node, push it
            if let scraper::Node::Text(t) = child.value() {
                text.push_str(t);
            }
        }
    }

    text
}

/// extracts all <a href>, attached to its base_url (incase its local) and checks for nofollow flag
pub fn extract_links(document: &scraper::Html, base_url: &url::Url) -> Vec<ExtractedLink> {
    let selector = scraper::Selector::parse("a[href]").unwrap();
    let mut links = Vec::new();

    // loop over every <a> tag with a href attribute
    for element in document.select(&selector) {
        let href = element.value().attr("href").unwrap(); // unwrap is safe because selector already proves it exists
        // grab the links label and check if theres a nofollow
        let anchor_text: String = element.text().collect();
        let no_follow = element
            .value()
            .attr("rel")
            .is_some_and(|rel| rel.split_whitespace().any(|token| token == "nofollow"));

        // turn href into absolute url (using the base url of the page) then continue if not malformed
        if let Ok(url) = base_url.join(href) {
            links.push(ExtractedLink {
                url,
                anchor_text,
                no_follow,
            });
        }
    }

    links
}

/// single unit tests for this
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_does_not_panic_on_malformed_html() {
        let broken = "<html><p>unclosed<div>ohwelllol</p>";
        let _doc = parse_document(broken);
    }

    #[test]
    fn test_extracts_title() {
        let doc =
            parse_document("<html><head><title>Hello World</title></head><body></body></html>");
        assert_eq!(extract_title(&doc), Some("Hello World".to_string()));
    }

    #[test]
    fn test_extract_title_returns_none_when_missing() {
        let doc = parse_document("<html><body><p>no title here</p></body></html>");
        assert_eq!(extract_title(&doc), None);
    }

    #[test]
    fn test_extracts_canonical() {
        let doc = parse_document(
            r#"<html><head><link rel="canonical" href="https://example.com/page"></head></html>"#,
        );
        assert_eq!(
            extract_canonical(&doc),
            Some("https://example.com/page".to_string())
        );
    }

    #[test]
    fn test_extract_canonical_returns_none_when_missing() {
        let doc = parse_document("<html><head></head></html>");
        assert_eq!(extract_canonical(&doc), None);
    }

    #[test]
    fn test_extracts_language() {
        let doc = parse_document(r#"<html lang="en"><head></head><body></body></html>"#);
        assert_eq!(extract_language(&doc), Some("en".to_string()));
    }

    #[test]
    fn test_extract_language_returns_none_when_missing() {
        let doc = parse_document("<html><head></head><body></body></html>");
        assert_eq!(extract_language(&doc), None);
    }

    #[test]
    fn test_extracts_visible_text() {
        let doc = parse_document("<html><body><p>Hello</p><p>World</p></body></html>");
        assert_eq!(extract_visible_text(&doc), "HelloWorld");
    }

    #[test]
    fn test_visible_text_skips_script_and_style() {
        let doc = parse_document(
            "<html><body><p>Visible</p><script>var x = 1;</script><style>p { color: red; }</style></body></html>",
        );
        assert_eq!(extract_visible_text(&doc), "Visible");
    }

    #[test]
    fn test_extracts_absolute_link() {
        let doc = parse_document(
            r#"<html><body><a href="https://other.com/page">Click</a></body></html>"#,
        );
        let base = url::Url::parse("https://example.com/").unwrap();
        let links = extract_links(&doc, &base);
        assert_eq!(
            links,
            vec![ExtractedLink {
                url: url::Url::parse("https://other.com/page").unwrap(),
                anchor_text: "Click".to_string(),
                no_follow: false,
            }]
        );
    }

    #[test]
    fn test_resolves_relative_link_against_base() {
        let doc = parse_document(r#"<html><body><a href="/about">About</a></body></html>"#);
        let base = url::Url::parse("https://example.com/blog/post").unwrap();
        let links = extract_links(&doc, &base);
        assert_eq!(
            links,
            vec![ExtractedLink {
                url: url::Url::parse("https://example.com/about").unwrap(),
                anchor_text: "About".to_string(),
                no_follow: false,
            }]
        );
    }

    #[test]
    fn test_skips_a_tags_without_href() {
        let doc = parse_document(r#"<html><body><a name="anchor">No link here</a></body></html>"#);
        let base = url::Url::parse("https://example.com/").unwrap();
        assert_eq!(extract_links(&doc, &base), vec![]);
    }

    #[test]
    fn test_marks_nofollow_link() {
        let doc = parse_document(
            r#"<html><body><a href="/page" rel="noopener nofollow">Link</a></body></html>"#,
        );
        let base = url::Url::parse("https://example.com/").unwrap();
        let links = extract_links(&doc, &base);
        assert_eq!(
            links,
            vec![ExtractedLink {
                url: url::Url::parse("https://example.com/page").unwrap(),
                anchor_text: "Link".to_string(),
                no_follow: true,
            }]
        );
    }
}
