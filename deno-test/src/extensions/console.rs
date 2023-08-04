use deno_core::{error::AnyError, op2};
deno_core::extension!(
    console,
    ops = [op_test_console],
    esm = [ dir "js", "console.js"]
);

#[op2(async)]
pub async fn op_test_console() -> Result<(), AnyError> {
    println!("Hello from Rust!");

    tokio::task::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        println!("Hello from Rust2!");
    });

    Ok(())
}
