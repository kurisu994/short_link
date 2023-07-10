use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::http::{Method, StatusCode};
use axum::response::IntoResponse;
use axum::Router;
use sqlx::{MySql,  Pool};
use sqlx::mysql::MySqlPoolOptions;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use idgen::{IdGeneratorOptions, YitIdHelper};

use crate::{
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

#[derive(Clone)]
pub struct IState {
    db_pool: Pool<MySql>,
    redis_pool: bb8::Pool<bb8_redis::RedisConnectionManager>,
}

#[tokio::main]
async fn main() {
    let options = IdGeneratorOptions::default();
    YitIdHelper::set_id_generator(options);
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "short_link=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    if let Err(err) = run_server().await {
        eprintln!("Server error: {}", err);
    }
}

async fn run_server() -> Result<(), axum::Error> {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let db_pool = MySqlPoolOptions::new()
        .max_connections(15)
        .acquire_timeout(Duration::from_secs(15))
        .connect(&db_url)
        .await
        .unwrap();

    let redis_url = std::env::var("REDIS_URL").unwrap();
    tracing::debug!("redis_url is {}", redis_url);

    let redis_manager = bb8_redis::RedisConnectionManager::new(redis_url).unwrap();
    let redis_pool = bb8::Pool::builder()
        .max_size(20)
        .build(redis_manager)
        .await
        .unwrap();

    // 创建状态对象
    let state = Arc::new(IState {
        db_pool,
        redis_pool,
    });

    let app = api_router()
        .layer(CorsLayer::new().allow_origin(Any).allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
        ]))
        .fallback(handler_404)
        .with_state(state)
        ;

    let addr = SocketAddr::from(([0, 0, 0, 0], 8008));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    Ok(())
}

fn api_router() -> Router<Arc<IState>> {
    // This is the order that the modules were authored in.
    Router::new().merge(demo::router())
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
