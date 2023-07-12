use std::sync::Arc;

use axum::extract::{Path, State};
use axum::headers::HeaderMap;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;

use crate::types::{IState, RedirectResponse, RedirectResult};

pub fn router() -> Router<Arc<IState>> {
    Router::new().route("/s/:hash", get(redirect))
}

async fn redirect(State(_pool): State<Arc<IState>>, Path(hash): Path<String>) -> RedirectResult {
    tracing::info!("the path is {}", hash);

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::LOCATION,
        "https://testh5.feewee.cn".parse().unwrap(),
    );
    let redirect: RedirectResponse = (StatusCode::FOUND, headers, ());
    Ok(redirect)
}
