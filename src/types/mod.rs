use axum::http::{HeaderMap, StatusCode};
use axum::response::Redirect;

use crate::Message;

pub mod enums;

#[allow(dead_code)]
pub type HandlerResult<T> = Result<T, crate::AppError>;
#[allow(dead_code)]
pub type RedirectResponse = (StatusCode, HeaderMap, ());
#[allow(dead_code)]
pub type RedirectResult = HandlerResult<Redirect>;
#[allow(dead_code)]
pub type MessageResult<T> = HandlerResult<Message<T>>;

#[derive(Clone)]
pub struct IState {
    pub db_pool: sqlx::Pool<sqlx::MySql>,
    pub redis_pool: bb8::Pool<bb8_redis::RedisConnectionManager>,
    pub redis_db: Option<usize>,
}
