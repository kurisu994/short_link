use diesel::prelude::*;
use crate::link_history;

#[derive(serde::Deserialize, Insertable)]
#[diesel(table_name = link_history)]
struct LinkHistory<'a> {
    id: i64,
    origin_url: &'a str,
    link_type: Option<i32>,
    expire_date: Option<chrono::NaiveDateTime>,
    active: bool,
    link_hash: &'a str,
}