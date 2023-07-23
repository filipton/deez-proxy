use color_eyre::eyre::Result;

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

pub trait OptionExt<T> {
    fn to_res(self, error_msg: &'static str) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn to_res(self, error_msg: &'static str) -> Result<T> {
        match self {
            Some(val) => Ok(val),
            None => return Err(color_eyre::eyre::eyre!(error_msg)),
        }
    }
}

#[inline(always)]
pub fn register_script(
    script: &'static str,
    name: &'static str,
    scope: &mut v8::TryCatch<v8::HandleScope>,
) -> Result<()> {
    let filename = v8::String::new(scope, name).to_res("Failed to create new string")?;
    let source_map_url = v8::undefined(scope);
    let origin = v8::ScriptOrigin::new(
        scope,
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

    let script = v8::String::new(scope, script).to_res("Failed to create new string")?;

    let compile_res = v8::Script::compile(scope, script, Some(&origin));
    if let Some(compile_res) = compile_res {
        let _ = compile_res.run(scope);
    } else {
        report_exceptions(scope)?;
    }

    Ok(())
}

pub fn report_exceptions(try_catch: &mut v8::TryCatch<v8::HandleScope>) -> Result<()> {
    let exception = try_catch.exception().to_res("Failed to get exception!")?;
    let exception_string = exception
        .to_string(try_catch)
        .to_res("Failed to convert exception to string!")?
        .to_rust_string_lossy(try_catch);
    let message = if let Some(message) = try_catch.message() {
        message
    } else {
        eprintln!("{}", exception_string);
        return Ok(());
    };

    // Print (filename):(line number): (message).
    let filename = message.get_script_resource_name(try_catch).map_or_else(
        || "(unknown)".into(),
        |s| {
            s.to_string(try_catch)
                .unwrap()
                .to_rust_string_lossy(try_catch)
        },
    );
    let line_number = message.get_line_number(try_catch).unwrap_or_default();

    eprintln!("{}:{}: {}", filename, line_number, exception_string);

    // Print line of source code.
    let source_line = message
        .get_source_line(try_catch)
        .map(|s| {
            s.to_string(try_catch)
                .unwrap()
                .to_rust_string_lossy(try_catch)
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
        .to_string(try_catch)
        .map(|s| s.to_rust_string_lossy(try_catch));

    if let Some(stack_trace) = stack_trace {
        eprintln!("{}", stack_trace);
    }

    Ok(())
}
