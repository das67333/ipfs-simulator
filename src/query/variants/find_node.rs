use super::QueryState;
use crate::{
    message::FindNodeRequest,
    query::{QueryId, QueryPool},
    Distance, Key, PeerId, ALPHA_VALUE, CONFIG, K_VALUE,
};
use std::collections::{BinaryHeap, HashSet};

/// Represents a query to find the closest peers to a target key.
///
/// This struct provides methods to create a new query, handle responses from peers,
/// and evaluate the query to calculate the correctness of the results.
pub struct FindNodeQuery {
    target_key: Key,
    peers_all: HashSet<PeerId>,   // waiting + responded + next
    peers_responded: Vec<PeerId>, // sorted by distance to target in descending order
    peers_waiting: Vec<PeerId>,
    peers_next: Vec<PeerId>, // sorted by distance to target in descending order
    state: QueryState<Vec<PeerId>>,
}

impl FindNodeQuery {
    /// Creates a new `FindNodeQuery` and adds it to the query pool.
    ///
    /// # Arguments
    ///
    /// * `queries` - A mutable reference to the query pool.
    /// * `target_key` - The target key to find the closest peers to.
    /// * `self_id` - The ID of the current peer initiating the query.
    ///
    /// # Returns
    ///
    /// A tuple containing the query ID and the `FindNodeRequest` associated with the query.
    /// This request should be sent to the current peer!
    pub fn new_query(
        queries: &mut QueryPool,
        target_key: Key,
        self_id: PeerId,
    ) -> (QueryId, FindNodeRequest) {
        let query_id = queries.add_find_node_query(FindNodeQuery {
            target_key: target_key.clone(),
            peers_all: HashSet::from_iter([self_id]),
            peers_responded: vec![],
            peers_waiting: {
                let mut v = Vec::with_capacity(*ALPHA_VALUE);
                v.push(self_id);
                v
            },
            peers_next: vec![],
            state: QueryState::InProgress,
        });
        (
            query_id,
            FindNodeRequest {
                query_id,
                key: target_key,
            },
        )
    }

    /// Handles a response from a peer.
    ///
    /// # Arguments
    ///
    /// * `src_id` - The ID of the peer that sent the response.
    /// * `query_id` - The ID of the query associated with the response.
    /// * `closest_peers` - A vector of sender's locally closest peers to the target key.
    ///
    /// # Returns
    ///
    /// A vector of pair containing the destination peer ID and the `FindNodeRequest` associated with it.
    pub fn on_response(
        &mut self,
        src_id: PeerId,
        query_id: QueryId,
        closest_peers: Vec<PeerId>,
    ) -> Vec<(PeerId, FindNodeRequest)> {
        if matches!(self.state, QueryState::Completed(_)) {
            return vec![];
        }
        match self.peers_waiting.iter().position(|&id| id == src_id) {
            Some(idx) => {
                self.peers_waiting.swap_remove(idx);
            }
            None => return vec![],
        }
        let key_func = self.key_func();
        match self
            .peers_responded
            .binary_search_by_key(&key_func(&src_id), &key_func)
        {
            Ok(_) => unreachable!("waiting for a peer that has already responded"),
            Err(idx) => {
                self.peers_responded.insert(idx, src_id);
            }
        }

        for &peer_next in closest_peers.iter() {
            if self.peers_all.insert(peer_next) {
                match self
                    .peers_next
                    .binary_search_by_key(&key_func(&peer_next), &key_func)
                {
                    Ok(_) => unreachable!("peers_all and peers_next are inconsistent"),
                    Err(idx) => {
                        self.peers_next.insert(idx, peer_next);
                    }
                }
            }
        }

        if self.check_if_completed() {
            return vec![];
        }
        let mut result = vec![];
        while self.peers_waiting.len() < *ALPHA_VALUE {
            if let Some(peer_id) = self.pop_next_peer() {
                let request = FindNodeRequest {
                    query_id,
                    key: self.target_key.clone(),
                };
                result.push((peer_id, request));
            } else {
                break;
            }
        }
        result
    }

    fn pop_next_peer(&mut self) -> Option<PeerId> {
        let next_peer = self.peers_next.pop();
        if let Some(peer_id) = next_peer {
            self.peers_waiting.push(peer_id);
        }
        next_peer
    }

    fn check_if_completed(&mut self) -> bool {
        let key_func = self.key_func();
        if self.peers_responded.len() >= *K_VALUE {
            if let Some(&peer_id) = self.peers_next.last() {
                let i = self.peers_responded.len() - *K_VALUE;
                if key_func(&peer_id) < key_func(&self.peers_responded[i]) {
                    self.peers_all.clear();
                    self.peers_next.clear();
                    self.peers_waiting.clear();
                    let result = self.peers_responded.split_off(i);
                    self.state = QueryState::Completed(result);

                    assert!(self
                        .peers_responded
                        .windows(2)
                        .all(|w| key_func(&w[0]) < key_func(&w[1])));
                    assert!(self
                        .peers_next
                        .windows(2)
                        .all(|w| key_func(&w[0]) < key_func(&w[1])));
                    return true;
                }
            }
        } else if self.peers_waiting.is_empty() && self.peers_next.is_empty() {
            self.peers_all.clear();
            let result = std::mem::take(&mut self.peers_responded);
            self.state = QueryState::Completed(result);
            return true;
        }

        false
    }

    /// Destroys the query and calculates the correctness of the obtained results
    /// (proportion of nodes that are actually top `K_VALUE` by proximity to the key).
    ///
    /// This method is slow as it iterates over all peers in the network.
    pub fn evaluate(self) -> f64 {
        let mut result = match self.state {
            QueryState::Completed(result) => result,
            QueryState::InProgress => return 0.,
        };
        result.sort_unstable();

        // Calculate the correct answer
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        struct HeapItem {
            dist: Distance,
            peer_id: PeerId,
        }

        let mut heap = BinaryHeap::with_capacity(*K_VALUE);
        for peer_id in 0..CONFIG.num_peers {
            let dist = Key::from_peer_id(peer_id).distance(&self.target_key);
            if heap.len() < *K_VALUE {
                heap.push(HeapItem { dist, peer_id });
            } else if dist < heap.peek().unwrap().dist {
                heap.pop();
                heap.push(HeapItem { dist, peer_id });
            }
        }
        let correct_result = heap
            .into_iter()
            .map(|item| item.peer_id)
            .collect::<HashSet<_>>();
        result
            .iter()
            .map(|peer_id| correct_result.contains(peer_id) as u64)
            .sum::<u64>() as f64
            / *K_VALUE as f64
    }

    /// Returns a key function for sorting peers by distance to the target key in descending order.
    fn key_func(&self) -> impl Fn(&PeerId) -> Distance {
        let target_key = self.target_key.clone();
        move |&peer_id| !Key::from_peer_id(peer_id).distance(&target_key)
    }
}
