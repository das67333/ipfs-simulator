use crate::PeerId;
use dslab_core::SimulationContext;
use lazy_static::lazy_static;
use sha2::{Digest, Sha256};
use uint::*;

construct_uint! {
    pub(super) struct U256(4);
}

/// A `Key` in the DHT keyspace with preserved preimage.
///
/// Keys in the DHT keyspace identify both the participating nodes, as well as
/// the records stored in the DHT.
#[derive(Clone, PartialEq, Eq)]
pub struct Key(U256);

/// A distance between two keys in the DHT keyspace.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Distance(U256);

lazy_static! {
    static ref SHA2_POOL: Vec<Key> = (0..100_000)
        .map(|id: PeerId| { Key(U256::from(Sha256::digest(id.to_le_bytes()).as_slice())) })
        .collect();
}

impl Key {
    /// Generates a random key using the given simulation context.
    pub fn random(ctx: &SimulationContext) -> Self {
        let bytes = [0; 32].map(|_| (ctx.gen_range(0..=u8::MAX)));
        Self(U256::from_little_endian(&bytes))
    }

    /// A static reference to the key that was lazily generated on the first
    /// call to this function.
    pub fn from_peer_id(peer_id: PeerId) -> &'static Self {
        &SHA2_POOL[peer_id as usize]
    }

    /// Calculates the distance between two keys using the XOR metric.
    pub fn distance(&self, other: &Key) -> Distance {
        Distance(self.0 ^ other.0)
    }

    /// Returns the key that is uniquely determined by the given distance to `self`.
    pub fn for_distance(&self, dist: Distance) -> Self {
        Self(self.0 ^ dist.0)
    }
}

impl serde::Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{:0>64x}", self.0).serialize(serializer)
    }
}

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:0>64x}", self.0)
    }
}

impl<'de> serde::Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Key, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let inner = s.parse::<U256>().map_err(serde::de::Error::custom)?;
        Ok(Key(inner))
    }
}

impl Distance {
    /// Returns the number of leading zeros in the binary representation of the self.
    pub fn leading_zeros(&self) -> u32 {
        self.0.leading_zeros()
    }
}

impl std::ops::Not for Distance {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
