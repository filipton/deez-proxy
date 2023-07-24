#[no_mangle]
pub async extern "C" fn test(a: i32, b: i32) -> i32 {
    println!("Hello, world! {} {}", a, b);

    return a + b;
}
