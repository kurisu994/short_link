use axum::extract::{Query,State};
use axum::headers::HeaderMap;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::idgen::YitIdHelper;
use crate::pojo::AppError;
use crate::utils::helper;
use crate::{HandlerResult, IState};
use crate::Message;
use crate::{MessageResult, RedirectResponse, RedirectResult};

pub fn router() -> Router<Arc<IState>> {
    Router::new()
        .route("/", get(root))
        .route("/id", get(gen_union_id))
        .route("/302", get(redirect))
        .route("/to_link", get(usize_to_base62))
        .route("/to_no", get(base62_to_usize))
        .route("/users", post(create_user))
        .route("/sqlx", get(using_connection_pool_extractor))
        .route("/redis", get(using_connection_pool_redis))
}

pub async fn root() -> &'static str {
    "Hello, World!"
}

async fn using_connection_pool_extractor(
    State(pool): State<Arc<IState>>,
) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("select 'hello world' from link_history")
        .fetch_one(&pool.db_pool)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

async fn using_connection_pool_redis(
    State(pool): State<Arc<IState>>,
) -> Result<String, (StatusCode, String)> {
    let mut redis_conn = pool.redis_pool.get().await.unwrap();
    let reply: String = redis::cmd("PING").query_async(&mut *redis_conn).await.unwrap();

    Ok(reply)
}

pub async fn gen_union_id() -> MessageResult<i64> {
    let next_id = YitIdHelper::next_id();
    println!("next_id: {}", next_id);
    Ok(Message::ok(next_id))
}

pub async fn usize_to_base62(Query(query): Query<Param>) -> MessageResult<String> {
    println!("no is: {:?}", query.no);
    let b62 = helper::encode_base62(query.no.unwrap_or(0));
    Ok(Message::ok(b62))
}

pub async fn base62_to_usize(Query(query): Query<Param>) -> HandlerResult<Message<String>> {
    println!("link is: {:?}", query.link);
    if query.link == None {
        return Err(AppError::from(anyhow::anyhow!("link is not found")));
    }
    let link = query.link.unwrap();
    let res = helper::decode_base62(&link)?;
    Ok(Message::ok(format!("{}", res)))
}

pub async fn create_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<Message<User>>) {
    let user = User {
        id: 1337,
        username: payload.username,
    };

    (StatusCode::OK, Json(Message::ok(user)))
}

pub async fn redirect() -> RedirectResult {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::LOCATION,
        "https://testh5.feewee.cn".parse().unwrap(),
    );
    let redirect: RedirectResponse = (StatusCode::FOUND, headers, ());
    Ok(redirect)
}

#[derive(Deserialize)]
pub struct CreateUser {
    username: String,
}

#[derive(Serialize)]
pub struct User {
    id: u64,
    username: String,
}

#[derive(Deserialize, Debug)]
pub struct Param {
    no: Option<usize>,
    link: Option<String>,
}
