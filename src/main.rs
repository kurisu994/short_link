use std::net::SocketAddr;

use axum::http::{Method, StatusCode};
use axum::response::IntoResponse;
use axum::{
    routing::{get, post},
    Router,
};
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};

use idgen::{IdGeneratorOptions, YitIdHelper};

use crate::demo::gen_union_id;
use crate::{
    demo::{base62_to_usize, create_user, redirect, root, usize_to_base62},
    pojo::AppError,
    pojo::Message,
    types::{HandlerResult, MessageResult, RedirectResponse, RedirectResult},
};

mod demo;
mod handle;
mod idgen;
mod pojo;
mod types;
mod utils;

#[tokio::main]
async fn main() {
    let options = IdGeneratorOptions::default();
    YitIdHelper::set_id_generator(options);

    tracing_subscriber::fmt::init();

    let app = Router::new()
        // .merge(router_fallible_middleware()) // 模拟使用中间件的错误处理
        // .merge(router_fallible_extractor())  // 模拟使用提取器的错误处理
        .route("/", get(root))
        .route("/id", get(gen_union_id))
        .route("/302", get(redirect))
        .route("/to_link", get(usize_to_base62))
        .route("/to_no", get(base62_to_usize))
        .route("/users", post(create_user))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
        ]))
        .fallback(handler_404);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8008));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 not found")
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
