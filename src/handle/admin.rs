use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use crate::{
    link_service,
    pojo::{link_history::LinkHistory, Message, Pagination},
    types::{IState, MessageResult},
};

pub fn router() -> Router<Arc<IState>> {
    Router::new()
        .route("/link/list", get(link_list))
        .route("/link/create", post(create_link))
}

async fn link_list<'a>(
    State(_pool): State<Arc<IState>>,
    pagination: Option<Query<Pagination>>,
) -> MessageResult<Vec<LinkHistory>> {
    let Query(pagination) = pagination.unwrap_or_default();
    tracing::info!("pagination is {:?}", pagination);
    Ok(Message::failed(""))
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct CreateLink {
    url: String,
    duration: Option<u64>,
}

async fn create_link<'a>(
    State(pool): State<Arc<IState>>,
    Json(payload): Json<CreateLink>,
) -> MessageResult<LinkHistory> {
    let res = link_service::create_link(pool, payload.url, payload.duration).await?;

    Ok(Message::failed(&res))
}
