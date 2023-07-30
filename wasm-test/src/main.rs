use color_eyre::Result;
use wasmer::{Cranelift, EngineBuilder, Instance, Module, Store, Value};

#[tokio::main]
async fn main() -> Result<()> {
    let mut store = Store::default();
    let module = Module::new(
        &store,
        include_bytes!("../wasix/target/wasm32-wasmer-wasi/debug/wasix.wasm"),
    )?;

    let import_object = wasmer::imports! {};
    let instance = Instance::new(&mut store, &module, &import_object)?;

    let test = instance.exports.get_function("test")?;
    let result = test.call(&mut store, &[Value::I32(65), Value::I32(4)])?;

    println!("Result: {:?}", result);

    Ok(())
}
