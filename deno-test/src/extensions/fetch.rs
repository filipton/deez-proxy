use deno_core::{error::AnyError, op2};
use std::collections::HashMap;

deno_core::extension!(
    fetch,
    ops = [op_internal_fetch],
    esm = [ dir "js", "fetch.js"]
);

// headers, method, url, buffer
#[op2(async)]
#[buffer]
pub async fn op_internal_fetch(
    #[serde] headers: HashMap<String, String>,
    #[string] method: String,
    #[string] url: String,
    #[buffer(copy)] body: Vec<u8>,
) -> Result<Vec<u8>, AnyError> {
    let method = reqwest::Method::from_bytes(method.as_bytes()).unwrap();
    let client = reqwest::Client::new();
    let mut request_builder = client.request(method, &url);
    for (key, value) in headers {
        request_builder = request_builder.header(key, value);
    }

    let response = request_builder.body(body).send().await?;
    let status = response.status();
    let is_ok = status.is_success();
    let url = response.url().to_string();
    let headers = response
        .headers()
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_str().unwrap().to_string()))
        .collect();

    let body = response.bytes().await?;

    let resp = FetchResponse {
        body: body.to_vec(),
        bodyUsed: false,
        headers,
        ok: is_ok,
        redirected: false,
        status: status.as_u16(),
        statusText: status.canonical_reason().unwrap_or("").to_string(),
        type_: "basic".to_string(),
        url,
    };

    let resp = deno_core::serde_json::to_string(&resp).unwrap();
    let resp = resp.into_bytes();

    Ok(resp)
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
