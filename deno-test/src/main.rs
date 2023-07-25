use deno_core::op;
use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::Op;
use deno_core::RuntimeOptions;

mod console;

#[op]
fn op_sum(nums: Vec<f64>) -> Result<f64, deno_core::error::AnyError> {
    // Sum inputs
    let sum = nums.iter().fold(0.0, |a, v| a + v);
    // return as a Result<f64, AnyError>
    Ok(sum)
}

fn main() {
    let start = std::time::Instant::now();

    let ext = Extension::builder("my_ext").ops(vec![op_sum::DECL]).build();
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![ext, console::console::init_ops(), console::console::init_js_only()],
        ..Default::default()
    });

    runtime
        .execute_script_static(
            "main.js",
            r#"
// Print helper function, calling Deno.core.print()
function print(value) {
  Deno.core.print(console);
  Deno.core.print(value.toString()+"\n");
}

const arr = [1, 2, 3];
print("The sum of");
print(arr);
print("is");
print(Deno.core.ops.op_sum(arr));

/*
// And incorrect usage
try {
  print(Deno.core.ops.op_sum(0));
} catch(e) {
  print('Exception:');
  print(e);
}
*/
"#,
        )
        .unwrap();

    println!("Script took {}ms", start.elapsed().as_millis());
}
