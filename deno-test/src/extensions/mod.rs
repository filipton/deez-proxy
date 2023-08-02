use deno_core::Extension;
mod console;

deno_core::extension!(
    runtime,
    deps = [console],
    esm = [ dir "js", "00_entry.js"],
);

deno_core::extension!(
    runtime_entry,
    //ops = [],
    deps = [runtime],
    esm_entry_point = "ext:runtime/00_entry.js",
);

pub fn get_all_extensions() -> Vec<Extension> {
    vec![
        console::console::init_ops_and_esm(),
        runtime::init_ops_and_esm(),
        runtime_entry::init_ops_and_esm(),
    ]
}
