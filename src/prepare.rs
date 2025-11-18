use std::env;
use std::sync::Arc;
use std::time::Duration;

use axum::{http::StatusCode, response::IntoResponse};
use bb8_redis::RedisConnectionManager;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::signal;

use crate::config::{Config, Datasource, Driver, Redis};
use crate::types::IState;

pub async fn create_state() -> Arc<IState> {
    let cfg = load_config("application.local.yaml", "application.yaml").unwrap_or_default();
    let redis_db = cfg.redis.database;
    let db_pool = create_db_pool(cfg.datasource).await;
    let redis_pool = create_redis_pool(cfg.redis).await;

    Arc::new(IState {
        db_pool,
        redis_pool,
        redis_db,
    })
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 NOT FOUND")
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

async fn create_db_pool(datasource: Datasource) -> PgPool {
    let mut max_size = datasource.max_pool_size.unwrap_or(2 << 4);
    let min_size = 2 << 2;
    if max_size <= min_size {
        max_size = min_size << 1;
    }
    let idle_timeout = datasource.idle_timeout.unwrap_or(120);
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| datasource.to_link());
    tracing::info!("db_url: {}", db_url);
    let db_pool = PgPoolOptions::new()
        .max_connections(max_size as u32)
        .min_connections(min_size as u32)
        .acquire_timeout(Duration::from_secs(15))
        .idle_timeout(Duration::from_secs(idle_timeout as u64))
        .connect(&db_url)
        .await
        .unwrap();

    db_pool
}

async fn create_redis_pool(redis_cfg: Redis) -> bb8::Pool<RedisConnectionManager> {
    let max_size = redis_cfg.max_size.unwrap_or(10);
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| redis_cfg.to_link());
    tracing::info!("redis_url: {}", redis_url);
    let redis_manager = RedisConnectionManager::new(redis_url).unwrap();
    let redis_pool = bb8::Pool::builder()
        .max_size(max_size as u32)
        .build(redis_manager)
        .await
        .unwrap();

    redis_pool
}

fn load_config(path: &str, default_path: &str) -> Option<Config> {
    let current_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(error) => {
            tracing::error!("Unable to get workspace: {}", error);
            return None;
        }
    };

    let mut file_path = current_dir.join(path);
    // 判断文件是否存在
    if !file_path.exists() {
        file_path = current_dir.join(default_path);
    }
    let cfg_str = match std::fs::read_to_string(file_path) {
        Ok(data) => data,
        Err(err) => {
            tracing::error!("read file failed: {}", err);
            return None;
        }
    };

    match serde_yaml::from_str::<Config>(&cfg_str) {
        Ok(cfg) => Some(cfg),
        Err(err) => {
            tracing::error!("deserialize failed: {}", err);
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prepare::load_config;

    #[test]
    pub fn load_config_test() {
        let file = "application.local.yaml";
        match load_config(file, "application.yaml") {
            None => {
                println!("None");
            }
            Some(config) => {
                println!("{:#?}", config);
            }
        }
    }
}
