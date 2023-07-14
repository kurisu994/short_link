use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Debug, Serialize)]
pub struct LinkHistory {
    pub id: i64,
    pub origin_url: String,
    pub link_type: Option<i32>,
    pub expire_date: Option<chrono::NaiveDateTime>,
    pub active: bool,
    pub link_hash: String,
    pub create_time: Option<chrono::NaiveDateTime>,
    pub update_time: Option<chrono::NaiveDateTime>,
}
