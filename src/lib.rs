extern crate base62;
extern crate cfg_if;
extern crate js_sys;
extern crate rand;
extern crate url;
extern crate wasm_bindgen;
extern crate web_sys;

use cfg_if::cfg_if;
use js_sys::Promise;
use serde::{Deserialize, Serialize};
use url::Url;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, Response, ResponseInit};
use rand::Rng;

mod utils;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
extern "C" {
    type ShortUrlData;

    #[wasm_bindgen(static_method_of = ShortUrlData)]
    fn get(key: &str) -> Promise;

    #[wasm_bindgen(static_method_of = ShortUrlData)]
    fn put(key: &str, val: &str) -> Promise;

    #[wasm_bindgen(static_method_of = ShortUrlData)]
    fn delete(key: &str) -> Promise;
}

#[wasm_bindgen]
extern "C" {
    type ShortUrlSetting;

    #[wasm_bindgen(static_method_of = ShortUrlSetting)]
    fn get(key: &str, data_type: &str) -> Promise;

    #[wasm_bindgen(static_method_of = ShortUrlSetting)]
    fn put(key: &str, val: &str) -> Promise;

    #[wasm_bindgen(static_method_of = ShortUrlSetting)]
    fn delete(key: &str) -> Promise;
}

#[wasm_bindgen]
extern "C" {
    type ShortUrlUser;

    #[wasm_bindgen(static_method_of = ShortUrlUser)]
    fn get(key: &str, data_type: &str) -> Promise;

    #[wasm_bindgen(static_method_of = ShortUrlUser)]
    fn put(key: &str, val: &str) -> Promise;

    #[wasm_bindgen(static_method_of = ShortUrlUser)]
    fn delete(key: &str) -> Promise;
}

#[derive(Serialize, Deserialize)]
struct NewShortUrlRequest {
    url: String
}

#[derive(Serialize, Deserialize)]
struct NewShortUrlResponse {
    short_url: String,
    raw_url: String,
}

#[derive(Serialize, Deserialize)]
struct ShortUrlDataEntity {
    raw_url: String,
    username: String,
    insert_time: u64,
    expire_time: u64,
}

#[derive(Serialize, Deserialize)]
struct JsError {
    message: String,
    status: u16,
    error_code: i16,
}

impl JsError {
    fn build(message: String, status: u16, error_code: i16) -> JsError {
        JsError {
            message,
            status,
            error_code,
        }
    }
}

#[wasm_bindgen]
pub async fn handle(request: Request) -> Result<Response, JsValue> {
    let url = Url::parse(&request.url())
        .map_err(|e| { gen_error(&e.to_string(), 500, -1) })?;
    let method = request.method().to_lowercase();
    return match url.path() {
        "/" => match method.as_str() {
            "get" => gen_str_response(Some("Hello World")),
            _ => not_found(),
        },
        "/new" => match method.as_str() {
            "post" => new_short_url(request).await,
            _ => not_found(),
        },
        _ => match method.as_str() {
            "get" => try_redirect(url.path()).await,
            _ => not_found()
        }
    };
}

async fn try_redirect(path: &str) -> Result<Response, JsValue> {
    let short_url_id = path.trim_start_matches('/');

    let entity_js_value = JsFuture::from(ShortUrlData::get(short_url_id)).await?;
    let entity_str = entity_js_value.as_string().ok_or_else(|| not_found_err())?;

    let entity = serde_json::from_str::<ShortUrlDataEntity>(&entity_str)
        .map_err(|_| { not_found_err() })?;
    let raw_url = entity.raw_url;

    Response::redirect(&raw_url)
}

async fn new_short_url(request: Request) -> Result<Response, JsValue> {
    let body = JsFuture::from(request.text()?).await?;
    let body_str = body.as_string().ok_or_else(|| { JsValue::from_str("Not Found") })?;
    let params = serde_json::from_str::<NewShortUrlRequest>(&body_str).map_err(
        |e| { gen_error(&e.to_string(), 400, 100) })?;

    let raw_url = params.url;
    let target_url = Url::parse(&raw_url)
        .map_err(|e| { gen_error(&e.to_string(), 400, 101) })?;
    if target_url.cannot_be_a_base() {
        return Err(gen_error("target url syntax error", 400, 101));
    };
    let mut short_url_id = gen_short_url_id();
    while check_exists(&short_url_id).await {
        short_url_id = gen_short_url_id();
    }
    let data = ShortUrlDataEntity {
        raw_url: raw_url.clone(),
        username: String::from(""),
        insert_time: js_sys::Date::now() as u64,
        expire_time: 0,
    };
    let entity_str = to_json_str(data);
    JsFuture::from(ShortUrlData::put(&short_url_id, &entity_str)).await?;

    let url = Url::parse(&request.url())
        .map_err(|e| { gen_error(&e.to_string(), 500, -1) })?;
    let host = url.host_str().unwrap_or_else(|| { "" });
    let short_url = format!("{}/{}", host, short_url_id);

    let res = NewShortUrlResponse { short_url, raw_url };
    let res_str = to_json_str(res);
    gen_json_response(Some(&res_str))
}

async fn check_exists(short_url_id: &str) -> bool {
    let result = JsFuture::from(ShortUrlData::get(short_url_id)).await;
    return match result {
        Ok(value) => {
            !value.is_null() && !value.is_undefined()
        }
        Err(_) => {
            false
        }
    };
}

fn gen_short_url_id() -> String {
    let mut rng = rand::thread_rng();
    let random_number: u64 = rng.gen_range(15_000_000, 3_500_000_000_000);
    let id_str = base62::encode(random_number);
    id_str
}

fn gen_error(message: &str, status: u16, error_code: i16) -> JsValue {
    let js_error = JsError::build(message.to_owned(), status, error_code);
    let err_str = to_json_str(&js_error);
    JsValue::from_str(&err_str)
}

fn to_json_str<T>(obj: T) -> String
    where T: Serialize {
    let err_str = match serde_json::to_string(&obj) {
        Ok(str) => { str }
        Err(err) => { err.to_string() }
    };
    err_str
}

fn not_found() -> Result<Response, JsValue> {
    Err(not_found_err())
}

fn not_found_err() -> JsValue {
    gen_error("Not Found", 404, -2)
}

fn gen_str_response(message: Option<&str>) -> Result<Response, JsValue> {
    let headers = Headers::new()?;
    headers.append("Content-Type", "text/html")?;
    gen_str_response_with_status(message, headers)
}

fn gen_json_response(message: Option<&str>) -> Result<Response, JsValue> {
    let headers = Headers::new()?;
    headers.append("Content-Type", "application/json")?;
    gen_str_response_with_status(message, headers)
}

fn gen_str_response_with_status(message: Option<&str>, headers: Headers) -> Result<Response, JsValue> {
    let mut response_init = ResponseInit::new();
    response_init.status(200);
    response_init.headers(&headers);
    Response::new_with_opt_str_and_init(message, &response_init)
}
