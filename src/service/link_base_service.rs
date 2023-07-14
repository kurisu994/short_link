use sqlx::MySql;

use crate::pojo::link_history::LinkHistory;

pub async fn query_by_id(
    m_conn: &sqlx::Pool<MySql>,
    id: u64,
) -> Result<Option<LinkHistory>, crate::AppError> {
    let history_res =
        sqlx::query_as::<_, LinkHistory>("select * from link_history where id = ? and active = 1")
            .bind(id)
            .fetch_optional(m_conn)
            .await?;
    tracing::info!("{:#?}", history_res);
    Ok(history_res)
}

pub async fn query_by_link_hash(
    m_conn: &sqlx::Pool<MySql>,
    link_hash: &str,
) -> Result<Option<LinkHistory>, crate::AppError> {
    let history_res = sqlx::query_as::<_, LinkHistory>(
        "select * from link_history where link_hash = ? and active = 1",
    )
    .bind(link_hash)
    .fetch_optional(m_conn)
    .await?;
    tracing::info!("{:#?}", history_res);
    Ok(history_res)
}

pub async fn save(
    m_conn: &sqlx::Pool<MySql>,
    link_history: LinkHistory,
) -> Result<bool, crate::AppError> {
    let insert_query = r#"
    INSERT INTO link_history (id, origin_url, link_type, expire_date, active, link_hash)
    VALUES (?, ?, ?, ?, ?, ?)
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
