use std::sync::Arc;

use bb8::Pool;
use bb8_redis::{
    redis::{AsyncCommands, cmd},
    RedisConnectionManager,
};

use crate::types::{HandlerResult, IState};
use crate::utils::helper::calculate_sha256;

pub async fn create_link(
    pool: Arc<IState>,
    link: String,
    _duration: Option<u64>,
) -> HandlerResult<String> {
    // let db_pool = &pool.db_pool;
    let redis_pool = &pool.redis_pool;
    let redis_db = pool.redis_db.unwrap_or(0);
    let link_hash = calculate_sha256(&link);
    tracing::info!("this link_hash is {}", link_hash);

    let id = query_unique_id(redis_pool, redis_db, link_hash).await?;
    tracing::info!("this id is {:?}", id);
    Ok("".to_string())
}

async fn query_unique_id(
    redis_pool: &Pool<RedisConnectionManager>,
    redis_db: usize,
    link_hash: String,
) -> Result<Option<u64>, crate::AppError> {
    let mut pool = redis_pool.get().await?;
    cmd("SELECT").arg(redis_db).query_async(&mut *pool).await?;
    let data: Option<String> = pool.get(format!("link:hash:{}", link_hash)).await?;
    if let Some(id) = data {
        let id: u64 = id.parse()?;
        return Ok(Some(id));
    }
    Ok(None)
}
