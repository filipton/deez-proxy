use crate::{structs::V8Response, JOB_QUEUE};
use deno_core::{op2, Extension};

mod console;
mod fetch;
mod others;

deno_core::extension!(
    runtime,
    deps = [console, others, fetch],
    ops = [op_callback],
    esm = [ dir "js", "entry.js"],
);

deno_core::extension!(
    runtime_entry,
    deps = [runtime],
    esm_entry_point = "ext:runtime/entry.js",
);

#[op2(async)]
async fn op_callback(
    job_id: u32,
    #[serde] response: V8Response,
) -> Result<(), deno_core::error::AnyError> {
    JOB_QUEUE
        .send_response(job_id, response)
        .await
        .expect("Failed to send response");

    Ok(())
}

pub fn get_all_extensions() -> Vec<Extension> {
    vec![
        others::others::init_ops_and_esm(),
        console::console::init_ops_and_esm(),
        fetch::fetch::init_ops_and_esm(),
        // MUST BE LAST
        runtime::init_ops_and_esm(),
        runtime_entry::init_ops_and_esm(),
    ]
}
