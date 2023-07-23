use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fnv::FnvHashMap;
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref TEST_FILE_2: Vec<u8> = include_bytes!("../../2m").to_vec();
    static ref TEST_FILE_50: Vec<u8> = include_bytes!("../../50m").to_vec();
}

fn bench_hashmap_std_2(mut hashmap: HashMap<Vec<u8>, u64>) {
    hashmap.insert(TEST_FILE_2.clone(), 1069);
}

fn bench_hashmap_fnv_2(mut hashmap: FnvHashMap<Vec<u8>, u64>) {
    hashmap.insert(TEST_FILE_2.clone(), 1069);
}

fn bench_hashmap_rustc_hash_2(mut hashmap: rustc_hash::FxHashMap<Vec<u8>, u64>) {
    hashmap.insert(TEST_FILE_2.clone(), 1069);
}

fn bench_hashmap_ccl_2(hashmap: dashmap::DashMap<Vec<u8>, u64>) {
    hashmap.insert(TEST_FILE_2.clone(), 1069);
}

fn bench_hashmap_std_50(mut hashmap: HashMap<Vec<u8>, u64>) {
    hashmap.insert(TEST_FILE_50.clone(), 1069);
}

fn bench_hashmap_fnv_50(mut hashmap: FnvHashMap<Vec<u8>, u64>) {
    hashmap.insert(TEST_FILE_50.clone(), 1069);
}

fn bench_hashmap_rustc_hash_50(mut hashmap: rustc_hash::FxHashMap<Vec<u8>, u64>) {
    hashmap.insert(TEST_FILE_50.clone(), 1069);
}

fn bench_hashmap_ccl_50(hashmap: dashmap::DashMap<Vec<u8>, u64>) {
    hashmap.insert(TEST_FILE_50.clone(), 1069);
}

fn criterion_benchmark(c: &mut Criterion) {
    let hashmap = HashMap::new();
    c.bench_function("STD 2MB", |b| {
        b.iter(|| bench_hashmap_std_2(black_box(hashmap.clone())))
    });
    c.bench_function("STD 50MB", |b| {
        b.iter(|| bench_hashmap_std_50(black_box(hashmap.clone())))
    });

    let fnv_hashmap = FnvHashMap::default();
    c.bench_function("FNV 2MB", |b| {
        b.iter(|| bench_hashmap_fnv_2(black_box(fnv_hashmap.clone())))
    });
    c.bench_function("FNV 50MB", |b| {
        b.iter(|| bench_hashmap_fnv_50(black_box(fnv_hashmap.clone())))
    });

    let rustc_hashmap = rustc_hash::FxHashMap::default();
    c.bench_function("rustc_hash 2MB", |b| {
        b.iter(|| bench_hashmap_rustc_hash_2(black_box(rustc_hashmap.clone())))
    });
    c.bench_function("rustc_hash 50MB", |b| {
        b.iter(|| bench_hashmap_rustc_hash_50(black_box(rustc_hashmap.clone())))
    });

    let ccl_hashmap = dashmap::DashMap::default();
    c.bench_function("ccl 2MB", |b| {
        b.iter(|| bench_hashmap_ccl_2(black_box(ccl_hashmap.clone())))
    });
    c.bench_function("ccl 50MB", |b| {
        b.iter(|| bench_hashmap_ccl_50(black_box(ccl_hashmap.clone())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
