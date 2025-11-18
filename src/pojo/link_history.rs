use serde::{Deserialize, Serialize};

use crate::types::enums::LinkType;
use crate::utils::helper::encode_base62;

#[derive(sqlx::FromRow, Deserialize, Debug)]
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

#[derive(Serialize, Debug)]
pub struct LinkHistoryResponse {
    pub id: i64,
    pub origin_url: String,
    pub link_type: Option<i32>,
    pub expire_date: Option<i64>,
    pub active: bool,
    pub link_hash: String,
    pub create_time: Option<i64>,
    pub update_time: Option<i64>,
    pub link_code: String,
}

#[derive(Serialize, Debug)]
pub struct LinkListResponse {
    pub data: Vec<LinkHistoryResponse>,
    pub page: usize,
    pub page_size: usize,
    pub total: i64,
    pub last_page: bool,
}

impl LinkHistory {
    pub fn from_url(id: i64, origin_url: &str, link_hash: String) -> Self {
        Self {
            id,
            origin_url: origin_url.to_string(),
            link_type: Some(LinkType::INTERIM.to_value()),
            expire_date: None,
            active: true,
            link_hash,
            create_time: None,
            update_time: None,
        }
    }

    pub fn to_response(&self) -> LinkHistoryResponse {
        LinkHistoryResponse {
            id: self.id,
            origin_url: self.origin_url.clone(),
            link_type: self.link_type,
            expire_date: self.expire_date.map(|dt| dt.and_utc().timestamp_millis()),
            active: self.active,
            link_hash: self.link_hash.clone(),
            create_time: self.create_time.map(|dt| dt.and_utc().timestamp_millis()),
            update_time: self.update_time.map(|dt| dt.and_utc().timestamp_millis()),
            link_code: encode_base62(self.id as usize),
        }
    }
}
