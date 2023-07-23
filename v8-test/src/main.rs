use color_eyre::eyre::Result;
use utils::OptionExt;

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
    color_eyre::install()?;

    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);

    let scope = &mut v8::ContextScope::new(scope, context);
    let mut scope = v8::TryCatch::new(scope);
    let global = context.global(&mut scope);

    apis::console::register(&mut scope, global);
    apis::fetch::register(&mut scope, global);

    let others_code = include_str!("../js/others.js");
    let fetch_code = include_str!("../js/fetch.js");
    let main_code = include_str!("../js/main.js");
    let code = format!("{}\n{}\n{}", others_code, fetch_code, main_code);

    let code = v8::String::new(&mut scope, &format!("{}\nrun", code))
        .to_res("Failed to change code to v8 string!")?;

    let script = match v8::Script::compile(&mut scope, code, None) {
        Some(script) => script,
        None => {
            report_exceptions(scope)?;
            return Err(color_eyre::eyre::eyre!("Error compiling script"));
        }
    };

    let function = script.run(&mut scope).to_res("Failed to run script!")?;
    let function = v8::Local::<v8::Function>::try_from(function)?;

    let a = v8::Number::new(&mut scope, 5.0).into();
    let b = v8::Number::new(&mut scope, 64.0).into();
    let args = vec![a, b];

    let result = function
        .call(&mut scope, global.into(), &args)
        .to_res("Failed to call function!")?;
    let promise = v8::Local::<v8::Promise>::try_from(result)?;

    let resolver = v8::PromiseResolver::new(&mut scope).to_res("Failed to create resolver!")?;
    resolver
        .resolve(&mut scope, result)
        .to_res("Failed to resolve promise!")?;

    let result = promise.result(&mut scope).to_object(&mut scope);
    if let Some(result) = result {
        let result_res: Result<TestStruct, serde_v8::Error> =
            serde_v8::from_v8(&mut scope, result.into());

        if let Ok(result) = result_res {
            println!("{:?}", result);
        }
    }

    Ok(())
}

fn report_exceptions(mut try_catch: v8::TryCatch<v8::HandleScope>) -> Result<()> {
    let exception = try_catch.exception().to_res("Failed to get exception!")?;
    let exception_string = exception
        .to_string(&mut try_catch)
        .to_res("Failed to convert exception to string!")?
        .to_rust_string_lossy(&mut try_catch);
    let message = if let Some(message) = try_catch.message() {
        message
    } else {
        eprintln!("{}", exception_string);
        return Ok(());
    };

    // Print (filename):(line number): (message).
    let filename = message
        .get_script_resource_name(&mut try_catch)
        .map_or_else(
            || "(unknown)".into(),
            |s| {
                s.to_string(&mut try_catch)
                    .unwrap()
                    .to_rust_string_lossy(&mut try_catch)
            },
        );
    let line_number = message.get_line_number(&mut try_catch).unwrap_or_default();

    eprintln!("{}:{}: {}", filename, line_number, exception_string);

    // Print line of source code.
    let source_line = message
        .get_source_line(&mut try_catch)
        .map(|s| {
            s.to_string(&mut try_catch)
                .unwrap()
                .to_rust_string_lossy(&mut try_catch)
        })
        .to_res("Failed to get source line!")?;
    eprintln!("{}", source_line);

    // Print wavy underline (GetUnderline is deprecated).
    let start_column = message.get_start_column();
    let end_column = message.get_end_column();

    for _ in 0..start_column {
        eprint!(" ");
    }

    for _ in start_column..end_column {
        eprint!("^");
    }

    eprintln!();

    // Print stack trace
    let stack_trace = if let Some(stack_trace) = try_catch.stack_trace() {
        stack_trace
    } else {
        return Ok(());
    };
    let stack_trace = unsafe { v8::Local::<v8::String>::cast(stack_trace) };
    let stack_trace = stack_trace
        .to_string(&mut try_catch)
        .map(|s| s.to_rust_string_lossy(&mut try_catch));

    if let Some(stack_trace) = stack_trace {
        eprintln!("{}", stack_trace);
    }

    Ok(())
}
