use std::error::Error;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::Response;

use crate::bean::to_json_str;

pub const SERVER_ERROR: i16 = -1;
pub const NOT_FOUND: i16 = -2;
pub const METHOD_NOT_ALLOWED: i16 = -3;
pub const NEED_AUTH: i16 = -4;

pub const URL_ERROR: i16 = 101;
pub const TTL_ERROR: i16 = 102;

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

pub fn not_found() -> Result<Response, JsValue> {
    Err(not_found_err())
}

pub fn method_not_allowed() -> Result<Response, JsValue> {
    Err(gen_error("Method Not Allowed", 415, METHOD_NOT_ALLOWED))
}

pub fn server_error(e: &dyn Error) -> JsValue {
    gen_error(&e.to_string(), 500, SERVER_ERROR)
}

pub fn not_found_err() -> JsValue {
    gen_error("Not Found", 404, NOT_FOUND)
}

pub fn gen_error(message: &str, status: u16, error_code: i16) -> JsValue {
    let js_error = JsError::build(message.to_owned(), status, error_code);
    let err_str = to_json_str(&js_error);
    JsValue::from_str(&err_str)
}
