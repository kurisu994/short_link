use std::sync::Arc;
use std::time::Duration;

use bb8_redis::redis::{cmd, AsyncCommands, Pipeline};
use chrono::Utc;
use tokio::select;
use tokio::sync::broadcast;

use crate::link_base_service::{query_expired_links, mark_links_as_inactive};
use crate::types::IState;
use crate::utils::helper::calculate_sha256;

const LINK_HASH_KEY: &str = "link:hash:";
const LINK_ID_KEY: &str = "link:origin:uri:";

/// 定时清理过期链接的任务
pub async fn cleanup_expired_links_task(
    state: Arc<IState>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let cleanup_config = &state.cleanup_config;
    let interval_secs = cleanup_config.interval_secs.unwrap_or(3600);
    let interval_duration = Duration::from_secs(interval_secs);
    
    tracing::info!("定时清理任务已启动，清理间隔: {} 秒", interval_secs);
    
    // 首次延迟执行，避免启动时立即触发
    tokio::time::sleep(interval_duration).await;
    
    let mut interval = tokio::time::interval(interval_duration);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    
    loop {
        select! {
            _ = interval.tick() => {
                tracing::debug!("开始执行定时清理任务");
                
                match cleanup_expired_links(state.clone()).await {
                    Ok(count) => {
                        if count > 0 {
                            tracing::debug!("成功清理 {} 条过期链接", count);
                        } else {
                            tracing::debug!("本次清理未发现过期链接");
                        }
                    }
                    Err(e) => {
                        tracing::error!("清理过期链接失败: {}", e);
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                tracing::info!("收到关闭信号，停止定时清理任务");
                break;
            }
        }
    }
    
    tracing::info!("定时清理任务已停止");
}

/// 执行一次清理操作
async fn cleanup_expired_links(state: Arc<IState>) -> Result<usize, crate::AppError> {
    let cleanup_config = &state.cleanup_config;
    let enable_lock = cleanup_config.enable_distributed_lock.unwrap_or(false);
    
    // 如果启用分布式锁，尝试获取锁
    let lock_acquired = if enable_lock {
        match try_acquire_lock(state.clone()).await {
            Ok(acquired) => {
                if !acquired {
                    tracing::debug!("未能获取分布式锁，跳过本次清理（可能其他实例正在执行）");
                    return Ok(0);
                }
                true
            }
            Err(e) => {
                tracing::warn!("获取分布式锁失败: {}，继续执行清理", e);
                false
            }
        }
    } else {
        false
    };
    
    let result = cleanup_expired_links_internal(state.clone()).await;
    
    // 释放分布式锁
    if lock_acquired {
        if let Err(e) = release_lock(state.clone()).await {
            tracing::warn!("释放分布式锁失败: {}", e);
        }
    }
    
    result
}

/// 内部清理逻辑
async fn cleanup_expired_links_internal(state: Arc<IState>) -> Result<usize, crate::AppError> {
    let db_pool = &state.db_pool;
    let batch_size = state.cleanup_config.batch_size.unwrap_or(1000);
    
    let expired_links = query_expired_links(db_pool).await?;
    if expired_links.is_empty() {
        return Ok(0);
    }
    
    let total_count = expired_links.len();
    tracing::info!("发现 {} 条过期链接，开始清理", total_count);
    
    // 分批处理
    let mut total_cleaned = 0;
    for chunk in expired_links.chunks(batch_size) {
        let ids: Vec<i64> = chunk.iter().map(|link| link.id).collect();
        
        let affected = mark_links_as_inactive(db_pool, &ids).await?;
        tracing::debug!("数据库更新完成，本批次影响 {} 行", affected);
        
        let cache_cleaned = cleanup_redis_cache(state.clone(), chunk).await;
        match cache_cleaned {
            Ok(cleaned_count) => {
                tracing::debug!("Redis缓存清理完成，本批次清理 {} 个键", cleaned_count);
                total_cleaned += chunk.len();
            }
            Err(e) => {
                tracing::warn!("Redis缓存清理部分失败: {}，但数据库已更新", e);
                total_cleaned += chunk.len();
            }
        }
    }
    
    // 更新健康检查统计数据
    {
        let mut stats = state.cleanup_stats.write().await;
        stats.last_cleanup_time = Some(Utc::now().timestamp());
        stats.last_cleanup_count = total_cleaned;
        stats.total_cleanup_runs += 1;
        stats.total_cleaned += total_cleaned as u64;
    }
    
    tracing::info!("清理完成，共清理 {} 条过期链接", total_cleaned);
    Ok(total_cleaned)
}

/// 清理Redis缓存（使用Pipeline批量删除）
async fn cleanup_redis_cache(
    state: Arc<IState>,
    expired_links: &[crate::pojo::link_history::LinkHistory],
) -> Result<usize, crate::AppError> {
    let redis_pool = &state.redis_pool;
    let redis_db = state.redis_db.unwrap_or(0);
    
    let mut r_con = redis_pool.get().await?;
    cmd("SELECT").arg(redis_db).query_async::<_, ()>(&mut *r_con).await?;
    
    // 使用Pipeline批量删除
    let mut pipe = Pipeline::new();
    let mut keys_to_delete = Vec::new();
    
    for link in expired_links {
        let link_hash = calculate_sha256(&link.origin_url);
        let hash_key = format!("{}{}", LINK_HASH_KEY, link_hash);
        let id_key = format!("{}{}", LINK_ID_KEY, link.id);
        
        pipe.del(&hash_key);
        pipe.del(&id_key);
        keys_to_delete.push((link.id, hash_key, id_key));
    }
    
    match pipe.query_async::<_, Vec<i32>>(&mut *r_con).await {
        Ok(results) => {
            let total_deleted: i32 = results.iter().sum();
            tracing::debug!("Pipeline批量删除成功，共删除 {} 个键", total_deleted);
            Ok(total_deleted as usize)
        }
        Err(e) => {
            tracing::error!("Pipeline批量删除失败: {}", e);
            // 记录失败的具体键
            for (link_id, hash_key, id_key) in &keys_to_delete {
                tracing::warn!("链接 {} 的缓存键删除失败: {}, {}", link_id, hash_key, id_key);
            }
            Err(e.into())
        }
    }
}

/// 尝试获取分布式锁
async fn try_acquire_lock(state: Arc<IState>) -> Result<bool, crate::AppError> {
    let redis_pool = &state.redis_pool;
    let redis_db = state.redis_db.unwrap_or(0);
    let cleanup_config = &state.cleanup_config;
    
    let lock_key = cleanup_config.lock_key.as_ref()
        .map(|s| s.as_str())
        .unwrap_or("cleanup:lock");
    let lock_timeout = cleanup_config.lock_timeout_secs.unwrap_or(300);
    
    let mut r_con = redis_pool.get().await?;
    cmd("SELECT").arg(redis_db).query_async::<_, ()>(&mut *r_con).await?;
    
    // 使用SET NX EX命令获取锁
    let result: Option<String> = cmd("SET")
        .arg(lock_key)
        .arg("locked")
        .arg("NX")
        .arg("EX")
        .arg(lock_timeout)
        .query_async(&mut *r_con)
        .await?;
    
    Ok(result.is_some())
}

/// 释放分布式锁
async fn release_lock(state: Arc<IState>) -> Result<(), crate::AppError> {
    let redis_pool = &state.redis_pool;
    let redis_db = state.redis_db.unwrap_or(0);
    let cleanup_config = &state.cleanup_config;
    
    let lock_key = cleanup_config.lock_key.as_ref()
        .map(|s| s.as_str())
        .unwrap_or("cleanup:lock");
    
    let mut r_con = redis_pool.get().await?;
    cmd("SELECT").arg(redis_db).query_async::<_, ()>(&mut *r_con).await?;
    
    let _: () = r_con.del(lock_key).await?;
    
    Ok(())
}
