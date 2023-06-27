use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Router,
};

use crate::demo::{base62_to_usize, create_user, redirect, root, usize_to_base62};
pub use pojo::Message;
mod demo;
mod handle;
mod pojo;
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        // .merge(router_fallible_service()) // 模拟使用 Service的错误处理
        // .merge(router_fallible_middleware()) // 模拟使用中间件的错误处理
        // .merge(router_fallible_extractor())  // 模拟使用提取器的错误处理
        .route("/", get(root))
        .route("/302", get(redirect))
        .route("/base62", get(usize_to_base62))
        .route("/number", get(base62_to_usize))
        .route("/users", post(create_user));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8008));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
