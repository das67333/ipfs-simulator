use criterion::{criterion_group, criterion_main, Criterion};
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha20Rng,
};
use rsa::{RsaPrivateKey, RsaPublicKey};

const SEED: u64 = 42;

fn rng_chacha(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::seed_from_u64(SEED);
    c.bench_function("rng_chacha_u64", |b| b.iter(|| rng.next_u64()));
}

fn rsa_generate_keys<const RSA_BITS: usize>(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::seed_from_u64(SEED);
    c.bench_function(&format!("rsa_generate_keys_{RSA_BITS}"), |b| {
        b.iter(|| {
            let priv_key =
                RsaPrivateKey::new(&mut rng, RSA_BITS).expect("Failed to create RSA key");
            let pub_key = RsaPublicKey::from(&priv_key);
            (priv_key, pub_key)
        })
    });
}

criterion_group!(
    benches,
    rng_chacha,
    rsa_generate_keys<2048>,
    rsa_generate_keys<1024>,
    rsa_generate_keys<512>,
    rsa_generate_keys<256>,
    rsa_generate_keys<128>,
    rsa_generate_keys<64>,
);
criterion_main!(benches);
