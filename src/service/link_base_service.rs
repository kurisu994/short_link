use crate::pojo::link_history::LinkHistory;
use crate::pojo::Pagination;

pub async fn query_by_id(
    m_conn: &sqlx::PgPool,
    id: i64,
) -> Result<Option<LinkHistory>, crate::AppError> {
    let history_res =
        sqlx::query_as::<_, LinkHistory>("select * from link_history where id = $1 and active = true")
            .bind(id)
            .fetch_optional(m_conn)
            .await?;
    Ok(history_res)
}

pub async fn query_by_link_hash(
    m_conn: &sqlx::PgPool,
    link_hash: &str,
) -> Result<Option<LinkHistory>, crate::AppError> {
    let history_res = sqlx::query_as::<_, LinkHistory>(
        "select * from link_history where link_hash = $1 and active = true",
    )
    .bind(link_hash)
    .fetch_optional(m_conn)
    .await?;
    Ok(history_res)
}

pub async fn save(
    m_conn: &sqlx::PgPool,
    link_history: LinkHistory,
) -> Result<bool, crate::AppError> {
    let insert_query = r#"
    INSERT INTO link_history (id, origin_url, link_type, expire_date, active, link_hash)
    VALUES ($1, $2, $3, $4, $5, $6)
    "#;
    let mut tx = m_conn.begin().await?;
    let result = sqlx::query(insert_query)
        .bind(link_history.id)
        .bind(link_history.origin_url)
        .bind(link_history.link_type)
        .bind(link_history.expire_date)
        .bind(link_history.active)
        .bind(link_history.link_hash)
        .execute(&mut *tx)
        .await;

    match result {
        Ok(res) => {
            tx.commit().await?;
            Ok(res.rows_affected() > 0)
        }
        Err(_) => {
            tx.rollback().await?;
            Ok(false)
        }
    }
}

pub async fn query_all_with_pagination(
    m_conn: &sqlx::PgPool,
    pagination: &Pagination,
) -> Result<Vec<LinkHistory>, crate::AppError> {
    let offset = (pagination.page - 1) * pagination.page_size;
    let limit = pagination.page_size;

    let history_res = sqlx::query_as::<_, LinkHistory>(
        "SELECT * FROM link_history ORDER BY create_time DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(m_conn)
    .await?;

    Ok(history_res)
}

pub async fn count_total_links(m_conn: &sqlx::PgPool) -> Result<i64, crate::AppError> {
    let count: Option<i64> = sqlx::query_scalar("SELECT COUNT(*) FROM link_history WHERE active = true")
        .fetch_one(m_conn)
        .await?;

    Ok(count.unwrap_or(0))
}
