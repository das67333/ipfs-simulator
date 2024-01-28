use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn dev1(c: &mut Criterion) {
    let f = |a: &str, b: &str| {
        let mut s = a.to_owned();
        s.extend(b.chars());
        s
    };
    c.bench_function("dev1", |b| b.iter(|| f(black_box("hello"), black_box("world"))));
}

fn dev2(c: &mut Criterion) {
    let f = |a: &str, b: &str| {
        format!("{a}{b}")
    };
    c.bench_function("dev2", |b| b.iter(|| f(black_box("hello"), black_box("world"))));
}

fn dev3(c: &mut Criterion) {
    let f = |a: &str, b: &str| {
        a.to_owned().push_str(b)
    };
    c.bench_function("dev3", |b| b.iter(|| f(black_box("hello"), black_box("world"))));
}

criterion_group!(benches, dev1, dev2, dev3);
criterion_main!(benches);
