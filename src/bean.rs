use serde::{de, Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::error::gen_error;

#[derive(Serialize, Deserialize)]
pub struct ExpireSetting {
    pub expiration: u64
}

#[derive(Serialize, Deserialize)]
pub struct NewShortUrlRequest {
    pub url: String,
    pub ttl: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct NewShortUrlResponse {
    pub short_url: String,
    pub raw_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct ShortUrlDataEntity {
    pub raw_url: String,
    pub username: String,
    pub insert_time: u64,
    pub expire_time: u64,
}

#[derive(Serialize, Deserialize)]
pub struct UserEntity {
    pub username: String,
    pub api_key: String,
}

pub fn to_json_str<T>(obj: T) -> String where T: Serialize {
    let err_str = match serde_json::to_string(&obj) {
        Ok(str) => { str }
        Err(err) => { err.to_string() }
    };
    err_str
}

pub fn js_value_from_json<T: Serialize>(body: &T) -> Result<JsValue, JsValue> {
    JsValue::from_serde(&body)
        .map_err(|e| { gen_error(&e.to_string(), 500, -4) })
}

pub fn from_json<'a, T: de::Deserialize<'a>>(body_str: &'a String) -> Result<T, JsValue> {
    let params = serde_json::from_str::<T>(&body_str).map_err(
        |e| { gen_error(&e.to_string(), 400, 100) })?;
    Ok(params)
}
