use serde::{Deserialize, Serialize};

use crate::types::enums::LinkType;

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

impl LinkHistory {
    pub fn from_url(id: i64, origin_url: String, link_hash: String) -> Self {
        Self {
            id,
            origin_url,
            link_type: Some(LinkType::INTERIM.to_value()),
            expire_date: None,
            active: true,
            link_hash,
            create_time: None,
            update_time: None,
        }
    }
}
