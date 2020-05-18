extern crate base62;
extern crate cfg_if;
extern crate js_sys;
extern crate rand;
extern crate url;
extern crate wasm_bindgen;
extern crate web_sys;

use cfg_if::cfg_if;
use mime_guess::{from_path, Mime};
use mime_guess::mime;
use rand::Rng;
use url::Url;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, Response, ResponseInit};

use kv_store::{ShortUrlAssets, ShortUrlData, ShortUrlUser};

use crate::bean::{ExpireSetting, from_json, js_value_from_json, NewShortUrlRequest, NewShortUrlResponse, ShortUrlDataEntity, to_json_str, UserEntity};
use crate::error::{gen_error, method_not_allowed, NEED_AUTH, not_found, not_found_err, server_error, TTL_ERROR, URL_ERROR};

mod kv_store;
mod utils;
mod error;
mod bean;
mod setting;

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
pub fn need_cache(url: &str) -> bool {
    let url = match Url::parse(url) {
        Ok(url) => { url }
        Err(_) => { return false; }
    };
    let path = url.path();
    let mime_option = from_path(path).first();
    if let Some(mime) = mime_option {
        return mime != mime::TEXT_HTML;
    };
    return false;
}

#[wasm_bindgen]
pub async fn handle(request: Request) -> Result<Response, JsValue> {
    let url = Url::parse(&request.url())
        .map_err(|e| { server_error(&e) })?;
    let mut path_string = url.path().to_string();
    if path_string.ends_with("/") {
        path_string += "index.html";
    }
    let path = path_string.as_str();
    let mime = from_path(path).first();
    if let Some(mime) = mime {
        return get_assert_data(path, mime).await;
    };
    let method: String = request.method().to_lowercase();
    return match path {
        "/new" => match method.as_str() {
            "post" => new_short_url(request).await,
            _ => method_not_allowed(),
        },
        _ => match method.as_str() {
            "get" => try_redirect(path).await,
            _ => not_found()
        }
    };
}

async fn get_assert_data(path: &str, mime: Mime) -> Result<Response, JsValue> {
    let js_value = JsFuture::from(ShortUrlAssets::get(path)).await?;
    let body_str = js_value.as_string().ok_or_else(|| not_found_err())?;
    return gen_str_response_with_content_type(Some(&body_str), mime);
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
    let user_option = get_user(request.headers()).await?;

    let body_str = load_str(&request).await?;
    let params: NewShortUrlRequest = from_json(&body_str)?;

    let raw_url = params.url;
    let target_url = Url::parse(&raw_url)
        .map_err(|e| { gen_error(&e.to_string(), 400, URL_ERROR) })?;
    if target_url.cannot_be_a_base() {
        return Err(gen_error("target url syntax error", 400, URL_ERROR));
    };
    let expire_time_seconds = match params.ttl {
        None => { 0 }
        Some(ttl) => { ttl }
    };
    if expire_time_seconds == 0 || expire_time_seconds > setting::GUEST_MAX_TTL {
        if user_option.is_none() {
            return Err(gen_error("Need Auth", 401, NEED_AUTH));
        }
    }
    if expire_time_seconds != 0 && expire_time_seconds < 60 {
        return Err(gen_error("The TTL must be greater than 60 seconds.", 400, TTL_ERROR));
    }
    let mut short_url_id = gen_short_url_id();
    while check_exists(&short_url_id).await {
        short_url_id = gen_short_url_id();
    }
    let expire_time = js_sys::Date::now() as u64 + expire_time_seconds * 1000;
    let username = match user_option {
        None => { "".to_string() }
        Some(user) => { user.username.to_string() }
    };
    let data = ShortUrlDataEntity {
        raw_url: raw_url.clone(),
        username,
        insert_time: js_sys::Date::now() as u64,
        expire_time,
    };
    let entity_str = to_json_str(data);
    if expire_time_seconds == 0 {
        JsFuture::from(ShortUrlData::put(&short_url_id, &entity_str)).await?;
    } else {
        let expiration = expire_time / 1000;
        let expire_setting = ExpireSetting { expiration };
        let value = js_value_from_json(&expire_setting)?;
        JsFuture::from(ShortUrlData::put_with_ttl(&short_url_id, &entity_str, &value)).await?;
    };

    let url = Url::parse(&request.url()).map_err(|e| { server_error(&e) })?;
    let host = url.host_str().unwrap_or_else(|| { "" });
    let short_url = format!("{}/{}", host, short_url_id);

    let res = NewShortUrlResponse { short_url, raw_url };
    let res_str = to_json_str(res);
    gen_json_response(Some(&res_str))
}

async fn get_user(headers: Headers) -> Result<Option<UserEntity>, JsValue> {
    let key_option: Option<String> = headers.get("X-AUTH-KEY")?;
    let key = match key_option {
        Some(val) => val,
        None => return Ok(None)
    };
    let result = JsFuture::from(ShortUrlUser::get(&key)).await?;
    let user_str_option = result.as_string();
    return match user_str_option {
        None => { Ok(None) }
        Some(user_str) => {
            let entity = from_json(&user_str)?;
            Ok(Some(entity))
        }
    };
}

async fn load_str(request: &Request) -> Result<String, JsValue> {
    let body = JsFuture::from(request.text()?).await?;
    let body_str = body.as_string().ok_or_else(|| { JsValue::from_str("Not Found") })?;
    Ok(body_str)
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
    let random_number: u64 = rng.gen_range(setting::SHORT_ID_MIN, setting::SHORT_ID_MAX);
    let id_str = base62::encode(random_number);
    id_str
}

fn gen_json_response(message: Option<&str>) -> Result<Response, JsValue> {
    let headers = Headers::new()?;
    headers.append("Content-Type", "application/json")?;
    gen_str_response_with_status(message, headers)
}

fn gen_str_response_with_content_type(message: Option<&str>, mime: Mime) -> Result<Response, JsValue> {
    let headers = Headers::new()?;
    headers.append("Content-Type", mime.essence_str())?;
    if mime != mime::TEXT_HTML {
        headers.append("Cache-Control", setting::CACHE_AGE_VALUE)?;
    }
    gen_str_response_with_status(message, headers)
}

fn gen_str_response_with_status(message: Option<&str>, headers: Headers) -> Result<Response, JsValue> {
    let mut response_init = ResponseInit::new();
    response_init.status(200);
    response_init.headers(&headers);
    Response::new_with_opt_str_and_init(message, &response_init)
}
