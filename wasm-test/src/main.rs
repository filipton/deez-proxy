use color_eyre::Result;
use wasmer::{Cranelift, EngineBuilder, Instance, Module, Store, Value};

#[tokio::main]
async fn main() -> Result<()> {
    let compiler = Cranelift::default();
    let mut features = wasmer::Features::default();
    features.multi_value(true);

    let engine = EngineBuilder::new(compiler).set_features(Some(features));

    let mut store = Store::new(engine);
    let module = Module::new(
        &store,
        include_bytes!("../wasm/target/wasm32-unknown-unknown/release/wasm.wasm"),
    )?;

    let import_object = wasmer::imports! {};
    let instance = Instance::new(&mut store, &module, &import_object)?;

    let test = instance.exports.get_function("test")?;
    let result = test.call(&mut store, &[Value::I32(65), Value::I32(4)])?;

    println!("Result: {:?}", result);

    Ok(())
}
