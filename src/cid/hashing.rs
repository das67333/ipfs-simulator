use super::MAX_HASH_LEN;
use cid::multihash::Multihash;
use num_derive::FromPrimitive;
use sha2::{Digest, Sha224, Sha256, Sha512, Sha512_224, Sha512_256};
use std::collections::HashMap;

// TODO: without heap allocations
type HashAlg = fn(&[u8]) -> Vec<u8>;

pub struct HashAlgorithms {
    type_to_func: HashMap<MultihashType, HashAlg>,
}

/// Supported variants:
///
/// https://github.com/multiformats/multicodec/blob/master/table.csv
///
/// (with tag = "multihash" and status = "permanent")
///
/// Currently only `Sha2_*` variants are implemented
#[repr(u64)]
#[derive(Debug, PartialEq, Eq, Hash, FromPrimitive)]
pub enum MultihashType {
    Identity = 0x00,
    Sha1 = 0x11,

    Sha2_224 = 0x1013,
    Sha2_256 = 0x12,
    Sha2_256Trunc254Padded = 0x1012,
    Sha2_384 = 0x20,
    Sha2_512 = 0x13,
    Sha2_512_224 = 0x1014,
    Sha2_512_256 = 0x1015,

    Sha3_224 = 0x17,
    Sha3_256 = 0x16,
    Sha3_384 = 0x15,
    Sha3_512 = 0x14,

    Blake2b256 = 0xb220,
    PoseidonBls12_381A2Fc1 = 0xb401,
}

impl HashAlgorithms {
    pub fn new() -> Self {
        let mut type_to_func: HashMap<MultihashType, HashAlg> = HashMap::new();

        type_to_func.insert(MultihashType::Identity, |data| data.to_vec());

        type_to_func.insert(MultihashType::Sha2_224, |data| {
            let mut hasher = Sha224::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        });

        type_to_func.insert(MultihashType::Sha2_256, |data| {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        });

        type_to_func.insert(MultihashType::Sha2_256Trunc254Padded, |data| {
            let mut hasher = Sha256::new();
            hasher.update(data);
            let mut v = hasher.finalize().to_vec();
            *v.last_mut().unwrap() &= 0b00111111;
            v
        });

        type_to_func.insert(MultihashType::Sha2_512, |data| {
            let mut hasher = Sha512::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        });

        type_to_func.insert(MultihashType::Sha2_512_224, |data| {
            let mut hasher = Sha512_224::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        });

        type_to_func.insert(MultihashType::Sha2_512_256, |data| {
            let mut hasher = Sha512_256::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        });

        Self { type_to_func }
    }

    pub fn digest(&self, hash_type: MultihashType, input: &[u8]) -> Multihash<MAX_HASH_LEN> {
        let func = self
            .type_to_func
            .get(&hash_type)
            .expect("This multihash code is either not supported or not implemented");
        Multihash::wrap(hash_type as u64, &func(input)).unwrap()
    }
}
