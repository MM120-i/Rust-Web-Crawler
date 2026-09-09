//! fake site for crawler testing lol

pub const TOTAL_PAGES: u32 = 97; // fake site has up to 100 pages total. 97 + 3 fixture routes

// A & B forever loops into each other (loop test)
pub const REDIRECT_LOOP_A: &str = "/redirect-a";
pub const REDIRECT_LOOP_B: &str = "/redirect-b";
pub const OVERSIZED_PAGE: &str = "/huge"; // page thats too big (test size limit)
pub const MALFORMED_PAGE: &str = "/broken"; // broken html to make sure it still parses
pub const OUT_OF_SCOPE_URL: &str = "http://out-of-scope.invalid/evil"; // domain doesnt exist

/// has all the links (hrefs) a page will hold
pub struct Page {
    pub links: Vec<String>,
}

/// list of pages, starting from index 0 for /root
pub struct Graph {
    pub pages: Vec<Page>,
}

/// page 0 is /root, everything else is /page/{id}.
fn page_path(id: u32) -> String {
    if id == 0 {
        "/root".to_string()
    } else {
        format!("/page/{id}")
    }
}

/// tree of pages, each page has up to 4 children. 100 pages deep
pub fn build_graph(total_pages: u32) -> Graph {
    const BRANCHING_FACTOR: u32 = 4;

    // child id is calculated as id * 4 + 1
    let mut pages: Vec<Page> = (0..total_pages)
        .map(|id| {
            let mut links: Vec<String> = Vec::new();
            for child in 1..=BRANCHING_FACTOR {
                let child_id = id * BRANCHING_FACTOR + child;
                if child_id < total_pages {
                    links.push(page_path(child_id));
                }
            }
            Page { links }
        })
        .collect();

    // special test cases to root page for testing (like the dupe or redirect loop)
    if let Some(root) = pages.get_mut(0) {
        if let Some(first_child) = root.links.first().cloned() {
            root.links.push(first_child); // duplicate link
        }
        root.links.push(OUT_OF_SCOPE_URL.to_string());
        root.links.push(REDIRECT_LOOP_A.to_string());
        root.links.push(OVERSIZED_PAGE.to_string());
        root.links.push(MALFORMED_PAGE.to_string());
    }

    Graph { pages }
}

/// turns page into actual HTML, one href per link and all
pub fn render_page(page_id: u32, page: &Page) -> String {
    let mut links_html = String::new();
    for href in &page.links {
        links_html.push_str(&format!(r#"<a href="{href}">link</a>"#));
    }

    format!("<html><head><title>Page {page_id}</title></head><body>{links_html}</body></html>")
}

/// returns html with body 10mb of "x", for fetcher page too big
pub fn oversized_body() -> String {
    format!("<html><body>{}</body></html>", "x".repeat(10 * 1024 * 1024))
}

/// intentionally broken HTML
pub fn malformed_body() -> &'static str {
    "<html><body><p>unclosed paragraph<div>bad nesting</div>"
}
