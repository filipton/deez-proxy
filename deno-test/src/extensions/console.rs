use deno_core::{error::AnyError, op2};
deno_core::extension!(
    console,
    ops = [op_test_console],
    esm = [ dir "js", "01_console.js"]
);

#[op2(fast)]
pub fn op_test_console() -> Result<(), AnyError> {
    println!("Hello from Rust!");

    Ok(())
}
