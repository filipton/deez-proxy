use crate::utils;

#[inline(always)]
pub fn register(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) {
    let console_key = v8::String::new(scope, "console").unwrap();
    let console_val = v8::Object::new(scope);
    global.set(scope, console_key.into(), console_val.into());

    utils::set_func(scope, console_val, "log", console_log);
    utils::set_func(scope, console_val, "debug", console_debug);
    utils::set_func(scope, console_val, "error", console_error);
}

fn console_log(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let mut s = String::new();
    for i in 0..args.length() {
        let arg = args.get(i).to_rust_string_lossy(scope);
        s.push_str(&format!("{} ", arg));
    }
    println!("LOG: {}", s);
    rv.set(v8::undefined(scope).into());
}

fn console_debug(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let mut s = String::new();
    for i in 0..args.length() {
        let arg = args.get(i).to_rust_string_lossy(scope);
        s.push_str(&format!("{} ", arg));
    }
    println!("DEBUG: {}", s);
    rv.set(v8::undefined(scope).into());
}

fn console_error(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let mut s = String::new();
    for i in 0..args.length() {
        let arg = args.get(i);
        if arg.is_object() {
            let arg = arg.to_object(scope);
            if let Some(arg) = arg {
                let arg: serde_json::Value = serde_v8::from_v8(scope, arg.into()).unwrap();
                let arg = serde_json::to_string_pretty(&arg).unwrap();
                s.push_str(&format!("{} ", arg));
            }
        } else {
            let arg = arg.to_rust_string_lossy(scope);
            s.push_str(&format!("{} ", arg));
        }
    }
    println!("ERROR: {}", s);
    rv.set(v8::undefined(scope).into());
}
