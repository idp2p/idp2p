use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn to_arr(bytes: &[u8]) -> usize {
    let arr: [u8;1000] = bytes.try_into().unwrap();
    arr.len()
}

fn to_arr2(bytes: &[u8])-> usize{
    let arr: &[u8;1000] = bytes.try_into().unwrap();
    arr.len()
}

fn criterion_benchmark(c: &mut Criterion) {
    let bytes = vec![0u8;1000];
    c.bench_function("to_arr", |b| {
        b.iter(|| black_box(to_arr2(&bytes)))
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
