use super::{
    hashing::{HashAlgorithms, MultihashType},
    multicodec::Multicodec,
    MAX_HASH_LEN,
};
use cid::{multibase::Base as Multibase, multihash::Multihash, CidGeneric, Error, Version};
use num_traits::FromPrimitive;

#[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
pub struct IpfsCid {
    cid: CidGeneric<MAX_HASH_LEN>,
}

impl IpfsCid {
    pub fn from_chunk(
        version: Version,
        codec: Multicodec,
        hash_type: MultihashType,
        chunk: &[u8],
        ha: &HashAlgorithms,
    ) -> Result<Self, Error> {
        CidGeneric::new(version, codec as u64, ha.digest(hash_type, chunk)).map(|cid| Self { cid })
    }

    pub fn from_bytes(b: &[u8]) -> Result<Self, Error> {
        CidGeneric::read_bytes(b).map(|cid| Self { cid })
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        CidGeneric::try_from(s).map(|cid| Self { cid })
    }

    pub fn into_v1(self) -> Result<Self, Error> {
        self.cid.into_v1().map(|cid| Self { cid })
    }

    pub fn multicodec(&self) -> Multicodec {
        let codec = self.cid.codec();
        Multicodec::from_u64(codec).expect(&format!("Invalid multicodec: {codec:#x}"))
    }

    pub fn multihash(&self) -> &Multihash<MAX_HASH_LEN> {
        self.cid.hash()
    }

    pub fn multihash_code(&self) -> MultihashType {
        let code = self.cid.hash().code();
        MultihashType::from_u64(code).expect(&format!("Invalid multihash code: {code:#x}"))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.cid.to_bytes()
    }

    pub fn to_string(&self, base: Multibase) -> Result<String, Error> {
        self.cid.to_string_of_base(base)
    }
}

impl std::fmt::Display for IpfsCid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cid)
    }
}
