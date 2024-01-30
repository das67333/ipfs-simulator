use super::MAX_HASH_LEN;
use cid::multihash::{Error, Multihash};
use num_derive::FromPrimitive;
use sha2::Digest;

/// Supported variants:
///
/// https://github.com/multiformats/multicodec/blob/master/table.csv
///
/// (with tag = "multihash" and status = "permanent")
///
/// Currently only `Sha2_*` variants are implemented
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

impl MultihashType {
    // TODO: without heap allocations
    pub fn digest(self, data: &[u8]) -> Result<Multihash<MAX_HASH_LEN>, Error> {
        let digest = match self {
            MultihashType::Identity => data.to_vec(),
            MultihashType::Sha2_224 => {
                let mut hasher = sha2::Sha224::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            MultihashType::Sha2_256 => {
                let mut hasher = sha2::Sha256::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            MultihashType::Sha2_256Trunc254Padded => {
                let mut hasher = sha2::Sha256::new();
                hasher.update(data);
                let mut v = hasher.finalize().to_vec();
                *v.last_mut().unwrap() &= 0b00111111;
                v
            }
            MultihashType::Sha2_384 => {
                let mut hasher = sha2::Sha384::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            MultihashType::Sha2_512 => {
                let mut hasher = sha2::Sha512::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            MultihashType::Sha2_512_224 => {
                let mut hasher = sha2::Sha512_224::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            MultihashType::Sha2_512_256 => {
                let mut hasher = sha2::Sha512_256::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            _ => unimplemented!(),
        };
        Multihash::wrap(self as u64, &digest)
    }
}
