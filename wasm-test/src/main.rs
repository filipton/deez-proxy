use color_eyre::Result;
use wasmer::{Store, Module, Instance, Value};

#[tokio::main]
async fn main() -> Result<()> {
    let mut store = Store::default();
    let module = Module::new(&store, include_bytes!("../wasm/pkg/wasm_bg.wasm"))?;

    let import_object = wasmer::imports! {};
    let instance = Instance::new(&mut store, &module, &import_object)?;

    let test = instance.exports.get_function("test")?;
    let result = test.call(&mut store, &[Value::I32(65), Value::I32(4)])?;

    println!("Result: {:?}", result);

    Ok(())
}
