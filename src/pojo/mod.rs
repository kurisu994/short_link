use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Message<T>
    where
        T: Serialize,
{
    code: i32,
    data: Option<T>,
    success: bool,
    result: String,
}

impl<T: Serialize> Message<T> {
    #[allow(dead_code)]
    pub fn ok(data: T) -> Self {
        Message {
            code: 0,
            result: "ok".to_owned(),
            data: Some(data),
            success: true,
        }
    }
    #[allow(dead_code)]
    pub fn failed(message: &str) -> Self {
        Message {
            code: -1,
            result: message.to_owned(),
            data: None,
            success: false,
        }
    }
}

impl<T> IntoResponse for Message<T>
    where
        T: Serialize,
{
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Message::<String>::failed(&self.0.to_string()),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
    where
        E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
