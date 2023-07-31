use color_eyre::Result;
use wasmer::{Cranelift, EngineBuilder, Instance, Module, Store, Value};

#[tokio::main]
async fn main() -> Result<()> {
    let bytes = include_bytes!("../wasix/test.wasmu");

    let mut store = Store::new(Cranelift::default());
    let module = Module::new(&store, bytes)?;

    let import_object = wasmer::imports! {};
    let instance = Instance::new(&mut store, &module, &import_object)?;

    let test = instance.exports.get_function("test")?;
    let result = test.call(&mut store, &[Value::I32(65), Value::I32(4)])?;

    println!("Result: {:?}", result);

    Ok(())
}
