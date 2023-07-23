use color_eyre::Result;
use tokio::net::{TcpListener, TcpStream};
use utils::OptionExt;

mod apis;
mod utils;

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
struct TestStruct {
    what: String,
    whats: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let mut tasks = vec![];

    tasks.push(tokio::spawn(port_worker(7071)));
    tasks.push(tokio::spawn(port_worker(7072)));

    futures::future::try_join_all(tasks).await?;

    Ok(())
}

async fn get_script_res(script: &str) -> Result<TestStruct> {
    let isolate = &mut v8::Isolate::new(Default::default());
    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);
    let mut scope = v8::TryCatch::new(scope);
    let global = context.global(&mut scope);
    apis::register_all(&mut scope, global)?;

    let start = cpu_time::ThreadTime::now();
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
            utils::report_exceptions(&mut scope)?;
            return Err(color_eyre::eyre::eyre!("Error compiling script"));
        }
    };

    let function = script.run(&mut scope).to_res("Failed to run script!")?;
    let function = v8::Local::<v8::Function>::try_from(function)?;

    let a = v8::Number::new(&mut scope, 5.0).into();
    let args = vec![a];

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
            println!("time: {:?}", start.elapsed().as_micros());
            return Ok(result);
        }
    }

    color_eyre::eyre::bail!("Failed to get result!")
}

async fn port_worker(port: u16) -> Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        let socket_res = listener.accept().await;

        match socket_res {
            Ok((socket, _addr)) => {
                tokio::spawn(async move {
                    let code = tokio::fs::read_to_string("./main.js").await.unwrap();

                    if let Err(e) =
                        handle_client(socket, get_script_res(&code).await.unwrap()).await
                    {
                        println!("Handle Client Error: {}", e);
                    }
                });
            }
            Err(e) => {
                println!("Bind Socket Error: {}", e);
            }
        }
    }
}

async fn handle_client(mut socket: TcpStream, output: TestStruct) -> Result<()> {
    println!("Output: {:?}", output);

    let mut out_stream = TcpStream::connect("192.168.1.1:80").await?;
    tokio::io::copy_bidirectional(&mut socket, &mut out_stream).await?;
    Ok(())
}
