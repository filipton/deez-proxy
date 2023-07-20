use anyhow::Result;

fn main() -> Result<()> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);

    let scope = &mut v8::ContextScope::new(scope, context);
    let global = context.global(scope);

    let code = r#"
        async function run(a, b) {
            return a + b;
        }
    "#;
    let code = v8::String::new(scope, &format!("{}\nrun", code)).unwrap();

    let script = v8::Script::compile(scope, code, None).unwrap();
    let function = script.run(scope).unwrap();
    let function = v8::Local::<v8::Function>::try_from(function).unwrap();

    let a = v8::Number::new(scope, 5.0).into();
    let b = v8::Number::new(scope, 64.0).into();
    let args = vec![a, b];

    let result = function.call(scope, global.into(), &args).unwrap();
    let promise = v8::Local::<v8::Promise>::try_from(result).unwrap();

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    resolver.resolve(scope, result).unwrap();

    println!(
        "result: {}",
        promise.result(scope).to_rust_string_lossy(scope)
    );

    Ok(())
}
