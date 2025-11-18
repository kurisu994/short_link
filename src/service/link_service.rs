use std::sync::Arc;

use bb8::PooledConnection;
use bb8_redis::{
    redis::{cmd, AsyncCommands},
    RedisConnectionManager,
};
use tokio::join;

use crate::idgen::YitIdHelper;
use crate::link_base_service::{query_by_id, query_by_link_hash, save, query_all_with_pagination, count_total_links};
use crate::pojo::link_history::{LinkHistory, LinkHistoryResponse, LinkListResponse};
use crate::pojo::{AppError, Pagination};
use crate::types::{HandlerResult, IState};
use crate::utils::helper::{calculate_sha256, decode_base62, encode_base62};

const LINK_HASH_KEY: &'static str = "link:hash:";
const LINK_ID_KEY: &'static str = "link:origin:uri:";

// 缓存过期时间配置
const CACHE_TTL_SECONDS: i64 = 3600; // URL缓存1小时过期
const HASH_CACHE_TTL_SECONDS: i64 = 86400; // 哈希缓存24小时过期

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
    match query_by_id(db_pool, id as i64).await? {
        None => Err(AppError::from(anyhow::anyhow!("invalid short link"))),
        Some(history) => {
            let url = history.origin_url.clone();
            // 设置缓存，如果键不存在则设置
            let set_result: bool = r_con.set_nx(&link_id_key, &url).await.unwrap_or(false);
            if set_result {
                // 只有设置成功时才设置过期时间
                let expire_result: () = r_con.expire(&link_id_key, CACHE_TTL_SECONDS).await.unwrap_or(());
                let _ = expire_result; // 显式处理结果
            }
            Ok(history.origin_url)
        }
    }
}

async fn query_and_create<'a>(
    r_con: &mut PooledConnection<'a, RedisConnectionManager>,
    m_conn: &sqlx::PgPool,
    origin_link: String,
) -> Result<u64, AppError> {
    let link_hash = calculate_sha256(&origin_link);
    let key = format!("{}{}", LINK_HASH_KEY, link_hash);

    // 并行查询缓存和数据库
    let (cached_id, db_result) = join!(
        async {
            // 查询缓存
            let data: Option<String> = r_con.get(&key).await.ok();
            data.and_then(|s| s.parse().ok())
        },
        async {
            // 查询数据库
            query_by_link_hash(m_conn, &link_hash).await.ok()
        }
    );

    // 如果缓存命中，直接返回
    if let Some(id) = cached_id {
        return Ok(id);
    }

    // 处理数据库查询结果
    match db_result.flatten() {
        None => {
            // 数据库中不存在，创建新记录
            let id = YitIdHelper::next_id();
            let db = LinkHistory::from_url(id, &origin_link, link_hash);
            assert!(save(m_conn, db).await?, "生成短链失败");
            if let Err(err) = set_cache(r_con, key, id, origin_link).await {
                tracing::error!("设置缓存失败: {}", err);
            }
            Ok(id as u64)
        }
        Some(history) => {
            // 数据库中存在，设置缓存并返回
            let id = history.id;
            if let Err(err) = set_cache(r_con, key, id, origin_link).await {
                tracing::error!("设置缓存失败: {}", err);
            }
            Ok(id as u64)
        }
    }
}

async fn set_cache<'a>(
    r_con: &mut PooledConnection<'a, RedisConnectionManager>,
    key: String,
    id: i64,
    origin_link: String,
) -> Result<(), anyhow::Error> {
    // 设置哈希缓存
    let _: () = r_con.set(&key, id).await?;
    let _: () = r_con.expire(&key, HASH_CACHE_TTL_SECONDS).await?;

    // 设置URL缓存
    let url_key = format!("{}{}", LINK_ID_KEY, id);
    let _: () = r_con.set(&url_key, &origin_link).await?;
    let _: () = r_con.expire(&url_key, CACHE_TTL_SECONDS).await?;

    Ok(())
}

pub async fn get_link_list(
    pool: Arc<IState>,
    pagination: Pagination,
) -> HandlerResult<LinkListResponse> {
    let db_pool = &pool.db_pool;

    // 先查询总数
    let total = count_total_links(db_pool).await?;

    // 如果总数为0，直接返回空结果，避免执行不必要的分页查询
    if total == 0 {
        return Ok(LinkListResponse {
            data: vec![],
            page: pagination.page,
            page_size: pagination.page_size,
            total: 0,
            last_page: true,
        });
    }

    // 获取分页数据
    let links = query_all_with_pagination(db_pool, &pagination).await?;
    let response_links: Vec<LinkHistoryResponse> = links.into_iter().map(|link| link.to_response()).collect();

    // 计算是否为最后一页
    let total_pages = ((total as f64) / (pagination.page_size as f64)).ceil() as usize;
    let last_page = pagination.page >= total_pages;

    Ok(LinkListResponse {
        data: response_links,
        page: pagination.page,
        page_size: pagination.page_size,
        total,
        last_page,
    })
}
