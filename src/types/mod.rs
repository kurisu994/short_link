use axum::{
    http::{HeaderMap, StatusCode},
};
use crate::Message;

#[allow(dead_code)]
pub type HandlerResult<T> = Result<T, crate::AppError>;
#[allow(dead_code)]
pub type RedirectResponse = (StatusCode, HeaderMap, ());
#[allow(dead_code)]
pub type RedirectResult = HandlerResult<RedirectResponse>;
#[allow(dead_code)]
pub type MessageResult<T> = HandlerResult<Message<T>>;
