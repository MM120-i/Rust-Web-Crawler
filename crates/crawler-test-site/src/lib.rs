//! local HTML fixture/fake pages defined by graph.rs for testing

pub mod graph;

use std::net::SocketAddr;
use std::sync::Arc;

// axum library used for routing, path extraction, html response, redirect
// https://docs.rs/axum/latest/axum/
use axum::extract::{Path, State};
use axum::response::{Html, Redirect};
use axum::routing::get;
use axum::{Router, http::StatusCode};

use graph::Graph;

/// running server rep, network address and base url thats ready to use
pub struct SpawnedSite {
    pub addr: SocketAddr,
    pub base_url: url::Url,
}

/// spawn the site, puts it all together
pub async fn spawn() -> SpawnedSite {
    // 100 page site, wraps it in Arc (shared, thread safe reference to allow for multiple requests)
    let graph = Arc::new(graph::build_graph(graph::TOTAL_PAGES));
    let router = build_router(graph);
    // binds TCP listener to the addr, :0 meaning os picks any free port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind crawler-test-site");
    // reads back the atual address or port it got asigned
    let addr = listener.local_addr().expect("failed to read local addr");

    // spawns server as background async task
    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("crawler-test-site server crashed");
    });

    // build a base url from the address given
    let base_url = url::Url::parse(&format!("http://{addr}")).expect("valid base url");

    SpawnedSite { addr, base_url }
}

/// maps url paths to handler functions
fn build_router(graph: Arc<Graph>) -> Router {
    Router::new()
        .route("/root", get(get_root))
        .route("/page/{id}", get(get_page))
        .route(graph::REDIRECT_LOOP_A, get(get_redirect_a))
        .route(graph::REDIRECT_LOOP_B, get(get_redirect_b))
        .route(graph::OVERSIZED_PAGE, get(get_huge))
        .route(graph::MALFORMED_PAGE, get(get_broken))
        .with_state(graph)
}

/// renders the root (page 0) & returns it
async fn get_root(State(graph): State<Arc<Graph>>) -> Html<String> {
    Html(graph::render_page(0, &graph.pages[0]))
}

/// gets the page with id {id} and renders it or 404 if it doesnt exist
async fn get_page(
    State(graph): State<Arc<Graph>>,
    Path(id): Path<u32>,
) -> Result<Html<String>, StatusCode> {
    let page = graph.pages.get(id as usize).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Html(graph::render_page(id, page)))
}

/// the redirects, a goes to b, b goes to a.
async fn get_redirect_a() -> Redirect {
    Redirect::temporary(graph::REDIRECT_LOOP_B)
}

async fn get_redirect_b() -> Redirect {
    Redirect::temporary(graph::REDIRECT_LOOP_A)
}

/// gets the giant page
async fn get_huge() -> Html<String> {
    Html(graph::oversized_body())
}

/// gets the broken html page
async fn get_broken() -> Html<&'static str> {
    Html(graph::malformed_body())
}
