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
    pub fn err(message: &str) -> Self {
        Message {
            code: -1,
            result: message.to_owned(),
            data: None,
            success: false,
        }
    }
}
