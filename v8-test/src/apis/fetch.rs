use crate::utils::{self, OptionExt, report_exceptions};
use color_eyre::Result;

#[inline(always)]
pub fn register(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> Result<()> {
    let mut scope = v8::TryCatch::new(scope);

    utils::set_func(&mut scope, global, "fetch", fetch);
    utils::set_func(&mut scope, global, "__internal_fetch", __internal_fetch);

    let filename = v8::String::new(&mut scope, "fetch.js").to_res("Failed to create new string")?;
    let source_map_url = v8::undefined(&mut scope);
    let origin = v8::ScriptOrigin::new(
        &mut scope,
        filename.into(),
        0,
        0,
        false,
        0,
        source_map_url.into(),
        false,
        false,
        false,
    );

    let script = v8::String::new(&mut scope, include_str!("../../js/fetch.js"))
        .to_res("Failed to create new string")?;

    let compile_res = v8::Script::compile(&mut scope, script, Some(&origin));
    if let Some(compile_res) = compile_res {
        let _ = compile_res.run(&mut scope);
    } else {
        report_exceptions(scope)?;
    }
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

    let buf = v8::ArrayBuffer::new_backing_store_from_boxed_slice(response);
    let buf = v8::ArrayBuffer::with_backing_store(scope, &buf.into());
    let val = v8::Int8Array::new(scope, buf, 0, buf.byte_length()).unwrap();

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    resolver.resolve(scope, val.into()).unwrap();

    let promise = resolver.get_promise(scope);
    rv.set(promise.into());
}

fn fetch(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let val = v8::Object::new(scope);
    utils::set_func(scope, val, "text", fetch_test);
    utils::set_func(scope, val, "json", fetch_test2);

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    resolver.resolve(scope, val.into()).unwrap();

    let promise = resolver.get_promise(scope);
    rv.set(promise.into());
}

fn fetch_test(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let mut s = String::new();
    for i in 0..args.length() {
        let arg = args.get(i).to_rust_string_lossy(scope);
        s.push_str(&format!("{} ", arg));
    }

    rv.set(v8::undefined(scope).into());
}

fn fetch_test2(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let mut s = String::new();
    for i in 0..args.length() {
        let arg = args.get(i).to_rust_string_lossy(scope);
        s.push_str(&format!("{} ", arg));
    }

    rv.set(
        v8::String::new(scope, "fetch_test2 return value")
            .unwrap()
            .into(),
    );
}
