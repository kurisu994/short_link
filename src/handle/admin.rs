use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::Router;

use crate::pojo::{link_history::LinkHistory, Message, Pagination};
use crate::types::{IState, MessageResult};

pub fn router() -> Router<Arc<IState>> {
    Router::new().route("/link/list", get(link_list))
}

async fn link_list<'a>(
    State(_pool): State<Arc<IState>>,
    pagination: Option<Query<Pagination>>,
) -> MessageResult<Vec<LinkHistory<'a>>> {
    let Query(pagination) = pagination.unwrap_or_default();
    tracing::info!("pagination is {:?}", pagination);
    Ok(Message::failed(""))
}
