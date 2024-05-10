use std::collections::BinaryHeap;

use super::key::Key;
use crate::{Distance, PeerId, K_VALUE};

#[derive(Debug, Clone)]
pub struct KBucketsTable {
    local_key: Key,
    buckets: Vec<Vec<PeerId>>,
}

impl KBucketsTable {
    pub fn new(local_key: &Key) -> Self {
        Self {
            local_key: local_key.clone(),
            buckets: vec![],
        }
    }

    pub fn local_key(&self) -> &Key {
        &self.local_key
    }

    pub fn local_closest_peers(&self, key: &Key, count: usize) -> Vec<PeerId> {
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
            } else if Key::from_peer_id(peer_id).distance(key) < heap.peek().unwrap().dist {
                heap.pop();
                heap.push(HeapItem { dist, peer_id });
            }
        }
        heap.into_iter().map(|item| item.peer_id).collect()
    }

    pub fn add_peer(&mut self, peer_id: PeerId) {
        let key = Key::from_peer_id(peer_id);
        if key == &self.local_key {
            return;
        }
        let pos = self.local_key.distance(key).leading_zeros() as usize;
        if self.buckets.len() <= pos {
            self.buckets.resize(pos + 1, Vec::with_capacity(K_VALUE));
        }
        let bucket = &mut self.buckets[pos];
        let pos = bucket.iter().position(|&id| id == peer_id);
        match pos {
            Some(idx) => {
                bucket.remove(idx);
                bucket.push(peer_id);
            }
            None => {
                if bucket.len() < K_VALUE {
                    bucket.push(peer_id);
                } else {
                    // ping the least recently seen node and replace if not responded
                }
            }
        }
    }
}
