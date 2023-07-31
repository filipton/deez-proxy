use deno_core::op;
use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::Op;
use deno_core::RuntimeOptions;

mod console;

#[op]
async fn op_sum(nums: Vec<f64>) -> Result<f64, deno_core::error::AnyError> {
    // Sum inputs
    let sum = nums.iter().fold(0.0, |a, v| a + v);
    // return as a Result<f64, AnyError>
    Ok(sum)
}

#[tokio::main]
async fn main() {
    let ext = Extension::builder("my_ext").ops(vec![op_sum::DECL]).build();
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![ext],
        ..Default::default()
    });

    let start = std::time::Instant::now();
    let res = runtime
        .execute_script_static(
            "main.js",
            r#"
            async function test() {
                Deno.core.print("DBG: LOL\n");
                Deno.core.print("DBG: LOL2\n");
                return {
                    test: 69
                };
            }
            test
            "#,
        )
        .unwrap();

    while let Err(e) = runtime.run_event_loop(true).await {
        println!("Error: {}", e);
    }

    let scope = &mut runtime.handle_scope();
    let local = deno_core::v8::Local::new(scope, res);
    let function = deno_core::v8::Local::<deno_core::v8::Function>::try_from(local).unwrap();
    //let deserialized: deno_core::serde_json::Value = deno_core::serde_v8::from_v8(scope, local).unwrap();

    //println!("Result: {:?}", deserialized);
    let test = deno_core::v8::undefined(scope).into();
    let result = function.call(scope, test, &[]).unwrap();

    let promise = deno_core::v8::Local::<deno_core::v8::Promise>::try_from(result).unwrap();

    let resolver = deno_core::v8::PromiseResolver::new(scope).unwrap();
    resolver.resolve(scope, result).unwrap();

    let mut promise_time = 0;
    while promise.state() == deno_core::v8::PromiseState::Pending {
        std::thread::sleep(std::time::Duration::from_millis(1));
        promise_time += 1;

        if promise_time > 5000 {
            println!("Promise timed out");
            break;
        }
    }

    let result = promise.result(scope).to_object(scope);
    if let Some(result) = result {
        let result_res: Result<deno_core::serde_json::Value, deno_core::serde_v8::Error> =
            deno_core::serde_v8::from_v8(scope, result.into());

        println!("Result: {:?}", deno_core::serde_json::to_string(&result_res.unwrap()).unwrap());
    }

    println!("Script took {}", start.elapsed().as_micros());
}
