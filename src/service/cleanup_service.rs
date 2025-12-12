use std::sync::Arc;
use std::time::Duration;

use bb8_redis::redis::{cmd, AsyncCommands};

use crate::link_base_service::{query_expired_links, mark_links_as_inactive};
use crate::types::IState;
use crate::utils::helper::calculate_sha256;

const LINK_HASH_KEY: &str = "link:hash:";
const LINK_ID_KEY: &str = "link:origin:uri:";

/// 定时清理过期链接的任务
pub async fn cleanup_expired_links_task(state: Arc<IState>) {
    let interval_duration = Duration::from_secs(3600); // 每小时执行一次
    let mut interval = tokio::time::interval(interval_duration);
    
    tracing::info!("定时清理任务已启动，清理间隔: {} 秒", interval_duration.as_secs());
    
    loop {
        interval.tick().await;
        
        match cleanup_expired_links(state.clone()).await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("成功清理 {} 条过期链接", count);
                } else {
                    tracing::debug!("本次清理未发现过期链接");
                }
            }
            Err(e) => {
                tracing::error!("清理过期链接失败: {}", e);
            }
        }
    }
}

/// 执行一次清理操作
async fn cleanup_expired_links(state: Arc<IState>) -> Result<usize, crate::AppError> {
    let db_pool = &state.db_pool;
    
    let expired_links = query_expired_links(db_pool).await?;
    if expired_links.is_empty() {
        return Ok(0);
    }
    
    let count = expired_links.len();
    tracing::info!("发现 {} 条过期链接，开始清理", count);
    
    let ids: Vec<i64> = expired_links.iter().map(|link| link.id).collect();
    
    let affected = mark_links_as_inactive(db_pool, &ids).await?;
    tracing::info!("数据库更新完成，影响 {} 行", affected);
    
    let cache_cleaned = cleanup_redis_cache(state.clone(), &expired_links).await;
    match cache_cleaned {
        Ok(cleaned_count) => {
            tracing::info!("Redis缓存清理完成，清理 {} 个键", cleaned_count);
        }
        Err(e) => {
            tracing::warn!("Redis缓存清理部分失败: {}，但数据库已更新", e);
        }
    }
    
    Ok(count)
}

/// 清理Redis缓存
async fn cleanup_redis_cache(
    state: Arc<IState>,
    expired_links: &[crate::pojo::link_history::LinkHistory],
) -> Result<usize, crate::AppError> {
    let redis_pool = &state.redis_pool;
    let redis_db = state.redis_db.unwrap_or(0);
    
    let mut r_con = redis_pool.get().await?;
    cmd("SELECT").arg(redis_db).query_async::<_, ()>(&mut *r_con).await?;
    
    let mut cleaned_count = 0;
    
    for link in expired_links {
        let link_hash = calculate_sha256(&link.origin_url);
        let hash_key = format!("{}{}", LINK_HASH_KEY, link_hash);
        
        let id_key = format!("{}{}", LINK_ID_KEY, link.id);
        
        match r_con.del::<_, i32>(&[&hash_key, &id_key]).await {
            Ok(deleted) => {
                cleaned_count += deleted as usize;
            }
            Err(e) => {
                tracing::warn!("清理链接 {} 的缓存失败: {}", link.id, e);
            }
        }
    }
    
    Ok(cleaned_count)
}
