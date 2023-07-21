use anyhow::Result;

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
struct TestStruct {
    what: String,
    whats: String,
    age: u8,
}

fn main() -> Result<()> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);

    let scope = &mut v8::ContextScope::new(scope, context);
    let global = context.global(scope);

    // CONSOLE //
    let console_key = v8::String::new(scope, "console").unwrap();
    let console_val = v8::Object::new(scope);
    global.set(scope, console_key.into(), console_val.into());

    set_func(scope, console_val, "log", console_log);
    set_func(scope, console_val, "debug", console_debug);
    set_func(scope, console_val, "error", console_error);

    // GLOBALS //
    set_func(scope, global, "fetch", fetch);

    let code = r#"
        async function run(a, b) {
            try {
                let fetchRes = await fetch("https://google.com");
                console.log("fetchRes", fetchRes);
                console.log(await fetchRes.text(1));
                console.log(await fetchRes.json(1, 2, "31231"));
                console.log("a", a);
                console.error({ what: 123, the: 69, fuck: "dsa" });
                console.error([1,3,6,1,3213,35542]);

                return {
                    what: "the fuck",
                    whats: "going on",
                    age: a + b
                };
            } catch(e) {
                console.log(e);
            }
        }
    "#;

    let code = v8::String::new(scope, &format!("{}\nrun", code)).unwrap();
    let script = v8::Script::compile(scope, code, None).unwrap();
    let function = script.run(scope).unwrap();
    let function = v8::Local::<v8::Function>::try_from(function)?;

    let a = v8::Number::new(scope, 5.0).into();
    let b = v8::Number::new(scope, 64.0).into();
    let args = vec![a, b];

    let result = function.call(scope, global.into(), &args).unwrap();
    let promise = v8::Local::<v8::Promise>::try_from(result)?;

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    resolver.resolve(scope, result).unwrap();

    let result = promise.result(scope).to_object(scope);
    if let Some(result) = result {
        let result_res: Result<TestStruct, serde_v8::Error> =
            serde_v8::from_v8(scope, result.into());

        if let Ok(result) = result_res {
            println!("{:?}", result);
        }
    }

    Ok(())
}

#[inline(always)]
pub fn set_func(
    scope: &mut v8::HandleScope,
    obj: v8::Local<v8::Object>,
    name: &str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) {
    let key = v8::String::new(scope, name).unwrap();
    let tmpl = v8::FunctionTemplate::new(scope, callback);
    let val = tmpl.get_function(scope).unwrap();
    val.set_name(key);
    obj.set(scope, key.into(), val.into());
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

// async function fetch(url) {
fn fetch(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let val = v8::Object::new(scope);
    set_func(scope, val, "text", fetch_test);
    set_func(scope, val, "json", fetch_test2);

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
    println!("fetch_test");
    let mut s = String::new();
    for i in 0..args.length() {
        let arg = args.get(i).to_rust_string_lossy(scope);
        s.push_str(&format!("{} ", arg));
    }

    println!("fetch_test: {}", s);
    rv.set(v8::undefined(scope).into());
}

fn fetch_test2(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    println!("fetch_test2");
    let mut s = String::new();
    for i in 0..args.length() {
        let arg = args.get(i).to_rust_string_lossy(scope);
        s.push_str(&format!("{} ", arg));
    }

    println!("fetch_test2: {}", s);
    rv.set(
        v8::String::new(scope, "fetch_test2 return value")
            .unwrap()
            .into(),
    );
}
