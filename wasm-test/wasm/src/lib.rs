use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn test(a: i32, b: i32) -> i32 {
    println!("Hello, world!");
    return a + b;
}
