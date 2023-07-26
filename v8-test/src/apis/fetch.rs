use std::{collections::HashMap, str::FromStr};

use crate::utils;
use color_eyre::Result;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

#[inline(always)]
pub fn register(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> Result<()> {
    let mut scope = v8::TryCatch::new(scope);

    utils::set_func(&mut scope, global, "__internal_fetch", __internal_fetch);
    utils::register_script(include_str!("./js/fetch.js"), "fetch.js", &mut scope)?;

    Ok(())
}

fn __internal_fetch(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let request: FetchRequest = serde_v8::from_v8(scope, args.get(0)).unwrap();
    let body = args.get(1);
    let array = v8::Local::<v8::ArrayBuffer>::try_from(body).unwrap();
    println!("Array: {:?}", array.byte_length());

    //let data = args.get(1).to_object(scope).unwrap();

    /*
    let buf = v8::ArrayBuffer::new_backing_store_from_boxed_slice(buf);
    let buf = v8::ArrayBuffer::with_backing_store(scope, &buf.into());
    let val = v8::Int8Array::new(scope, buf, 0, buf.byte_length()).unwrap();
    */

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    rv.set(promise.into());

    /*
    resolver.resolve(scope, val.into()).unwrap();
    */

    let mut headers: HeaderMap = HeaderMap::new();
    for (k, v) in request.headers {
        headers.insert(
            HeaderName::from_str(&k).unwrap(),
            HeaderValue::from_str(&v).unwrap(),
        );
    }

    let response = reqwest::blocking::Client::new()
        .request(
            reqwest::Method::from_bytes(request.method.as_bytes()).unwrap(),
            request.url,
        )
        .headers(headers)
        //.body(request.body.unwrap_or_default())
        .send()
        .unwrap();

    let headers = response.headers().clone();
    let is_ok = response.status().is_success();
    let status = response.status();
    let url = response.url().clone();

    let response = FetchResponse {
        body: response.bytes().unwrap().to_vec(),
        bodyUsed: false,
        headers: headers
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap().to_string()))
            .collect(),
        ok: is_ok,
        redirected: false,
        status: status.as_u16(),
        statusText: status.canonical_reason().unwrap_or_default().to_string(),
        type_: "basic".to_string(),
        url: url.to_string(),
    };

    let response = serde_v8::to_v8(scope, &response).unwrap();
    resolver.resolve(scope, response.into()).unwrap();
}

#[derive(serde::Deserialize, Debug)]
#[allow(non_snake_case, dead_code)]
struct FetchRequest {
    headers: HashMap<String, String>,
    method: String,
    url: String,
}

#[derive(serde::Deserialize, Debug)]
#[allow(non_snake_case, dead_code)]
struct FetchHeaders {
    headers: HashMap<String, String>,
}

#[derive(serde::Serialize, Debug)]
#[allow(non_snake_case, dead_code)]
struct FetchResponse {
    body: Vec<u8>,
    bodyUsed: bool,
    headers: HashMap<String, String>,
    ok: bool,
    redirected: bool,
    status: u16,
    statusText: String,

    #[serde(rename = "type")]
    type_: String,

    url: String,
}
