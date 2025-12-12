use std::sync::Arc;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Redirect;
use tokio::sync::RwLock;

use crate::Message;
use crate::config::Cleanup;

pub mod enums;

#[allow(dead_code)]
pub type HandlerResult<T> = Result<T, crate::AppError>;
#[allow(dead_code)]
pub type RedirectResponse = (StatusCode, HeaderMap, ());
#[allow(dead_code)]
pub type RedirectResult = HandlerResult<Redirect>;
#[allow(dead_code)]
pub type MessageResult<T> = HandlerResult<Message<T>>;

/// 清理任务健康检查数据
#[derive(Debug, Clone, Default)]
pub struct CleanupStats {
    /// 最后一次清理时间（Unix时间戳）
    pub last_cleanup_time: Option<i64>,
    /// 最后一次清理数量
    pub last_cleanup_count: usize,
    /// 总清理次数
    pub total_cleanup_runs: u64,
    /// 总清理数量
    pub total_cleaned: u64,
}

#[derive(Clone)]
pub struct IState {
    pub db_pool: sqlx::PgPool,
    pub redis_pool: bb8::Pool<bb8_redis::RedisConnectionManager>,
    pub redis_db: Option<usize>,
    pub cleanup_config: Cleanup,
    pub cleanup_stats: Arc<RwLock<CleanupStats>>,
}
