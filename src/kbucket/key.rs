use crate::{PeerId, KEYS_POOL, PEER_ID_BY_KEY};
use dslab_core::SimulationContext;
use std::collections::HashSet;
use uint::*;

construct_uint! {
    pub(super) struct U256(4);
}

/// A `Key` in the DHT keyspace with preserved preimage.
///
/// Keys in the DHT keyspace identify both the participating nodes, as well as
/// the records stored in the DHT.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Key(U256);

/// A distance between two keys in the DHT keyspace.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Distance(U256);

impl Key {
    /// Generates a random key using the given simulation context.
    pub fn random(ctx: &SimulationContext) -> Self {
        let bytes = [0; 32].map(|_| (ctx.gen_range(0..=u8::MAX)));
        Self(U256::from_little_endian(&bytes))
    }

    /// Creates a `Key` from the SHA256 hash of the given bytes.
    pub fn from_sha256(bytes: &[u8]) -> Self {
        use sha2::{Digest, Sha256};
        Self(U256::from(Sha256::digest(bytes).as_slice()))
    }

    /// Returns a static reference to the key that was lazily generated.
    pub fn from_peer_id(peer_id: PeerId) -> &'static Self {
        &KEYS_POOL[peer_id as usize]
    }

    /// Generates a random key in the bucket at the given index.
    pub fn random_in_bucket(ctx: &SimulationContext, local_key: Key, index: usize) -> Self {
        let result = Self::random(ctx).0;
        let mask = U256::MAX >> index;
        let last_bit = U256::from(1) << (255 - index);
        local_key.for_distance(Distance((result & mask) | last_bit))
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

/// A tree structure for efficiently finding the closest keys to a given key.
pub struct KeysTree {
    root: Option<KeysTreeNode>,
}

enum KeysTreeNode {
    Leaf(Option<Key>),
    Inner {
        left: Box<KeysTreeNode>,
        right: Box<KeysTreeNode>,
        size: usize,
    },
}

impl KeysTree {
    /// Creates a new `KeysTree` with the given keys.
    pub fn new(keys: &[Key]) -> Self {
        let mut tree = Self { root: None };
        for key in keys {
            tree.insert(key.clone());
        }
        tree
    }

    /// Inserts a key into the tree.
    pub fn insert(&mut self, key: Key) {
        fn inner(node: KeysTreeNode, key: &Key, bit_pos: usize) -> KeysTreeNode {
            match node {
                KeysTreeNode::Leaf(leaf_key) => {
                    if let Some(leaf_key) = leaf_key {
                        if leaf_key.0.bit(bit_pos) != key.0.bit(bit_pos) {
                            if key.0.bit(bit_pos) {
                                KeysTreeNode::Inner {
                                    left: Box::new(KeysTreeNode::Leaf(Some(leaf_key))),
                                    right: Box::new(KeysTreeNode::Leaf(Some(key.clone()))),
                                    size: 2,
                                }
                            } else {
                                KeysTreeNode::Inner {
                                    left: Box::new(KeysTreeNode::Leaf(Some(key.clone()))),
                                    right: Box::new(KeysTreeNode::Leaf(Some(leaf_key))),
                                    size: 2,
                                }
                            }
                        } else if key.0.bit(bit_pos) {
                            KeysTreeNode::Inner {
                                left: Box::new(KeysTreeNode::Leaf(None)),
                                right: Box::new(inner(
                                    KeysTreeNode::Leaf(Some(leaf_key)),
                                    key,
                                    bit_pos - 1,
                                )),
                                size: 2,
                            }
                        } else {
                            KeysTreeNode::Inner {
                                left: Box::new(inner(
                                    KeysTreeNode::Leaf(Some(leaf_key)),
                                    key,
                                    bit_pos - 1,
                                )),
                                right: Box::new(KeysTreeNode::Leaf(None)),
                                size: 2,
                            }
                        }
                    } else {
                        KeysTreeNode::Leaf(Some(key.clone()))
                    }
                }
                KeysTreeNode::Inner { left, right, size } => {
                    if key.0.bit(bit_pos) {
                        KeysTreeNode::Inner {
                            left,
                            right: Box::new(inner(*right, key, bit_pos - 1)),
                            size: size + 1,
                        }
                    } else {
                        KeysTreeNode::Inner {
                            left: Box::new(inner(*left, key, bit_pos - 1)),
                            right,
                            size: size + 1,
                        }
                    }
                }
            }
        }

        match self.root.take() {
            None => {
                self.root = Some(KeysTreeNode::Leaf(Some(key)));
            }
            Some(root) => {
                self.root = Some(inner(root, &key, 255));
            }
        }
    }

    /// Finds the closest keys to the given key in the tree.
    ///
    /// Returns `count` closest keys, if possible.
    pub fn find_closest_keys(&self, key: &Key, count: usize) -> Vec<Key> {
        let mut ans;
        let node = match self.root.as_ref() {
            None => return vec![],
            Some(root) => {
                let mut bit_pos = 255;
                let mut node = root;
                loop {
                    match node {
                        KeysTreeNode::Leaf(_) => unreachable!(),
                        KeysTreeNode::Inner { left, right, size } => {
                            let next = if key.0.bit(bit_pos) {
                                right.as_ref()
                            } else {
                                left.as_ref()
                            };
                            if match next {
                                KeysTreeNode::Leaf(_) => true,
                                KeysTreeNode::Inner { size, .. } => *size < count,
                            } {
                                ans = Vec::with_capacity(*size);
                                break node;
                            }
                            node = next;
                            bit_pos -= 1;
                        }
                    }
                }
            }
        };

        fn inner(node: &KeysTreeNode, ans: &mut Vec<Key>) {
            match node {
                KeysTreeNode::Leaf(leaf_key) => {
                    if let Some(leaf_key) = leaf_key {
                        ans.push(leaf_key.clone());
                    }
                }
                KeysTreeNode::Inner { left, right, .. } => {
                    inner(left, ans);
                    inner(right, ans);
                }
            }
        }

        inner(node, &mut ans);
        ans.sort_unstable_by_key(|k| key.distance(k));
        ans.truncate(count);
        ans
    }

    /// Finds the closest peers to the given key in the tree.
    pub fn find_closest_peers(&self, key: &Key, count: usize) -> HashSet<PeerId> {
        self.find_closest_keys(key, count)
            .iter()
            .map(|key| *PEER_ID_BY_KEY.get(key).expect("Got unexpected key"))
            .collect()
    }
}
