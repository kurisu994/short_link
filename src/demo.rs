use crate::utils::helper;
use crate::Message;
use axum::extract::Query;
use axum::headers::HeaderMap;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

pub async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn usize_to_base62(Query(query): Query<Param>) -> String {
    println!("{:?}", query);
    helper::encode_base62(query.no.unwrap_or(0))
}

pub async fn base62_to_usize(Query(query): Query<Param>) -> String {
    println!("{:?}", query);
    let link = query.link.unwrap_or("0".to_string());
    let res = helper::decode_base62(&link);
    format!("{}", res)
}

pub async fn create_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<Message<User>>) {
    let user = User {
        id: 1337,
        username: payload.username,
    };

    (StatusCode::OK, Json(Message::ok(user)))
}

pub async fn redirect() -> (StatusCode, HeaderMap, ()) {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::LOCATION,
        "https://testh5.feewee.cn".parse().unwrap(),
    );
    (StatusCode::FOUND, headers, ())
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
