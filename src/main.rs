use std::net::SocketAddr;

use axum::{
    http::StatusCode,
    Json,
    Router, routing::{get, post},
};
use axum::extract::Query;
use serde::{Deserialize, Serialize};

use crate::utils::helper;

mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/base62", get(usize_to_base62))
        .route("/number", get(base62_to_usize))
        .route("/users", post(create_user));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8008));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn usize_to_base62(Query(query): Query<Param>) -> String {
    println!("{:?}", query);
    helper::encode_base62(query.no.unwrap_or(0))
}

async fn base62_to_usize(Query(query): Query<Param>) -> String {
    println!("{:?}", query);
    let link = query.link.unwrap_or("0".to_string());
    let res = helper::decode_base62(&link);
    format!("{}", res)
}

async fn create_user(
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    let user = User {
        id: 1337,
        username: payload.username,
    };

    (StatusCode::CREATED, Json(user))
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

#[derive(Deserialize, Debug)]
struct Param {
    no: Option<usize>,
    link: Option<String>,
}
