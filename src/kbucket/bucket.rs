use super::key::Key;
use crate::{Distance, PeerId, CONFIG, K_VALUE};
use std::collections::BinaryHeap;

/// Represents a Kademlia buckets table.
#[derive(Debug)]
pub struct KBucketsTable {
    local_key: Key,
    buckets: Vec<Vec<KBucketEntry>>,
}

#[derive(Debug, Clone)]
struct KBucketEntry {
    pub peer_id: PeerId,
    pub last_seen: f64,
}

impl KBucketsTable {
    /// Creates a new instance of `KBucketsTable` with the given local key.
    pub fn new(local_key: &Key) -> Self {
        Self {
            local_key: local_key.clone(),
            buckets: vec![],
        }
    }

    /// Returns the local key.
    pub fn local_key(&self) -> Key {
        self.local_key.clone()
    }

    /// Returns the number of buckets in the Kademlia buckets table.
    pub fn buckets_count(&self) -> usize {
        self.buckets.len()
    }

    /// Returns a precise list of the closest peers to the given key.
    pub fn local_closest_peers_precise(&self, key: &Key, count: usize) -> Vec<PeerId> {
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        struct HeapItem {
            dist: Distance,
            peer_id: PeerId,
        }

        let mut heap = BinaryHeap::with_capacity(count);
        for entry in self.buckets.iter().flatten() {
            let peer_id = entry.peer_id;
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
            return bucket.iter().map(|entry| entry.peer_id).collect();
        }

        if count < bucket.len() {
            let mut copy = bucket.iter().map(|entry| entry.peer_id).collect::<Vec<_>>();
            copy.sort_by_key(|&id| Key::from_peer_id(id).distance(key));
            return copy.into_iter().take(count).collect();
        }

        let mut result = Vec::with_capacity(count.min(bucket.len() * *K_VALUE));
        let mut i = pos;
        while i < self.buckets.len() && result.len() < count {
            result.extend(self.buckets[i].iter().map(|entry| entry.peer_id));
            i += 1;
        }
        i = pos;
        while i != 0 && result.len() < count {
            i -= 1;
            result.extend(self.buckets[i].iter().map(|entry| entry.peer_id));
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
    pub fn add_peer(&mut self, peer_id: PeerId, curr_time: f64) -> bool {
        let key = Key::from_peer_id(peer_id);
        if key == &self.local_key {
            return false;
        }
        let pos = self.local_key.distance(key).leading_zeros() as usize;
        if self.buckets.len() <= pos {
            self.buckets.resize(pos + 1, Vec::with_capacity(*K_VALUE));
        }
        let bucket = &mut self.buckets[pos];
        let pos = bucket.iter().position(|entry| entry.peer_id == peer_id);
        let entry = KBucketEntry {
            peer_id,
            last_seen: curr_time,
        };
        match pos {
            Some(idx) => {
                bucket.remove(idx);
                bucket.push(entry);
            }
            None => {
                if bucket.len() < *K_VALUE {
                    bucket.push(entry);
                    return true;
                }
                let mut idx = None;
                for (i, kb_entry) in bucket.iter().enumerate() {
                    if curr_time - kb_entry.last_seen > CONFIG.kbuckets_refresh_interval {
                        idx = Some(i);
                    }
                }
                if let Some(idx) = idx {
                    bucket.remove(idx);
                    bucket.push(entry);
                }
            }
        }
        true
    }
}
