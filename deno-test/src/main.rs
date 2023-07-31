use deno_core::op;
use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::Op;
use deno_core::RuntimeOptions;

mod console;

#[op]
fn op_sum(nums: Vec<f64>) -> Result<f64, deno_core::error::AnyError> {
    println!("Rust: op_sum {:?}", nums);
    // Sum inputs
    let sum = nums.iter().fold(0.0, |a, v| a + v);
    // return as a Result<f64, AnyError>
    Ok(sum)
}

#[op]
async fn op_sleep(duration: u64) -> Result<(), deno_core::error::AnyError> {
    println!("Sleeping for {}ms", duration);
    tokio::time::sleep(tokio::time::Duration::from_millis(duration)).await;

    Ok(())
}

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

#[tokio::main]
async fn main() {
    let ext = Extension::builder("my_ext")
        .ops(vec![op_sum::DECL, op_sleep::DECL])
        .build();
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![ext],
        ..Default::default()
    });

    let start = std::time::Instant::now();

    let req = V8Request {
        ip: "192.158.1.69".to_string(),
        port: 42069,
    };

    let req = deno_core::serde_json::to_string(&req).unwrap();
    let script = format!(
        r#"
            async function test(req) {{
                Deno.core.print(`DBG: ${{req.ip}} ${{req.port}}\n`);

                let val = Deno.core.ops.op_sum([1,2,3]);
                Deno.core.print(val + "\n");
                Deno.core.print("DBG: LOL\n");
                await Deno.core.ops.op_sleep(1000);
                Deno.core.print("DBG: LOL2\n");
                return {{
                    test: 69
                }};
            }}

            test({}).then((res) => {{
                Deno.core.print("dsa: " + JSON.stringify(res) + "\n");
            }});
            "#,
        req
    );
    let res = runtime.execute_script("main.js", script.into()).unwrap();

    while let Err(e) = runtime.run_event_loop(false).await {
        println!("Error: {}", e);
    }

    let scope = &mut runtime.handle_scope();
    let local = deno_core::v8::Local::new(scope, res);
    let deserialized: deno_core::serde_json::Value =
        deno_core::serde_v8::from_v8(scope, local).unwrap();

    println!("Result: {:?}", deserialized);

    println!("Script took {}", start.elapsed().as_micros());
}
