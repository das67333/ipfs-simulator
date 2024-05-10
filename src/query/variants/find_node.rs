use std::collections::HashSet;

use super::QueryState;
use crate::{
    message::FindNodeRequest,
    query::{QueryId, QueryPool},
    Distance, Key, PeerId, ALPHA_VALUE, K_VALUE,
};

pub struct FindNodeQuery {
    target_key: Key,
    peers_all: HashSet<PeerId>,   // waiting + responded + next
    peers_responded: Vec<PeerId>, // sorted by distance to target in descending order
    peers_waiting: Vec<PeerId>,
    peers_next: Vec<PeerId>, // sorted by distance to target in descending order
    pub state: QueryState<Vec<PeerId>>,
}

impl FindNodeQuery {
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
                let mut v = Vec::with_capacity(ALPHA_VALUE);
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
        while self.peers_waiting.len() < ALPHA_VALUE {
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
        if self.peers_responded.len() >= K_VALUE {
            if let Some(&peer_id) = self.peers_next.last() {
                let i = self.peers_responded.len() - K_VALUE;
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
        } else {
            if self.peers_waiting.is_empty() && self.peers_next.is_empty() {
                self.peers_all.clear();
                let result = std::mem::replace(&mut self.peers_responded, vec![]);
                self.state = QueryState::Completed(result);
                return true;
            }
        }
        false
    }

    /// Evaluate
    pub fn evaluate(self) -> f64 {
        let mut result = match self.state {
            QueryState::Completed(result) => result,
            QueryState::InProgress => return 0.,
        };
        todo!()
    }

    /// Returns a key function for sorting peers by distance to the target key in descending order.
    fn key_func(&self) -> impl Fn(&PeerId) -> Distance {
        let target_key = self.target_key.clone();
        move |&peer_id| !Key::from_peer_id(peer_id).distance(&target_key)
    }
}
