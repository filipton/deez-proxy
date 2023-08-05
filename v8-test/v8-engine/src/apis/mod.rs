use color_eyre::Result;
use v8::{HandleScope, Local, Object, TryCatch};

mod console;
mod fetch;

pub fn register_all(scope: &mut TryCatch<HandleScope>, global: Local<Object>) -> Result<()> {
    console::register(scope, global)?;
    fetch::register(scope, global)?;
    crate::utils::set_func(scope, global, "sleep", __internal_sleep2);

    crate::utils::register_script(include_str!("./js/others.js"), "others.js", scope)?;

    Ok(())
}

fn __internal_sleep(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let delay = args.get(0).number_value(scope).unwrap_or(0.0);
    std::thread::sleep(std::time::Duration::from_millis(delay as u64));
    rv.set(v8::undefined(scope).into());
}

fn __internal_sleep2(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    println!("sleep2");

    let delay = args.get(0).number_value(scope).unwrap_or(0.0);
    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    rv.set(promise.into());
    println!("sleep2 promise");

    let (tx, rx) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();

        let fut = async move {
            println!("sleeping {}ms", delay);
            tokio::time::sleep(std::time::Duration::from_millis(delay as u64)).await;
            println!("sleeping {}ms done", delay);
            tx.send(()).unwrap();
        };

        rt.block_on(fut);
    });

    loop {
        if let Ok(_res) = rx.try_recv() {
            let undefined = v8::undefined(scope);
            resolver.resolve(scope, undefined.into()).unwrap();

            break;
        }
        std::thread::sleep(std::time::Duration::from_micros(100));
    }
}
