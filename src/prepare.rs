use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use bb8_redis::RedisConnectionManager;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use tokio::signal;

use crate::types::IState;

pub async fn create_state() -> Arc<IState> {
    let db_pool = create_db_pool().await;
    let redis_pool = create_redis_pool().await;
    // 创建状态对象
    Arc::new(IState {
        db_pool,
        redis_pool,
    })
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 not found")
}

pub async fn shutdown_signal() {
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

    tracing::info!("signal received, starting graceful shutdown");
}

async fn create_db_pool() -> Pool<MySql> {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let db_pool = MySqlPoolOptions::new()
        .max_connections(15)
        .acquire_timeout(Duration::from_secs(15))
        .connect(&db_url)
        .await
        .unwrap();

    db_pool
}

async fn create_redis_pool() -> bb8::Pool<RedisConnectionManager> {
    let redis_url = std::env::var("REDIS_URL").unwrap();
    let redis_manager = bb8_redis::RedisConnectionManager::new(redis_url).unwrap();
    let redis_pool = bb8::Pool::builder()
        .max_size(20)
        .build(redis_manager)
        .await
        .unwrap();

    redis_pool
}
