use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use validator::Validate;

use crate::pojo::AppError;
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

#[derive(Deserialize, Validate, Debug)]
#[allow(dead_code)]
struct CreateLink {
    #[validate(url(message = "无效"), required(message = "不能为空"))]
    url: Option<String>,
    duration: Option<u64>,
}

async fn create_link(
    State(pool): State<Arc<IState>>,
    Json(payload): Json<CreateLink>,
) -> MessageResult<String> {
    if let Err(e) = payload.validate() {
        return Err(AppError::from(e));
    }
    let res = link_service::create_link(pool, payload.url.unwrap(), payload.duration).await?;
    Ok(Message::ok(res))
}
