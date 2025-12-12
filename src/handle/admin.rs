use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::pojo::AppError;
use crate::{
    link_service,
    pojo::{link_history::LinkListResponse, Message, Pagination},
    types::{IState, MessageResult},
};

pub fn router() -> Router<Arc<IState>> {
    Router::new()
        .route("/link/list", get(link_list))
        .route("/link/create", post(create_link))
        .route("/health/cleanup", get(cleanup_health))
}

async fn link_list(
    State(pool): State<Arc<IState>>,
    pagination: Option<Query<Pagination>>,
) -> MessageResult<LinkListResponse> {
    let Query(pagination) = pagination.unwrap_or_default();
    match link_service::get_link_list(pool, pagination).await {
        Ok(link_list_response) => Ok(Message::ok(link_list_response)),
        Err(e) => {
            tracing::error!("查询链接列表失败: {}", e);
            Err(AppError::from(e))
        }
    }
}

#[derive(Deserialize, Validate, Debug)]
#[allow(dead_code)]
struct CreateLink {
    #[validate(url(message = "无效"), required(message = "不能为空"))]
    url: Option<String>,
    duration: Option<u64>,
}

async fn create_link(
    State(pool): State<Arc<IState>>,
    Json(payload): Json<CreateLink>,
) -> MessageResult<String> {
    if let Err(e) = payload.validate() {
        return Err(AppError::from(e));
    }
    let res = link_service::create_link(pool, payload.url.unwrap(), payload.duration).await?;
    Ok(Message::ok(res))
}

/// 清理任务健康检查响应
#[derive(Serialize, Debug)]
struct CleanupHealthResponse {
    /// 状态：healthy, unknown
    status: String,
    /// 最后一次清理时间（Unix时间戳）
    last_cleanup_time: Option<i64>,
    /// 最后一次清理时间（格式化字符串）
    last_cleanup_time_formatted: Option<String>,
    /// 最后一次清理数量
    last_cleanup_count: usize,
    /// 总清理次数
    total_cleanup_runs: u64,
    /// 总清理数量
    total_cleaned: u64,
}

/// 清理任务健康检查端点
async fn cleanup_health(State(state): State<Arc<IState>>) -> MessageResult<CleanupHealthResponse> {
    let stats = state.cleanup_stats.read().await;
    
    let status = if stats.last_cleanup_time.is_some() {
        "healthy"
    } else {
        "unknown"
    };
    
    let last_cleanup_time_formatted = stats.last_cleanup_time.map(|timestamp| {
        chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Invalid timestamp".to_string())
    });
    
    let response = CleanupHealthResponse {
        status: status.to_string(),
        last_cleanup_time: stats.last_cleanup_time,
        last_cleanup_time_formatted,
        last_cleanup_count: stats.last_cleanup_count,
        total_cleanup_runs: stats.total_cleanup_runs,
        total_cleaned: stats.total_cleaned,
    };
    
    Ok(Message::ok(response))
}
