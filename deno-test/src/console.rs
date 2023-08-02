use deno_core::{error::AnyError, op};

deno_core::extension!(console, ops = [op_test_console], esm_entry_point = "ext:console/01_console.js", esm = ["01_console.js"]);

#[op]
pub fn op_test_console() -> Result<(), AnyError> {
    println!("Hello from Rust!");

    Ok(())
}
