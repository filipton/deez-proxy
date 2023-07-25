use crate::utils;
use color_eyre::Result;

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
    let url = args.get(0).to_rust_string_lossy(scope);
    let response = reqwest::blocking::get(url)
        .unwrap()
        .bytes()
        .unwrap()
        .to_vec()
        .into_boxed_slice();

    let resolver = v8::PromiseResolver::new(scope).unwrap();

    let promise = resolver.get_promise(scope);
    rv.set(promise.into());

    let buf = v8::ArrayBuffer::new_backing_store_from_boxed_slice(response);
    let buf = v8::ArrayBuffer::with_backing_store(scope, &buf.into());
    let val = v8::Int8Array::new(scope, buf, 0, buf.byte_length()).unwrap();
    resolver.resolve(scope, val.into()).unwrap();
}
