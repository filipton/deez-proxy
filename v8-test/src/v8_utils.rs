use std::net::SocketAddr;

use crate::utils::OptionExt;
use color_eyre::Result;

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct V8Response {
    pub block_connection: Option<bool>,
    pub hang_connection: Option<bool>,
    pub ip: Option<String>,
    pub no_delay: Option<bool>,

    pub cpu_time: Option<u128>,
}

#[derive(serde::Serialize, Debug)]
#[allow(dead_code)]
pub struct V8Request {
    pub ip: String,
    pub port: u16,
}

pub fn install() {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
}

pub async fn get_script_res(script: &str, port: u16, addr: SocketAddr) -> Result<V8Response> {
    let isolate = &mut v8::Isolate::new(Default::default());
    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);
    let mut scope = v8::TryCatch::new(scope);
    let global = context.global(&mut scope);
    crate::apis::register_all(&mut scope, global)?;

    let code = format!(
        r#"
        async function run(req) {{
            try {{
                return await handle(req);
            }}
            catch (e) {{
                console.error(e.stack);
            }}
        }}

        {}

        run
    "#,
        script
    );

    let code = v8::String::new(&mut scope, &code).to_res("Failed to change code to v8 string!")?;
    let script = match v8::Script::compile(&mut scope, code, None) {
        Some(script) => script,
        None => {
            crate::utils::report_exceptions(&mut scope)?;
            return Err(color_eyre::eyre::eyre!("Error compiling script"));
        }
    };

    let cpu_time_start = cpu_time::ThreadTime::now();
    let function = script.run(&mut scope).to_res("Failed to run script!")?;
    let function = v8::Local::<v8::Function>::try_from(function)?;

    let request = V8Request {
        ip: format!("{}", addr.ip()),
        port,
    };
    let arg = serde_v8::to_v8(&mut scope, request)?.into();

    let result = function
        .call(&mut scope, global.into(), &vec![arg])
        .to_res("Failed to call function!")?;
    let promise = v8::Local::<v8::Promise>::try_from(result)?;

    let resolver = v8::PromiseResolver::new(&mut scope).to_res("Failed to create resolver!")?;
    resolver
        .resolve(&mut scope, result)
        .to_res("Failed to resolve promise!")?;

    let mut promise_time = 0;
    while promise.state() == v8::PromiseState::Pending {
        std::thread::sleep(std::time::Duration::from_millis(1));
        promise_time += 1;

        if promise_time > 5000 {
            color_eyre::eyre::bail!("Promise timed out!");
        }
    }

    let result = promise.result(&mut scope).to_object(&mut scope);
    if let Some(result) = result {
        let result_res: Result<V8Response, serde_v8::Error> =
            serde_v8::from_v8(&mut scope, result.into());

        if let Ok(mut result) = result_res {
            result.cpu_time = Some(cpu_time_start.elapsed().as_micros());
            return Ok(result);
        }
    }

    color_eyre::eyre::bail!("Failed to get result!")
}
