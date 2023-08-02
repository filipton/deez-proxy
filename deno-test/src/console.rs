use deno_core::{error::AnyError, op};

deno_core::extension!(console, ops = [op_test_console], esm = ["01_console.js"]);

#[op]
pub fn op_test_console() -> Result<(), AnyError> {
    println!("Hello from Rust!");

    Ok(())
}
