use std::sync::Arc;

use bb8::PooledConnection;
use bb8_redis::{
    redis::{AsyncCommands, cmd},
    RedisConnectionManager,
};
use sqlx::MySql;

use crate::link_base_service::{query_by_id, query_by_link_hash, save};
use crate::pojo::link_history::LinkHistory;
use crate::types::{HandlerResult, IState};
use crate::utils::helper::{calculate_sha256, encode_base62};

const LINK_HASH_KEY: &'static str = "link:hash:";
const LINK_ID_KEY: &'static str = "link:id:";

pub async fn create_link(
    pool: Arc<IState>,
    link: String,
    _duration: Option<u64>,
) -> HandlerResult<String> {
    let db_pool = &pool.db_pool;
    let redis_pool = &pool.redis_pool;
    let redis_db = pool.redis_db.unwrap_or(0);
    let link_hash = calculate_sha256(&link);

    let mut r_con = redis_pool.get().await?;
    cmd("SELECT").arg(redis_db).query_async(&mut *r_con).await?;
    let id = query_unique_id(&mut r_con, db_pool, link_hash).await?;
    Ok(encode_base62(id as usize))
}

async fn query_unique_id<'a>(
    r_con: &mut PooledConnection<'a, RedisConnectionManager>,
    m_conn: &sqlx::Pool<MySql>,
    link_hash: String,
) -> Result<u64, crate::AppError> {
    let data: Option<String> = r_con.get(format!("{}{}", LINK_HASH_KEY, link_hash)).await?;
    if let Some(id) = data {
        let id: u64 = id.parse()?;
        return Ok(id);
    }
    match query_by_link_hash(m_conn, &link_hash).await? {
        None => {

            Ok(0)
        },
        Some(history) => Ok(history.id as u64),
    }
}

async fn query_exist_short_link<'a>(
    r_con: &mut PooledConnection<'a, RedisConnectionManager>,
    m_conn: &sqlx::Pool<MySql>,
    id: u64,
) -> Result<Option<String>, crate::AppError> {
    let link_id_key = format!("{}{}", LINK_ID_KEY, id);
    let data: Option<String> = r_con.get(&link_id_key).await?;
    if let Some(short_url) = data {
        return Ok(Some(short_url));
    }
    match query_by_id(m_conn, id).await? {
        None => Ok(None),
        Some(history) => {
            let url = encode_base62(history.id as usize);
            Ok(Some(url))
        }
    }
}
