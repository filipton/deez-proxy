use crate::utils;
use color_eyre::Result;
use color_eyre::owo_colors::OwoColorize;

#[inline(always)]
pub fn register(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> Result<()> {
    let console_key = v8::String::new(scope, "console").unwrap();
    let console_val = v8::Object::new(scope);
    global.set(scope, console_key.into(), console_val.into());

    utils::set_func(scope, console_val, "log", console_log);
    utils::set_func(scope, console_val, "debug", console_debug);
    utils::set_func(scope, console_val, "error", console_error);
    utils::set_func(scope, console_val, "warn", console_warn);
    utils::set_func(scope, console_val, "info", console_log);

    Ok(())
}

fn console_log(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let mut s = String::new();
    for i in 0..args.length() {
        let arg = args.get(i);
        let arg = arg.to_rust_string_lossy(scope);
        s.push_str(&format!("{} ", arg));
    }

    println!("{}", s);
    rv.set(v8::undefined(scope).into());
}

fn console_warn(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let mut s = String::new();
    for i in 0..args.length() {
        let arg = args.get(i);
        let arg = arg.to_rust_string_lossy(scope);
        s.push_str(&format!("{} ", arg));
    }

    println!("{}", s.yellow());
    rv.set(v8::undefined(scope).into());
}

fn console_debug(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let mut s = String::new();
    for i in 0..args.length() {
        let arg = args.get(i);
        let arg = arg.to_rust_string_lossy(scope);
        s.push_str(&format!("{} ", arg));
    }

    println!("{}", s.blue());
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
        let arg = arg.to_rust_string_lossy(scope);
        s.push_str(&format!("{} ", arg));
    }

    println!("{}", s.red().bold());
    rv.set(v8::undefined(scope).into());
}
