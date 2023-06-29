use axum::extract::Query;
use axum::headers::HeaderMap;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::idgen::YitIdHelper;
use crate::utils::helper;
use crate::HandlerResult;
use crate::Message;
use crate::{MessageResult, RedirectResponse, RedirectResult};

pub async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn gen_union_id() -> MessageResult<i64> {
    let next_id = YitIdHelper::next_id();
    println!("next_id: {}", next_id);
    Ok(Message::ok(next_id))
}

pub async fn usize_to_base62(Query(query): Query<Param>) -> MessageResult<String> {
    println!("{:?}", query);
    let b62 = helper::encode_base62(query.no.unwrap_or(0));
    Ok(Message::ok(b62))
}

pub async fn base62_to_usize(Query(query): Query<Param>) -> HandlerResult<Message<String>> {
    println!("{:?}", query);
    let link = query.link.unwrap_or("0".to_string());
    let res = helper::decode_base62(&link);
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
