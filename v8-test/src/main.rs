use anyhow::Result;

mod utils;
mod apis {
    pub mod console;
    pub mod fetch;
}

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

    apis::console::register(scope, global);
    apis::fetch::register(scope, global);

    let others_code = include_str!("../js/others.js");
    let fetch_code = include_str!("../js/fetch.js");
    let main_code = include_str!("../js/main.js");
    let code = format!("{}\n{}\n{}", others_code, fetch_code, main_code);

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
