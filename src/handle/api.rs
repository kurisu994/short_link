use std::sync::Arc;

use axum::extract::{Path, State};
use axum::headers::HeaderMap;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;

use crate::service::link_service;
use crate::types::{IState, RedirectResponse, RedirectResult};

pub fn router() -> Router<Arc<IState>> {
    Router::new().route("/s/:hash", get(redirect))
}

async fn redirect(State(pool): State<Arc<IState>>, Path(hash): Path<String>) -> RedirectResult {
    tracing::info!("the path is {}", hash);
    let url = link_service::query_origin_url(pool, hash).await?;
    let mut headers = HeaderMap::new();
    headers.insert(axum::http::header::LOCATION, url.parse().unwrap());
    let redirect: RedirectResponse = (StatusCode::FOUND, headers, ());
    Ok(redirect)
}
