use js_sys::Promise;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type ShortUrlAssets;

    #[wasm_bindgen(static_method_of = ShortUrlAssets)]
    pub fn get(key: &str) -> Promise;

    #[wasm_bindgen(static_method_of = ShortUrlAssets)]
    pub fn put(key: &str, val: &str) -> Promise;

    #[wasm_bindgen(static_method_of = ShortUrlAssets)]
    pub fn delete(key: &str) -> Promise;
}

#[wasm_bindgen]
extern "C" {
    pub type ShortUrlData;

    #[wasm_bindgen(static_method_of = ShortUrlData)]
    pub fn get(key: &str) -> Promise;

    #[wasm_bindgen(static_method_of = ShortUrlData)]
    pub fn put(key: &str, val: &str) -> Promise;

    #[wasm_bindgen(static_method_of = ShortUrlData, js_name = put)]
    pub fn put_with_ttl(key: &str, val: &str, ttl: &JsValue) -> Promise;

    #[wasm_bindgen(static_method_of = ShortUrlData)]
    pub fn delete(key: &str) -> Promise;
}

#[wasm_bindgen]
extern "C" {
    pub type ShortUrlUser;

    #[wasm_bindgen(static_method_of = ShortUrlUser)]
    pub fn get(key: &str) -> Promise;

    #[wasm_bindgen(static_method_of = ShortUrlUser)]
    pub fn put(key: &str, val: &str) -> Promise;

    #[wasm_bindgen(static_method_of = ShortUrlUser)]
    pub fn delete(key: &str) -> Promise;
}
