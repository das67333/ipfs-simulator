use criterion::{criterion_group, criterion_main, Criterion};
use sha2::{Digest, Sha256};

fn func<const N: usize>(c: &mut Criterion) {
    c.bench_function(&format!("func_{}", N), |b| {
        b.iter(|| {
            for i in 0..N {
                Sha256::digest(&i.to_le_bytes());
            }
        })
    });
}

criterion_group!(benches, func::<2>);
criterion_main!(benches);
