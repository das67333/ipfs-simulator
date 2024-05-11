use std::collections::BinaryHeap;

use super::key::Key;
use crate::{Distance, PeerId, K_VALUE};

/// Represents a Kademlia buckets table.
#[derive(Debug, Clone)]
pub struct KBucketsTable {
    local_key: Key,
    buckets: Vec<Vec<PeerId>>,
}

impl KBucketsTable {
    /// Creates a new instance of `KBucketsTable` with the given local key.
    pub fn new(local_key: &Key) -> Self {
        Self {
            local_key: local_key.clone(),
            buckets: vec![],
        }
    }

    /// Returns a reference to the local key.
    pub fn local_key(&self) -> &Key {
        &self.local_key
    }

    /// Returns a precise list of the closest peers to the given key.
    pub fn local_closest_peers_precise(&self, key: &Key, count: usize) -> Vec<PeerId> {
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        struct HeapItem {
            dist: Distance,
            peer_id: PeerId,
        }

        let mut heap = BinaryHeap::with_capacity(count);
        for &peer_id in self.buckets.iter().flatten() {
            let dist = Key::from_peer_id(peer_id).distance(key);
            if heap.len() < count {
                heap.push(HeapItem { dist, peer_id });
            } else if dist < heap.peek().unwrap().dist {
                heap.pop();
                heap.push(HeapItem { dist, peer_id });
            }
        }
        heap.into_iter().map(|item| item.peer_id).collect()
    }

    /// Returns an approximate list of the closest peers to the given key.
    /// It is expected to be faster than `local_closest_peers_precise`.
    pub fn local_closest_peers_approximate(&self, key: &Key, count: usize) -> Vec<PeerId> {
        if self.buckets.is_empty() {
            return vec![];
        }
        let pos =
            (self.buckets.len() - 1).min(self.local_key.distance(key).leading_zeros() as usize);
        let bucket = &self.buckets[pos];
        // this is usually true
        if count == bucket.len() {
            return bucket.clone();
        }

        if count < bucket.len() {
            let mut copy = bucket.clone();
            copy.sort_by_key(|id| Key::from_peer_id(*id).distance(key));
            return copy.into_iter().take(count).collect();
        }

        let mut result = Vec::with_capacity(count.min(bucket.len() * *K_VALUE));
        let mut i = pos;
        while i < self.buckets.len() && result.len() < count {
            result.extend(self.buckets[i].iter().cloned());
            i += 1;
        }
        i = pos;
        while i != 0 && result.len() < count {
            i -= 1;
            result.extend(self.buckets[i].iter().cloned());
        }
        result.truncate(count);
        result
    }

    /// Adds a peer to the appropriate bucket in the Kademlia buckets table.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to add.
    /// * `on_full` - The action to take when the bucket is full.
    ///
    /// # Returns
    ///
    /// Returns `true` if the peer was successfully added, `false` otherwise.
    pub fn add_peer(&mut self, peer_id: PeerId, on_full: OnFullKBucket) -> bool {
        let key = Key::from_peer_id(peer_id);
        if key == &self.local_key {
            return false;
        }
        let pos = self.local_key.distance(key).leading_zeros() as usize;
        if self.buckets.len() <= pos {
            self.buckets.resize(pos + 1, Vec::with_capacity(*K_VALUE));
        }
        let bucket = &mut self.buckets[pos];
        let pos = bucket.iter().position(|&id| id == peer_id);
        match pos {
            Some(idx) => {
                bucket.remove(idx);
                bucket.push(peer_id);
            }
            None => {
                if bucket.len() < *K_VALUE {
                    bucket.push(peer_id);
                } else {
                    match on_full {
                        OnFullKBucket::Ignore => {}
                        OnFullKBucket::PingLeastRecentlySeen => {
                            // ping the least recently seen node and replace if not responded
                        }
                        OnFullKBucket::ReplaceLeastRecentlySeen => {
                            bucket[*K_VALUE - 1] = peer_id;
                        }
                        OnFullKBucket::ForceReplace(prev) => {
                            if let Some(idx) = bucket.iter().position(|&id| id == prev) {
                                bucket[idx] = peer_id;
                            }
                        }
                    }
                    // ping the least recently seen node and replace if not responded
                }
            }
        }
        true
    }
}

pub enum OnFullKBucket {
    Ignore,
    PingLeastRecentlySeen,
    ReplaceLeastRecentlySeen,
    ForceReplace(PeerId),
}
