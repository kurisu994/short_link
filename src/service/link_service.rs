use std::sync::Arc;

use bb8::PooledConnection;
use bb8_redis::{
    redis::{cmd, AsyncCommands},
    RedisConnectionManager,
};
use sqlx::MySql;

use crate::idgen::YitIdHelper;
use crate::link_base_service::{query_by_id, query_by_link_hash, save};
use crate::pojo::link_history::LinkHistory;
use crate::pojo::AppError;
use crate::types::{HandlerResult, IState};
use crate::utils::helper::{calculate_sha256, decode_base62, encode_base62};

const LINK_HASH_KEY: &'static str = "link:hash:";
const LINK_ID_KEY: &'static str = "link:origin:uri:";

pub async fn create_link(
    pool: Arc<IState>,
    link: String,
    _duration: Option<u64>,
) -> HandlerResult<String> {
    let db_pool = &pool.db_pool;
    let redis_pool = &pool.redis_pool;
    let redis_db = pool.redis_db.unwrap_or(0);

    let mut r_con = redis_pool.get().await?;
    cmd("SELECT").arg(redis_db).query_async::<_, ()>(&mut *r_con).await?;
    let id = query_and_create(&mut r_con, db_pool, link).await?;
    Ok(encode_base62(id as usize))
}

pub async fn query_origin_url(pool: Arc<IState>, link_hash: String) -> Result<String, AppError> {
    let id = decode_base62(&link_hash)?;
    let db_pool = &pool.db_pool;
    let redis_pool = &pool.redis_pool;
    let redis_db = pool.redis_db.unwrap_or(0);
    let mut r_con = redis_pool.get().await?;
    cmd("SELECT").arg(redis_db).query_async::<_, ()>(&mut *r_con).await?;

    let link_id_key = format!("{}{}", LINK_ID_KEY, id);
    let data: Option<String> = r_con.get(&link_id_key).await?;
    if let Some(url) = data {
        return Ok(url);
    }
    match query_by_id(db_pool, id as u64).await? {
        None => Err(AppError::from(anyhow::anyhow!("invalid short link"))),
        Some(history) => {
            let url = history.origin_url.clone();
            if let Err(err) = r_con
                .set_nx::<String, String, isize>(link_id_key, url)
                .await
            {
                tracing::error!("cache url failed: {}", err)
            }
            Ok(history.origin_url)
        }
    }
}

async fn query_and_create<'a>(
    r_con: &mut PooledConnection<'a, RedisConnectionManager>,
    m_conn: &sqlx::Pool<MySql>,
    origin_link: String,
) -> Result<u64, AppError> {
    let link_hash = calculate_sha256(&origin_link);
    let key = format!("{}{}", LINK_HASH_KEY, link_hash);
    let data: Option<String> = r_con.get(&key).await?;
    if let Some(id) = data {
        let id: u64 = id.parse()?;
        return Ok(id);
    }

    match query_by_link_hash(m_conn, &link_hash).await? {
        None => {
            let id = YitIdHelper::next_id();
            let db = LinkHistory::from_url(id, &origin_link, link_hash);
            assert!(save(m_conn, db).await?, "生成短链失败");
            let _ = set_cache(r_con, key, id, origin_link).await;
            Ok(id as u64)
        }
        Some(history) => {
            let id = history.id;
            let _ = set_cache(r_con, key, id, origin_link).await;
            Ok(id as u64)
        }
    }
}

async fn set_cache<'a>(
    r_con: &mut PooledConnection<'a, RedisConnectionManager>,
    key: String,
    id: i64,
    origin_link: String,
) {
    let _ = r_con.set::<String, i64, String>(key, id).await;
    let _ = r_con
        .set::<String, String, String>(format!("{}{}", LINK_ID_KEY, id), origin_link)
        .await;
}
