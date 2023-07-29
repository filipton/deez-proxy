use color_eyre::Result;
use v8::{HandleScope, Local, Object, TryCatch};

mod console;
mod fetch;

pub fn register_all(scope: &mut TryCatch<HandleScope>, global: Local<Object>) -> Result<()> {
    console::register(scope, global)?;
    fetch::register(scope, global)?;
    crate::utils::register_script(include_str!("./js/others.js"), "others.js", scope)?;

    Ok(())
}
