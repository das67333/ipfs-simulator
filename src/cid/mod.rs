mod hashing;
mod ipfs_cid;
mod multicodec;

const MAX_HASH_LEN: usize = 64;

pub use cid::{multibase::Base as Multibase, Version as CidVersion};
pub use hashing::MultihashType;
pub use ipfs_cid::IpfsCid;
pub use multicodec::Multicodec;
