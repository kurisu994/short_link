use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Redirect;
use axum::routing::get;
use axum::Router;

use crate::types::IState;
use crate::{service::link_service, RedirectResult};

pub fn router() -> Router<Arc<IState>> {
    Router::new().route("/s/:hash", get(redirect))
}

async fn redirect(State(pool): State<Arc<IState>>, Path(hash): Path<String>) -> RedirectResult {
    let url = link_service::query_origin_url(pool, hash).await?;
    Ok(Redirect::permanent(&url))
}
