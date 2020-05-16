use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JsError {
    message: String,
    status: u16,
    error_code: i16,
}

impl JsError {
    pub fn build(message: String, status: u16, error_code: i16) -> JsError {
        JsError {
            message,
            status,
            error_code,
        }
    }
}
