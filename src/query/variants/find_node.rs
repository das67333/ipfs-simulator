use super::{QueryState, QueryTrigger};
use crate::{
    message::FindNodeRequest, query::QueryId, Distance, Key, PeerId, ALPHA_VALUE, KEYS_TREE,
    K_VALUE,
};
use std::collections::HashSet;

type FindNodeQueryState = QueryState<Vec<(PeerId, FindNodeRequest)>, (Key, Vec<PeerId>)>;

/// Represents a query to find the closest peers to a target key.
///
/// This struct provides methods to create a new query, handle responses from peers,
/// and evaluate the query to calculate the correctness of the results.
#[derive(Debug)]
pub struct FindNodeQuery {
    trigger: QueryTrigger,
    target_key: Key,
    peers_all: HashSet<PeerId>,   // waiting + responded + next
    peers_responded: Vec<PeerId>, // sorted by distance to target in descending order
    peers_waiting: Vec<PeerId>,
    peers_next: Vec<PeerId>, // sorted by distance to target in descending order
}

impl FindNodeQuery {
    /// Creates a new `FindNodeQuery` instance.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query.
    /// * `trigger` - The trigger that initiated the query.
    /// * `target_key` - The key to find the closest peers to.
    /// * `self_id` - The ID of the peer that initiated the query.
    ///
    /// # Returns
    ///
    /// A tuple containing the query and the request to send to itself.
    pub fn new(
        query_id: QueryId,
        trigger: QueryTrigger,
        target_key: Key,
        self_id: PeerId,
    ) -> (FindNodeQuery, FindNodeRequest) {
        let query = FindNodeQuery {
            trigger,
            target_key: target_key.clone(),
            peers_all: HashSet::from_iter([self_id]),
            peers_responded: vec![],
            peers_waiting: {
                let mut v = Vec::with_capacity(*ALPHA_VALUE);
                v.push(self_id);
                v
            },
            peers_next: vec![],
        };
        let request = FindNodeRequest {
            query_id,
            key: target_key,
        };
        (query, request)
    }

    /// Returns the trigger that initiated the query.
    pub fn trigger(&self) -> QueryTrigger {
        self.trigger.clone()
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
    /// If query is completed, returns the target key and the list of closest peers to it.
    /// Otherwise, returns the list of requests to send to the next peers.
    pub fn on_response(
        &mut self,
        src_id: PeerId,
        query_id: QueryId,
        closest_peers: Vec<PeerId>,
    ) -> FindNodeQueryState {
        match self.peers_waiting.iter().position(|&id| id == src_id) {
            Some(idx) => {
                self.peers_waiting.swap_remove(idx);
            }
            None => return QueryState::InProgress(vec![]),
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

        if let Some(peers) = self.check_if_completed() {
            return QueryState::Completed((self.target_key.clone(), peers));
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
        QueryState::InProgress(result)
    }

    /// Pops the next peer from the list of next peers and moves it to the list of waiting peers.
    ///
    /// # Returns
    ///
    /// The ID of the next peer, if it exists.
    fn pop_next_peer(&mut self) -> Option<PeerId> {
        let next_peer = self.peers_next.pop();
        if let Some(peer_id) = next_peer {
            self.peers_waiting.push(peer_id);
        }
        next_peer
    }

    /// Checks if the query is completed and returns the list
    /// of closest peers if so.
    fn check_if_completed(&mut self) -> Option<Vec<PeerId>> {
        let key_func = self.key_func();
        if self.peers_responded.len() >= *K_VALUE {
            if let Some(&peer_id) = self.peers_next.last() {
                let i = self.peers_responded.len() - *K_VALUE;
                if key_func(&peer_id) < key_func(&self.peers_responded[i]) {
                    let ans = self.peers_responded.split_off(i);
                    return Some(ans);
                }
            }
        } else if self.peers_waiting.is_empty() && self.peers_next.is_empty() {
            return Some(std::mem::take(&mut self.peers_responded));
        }
        None
    }

    /// Returns a key function for sorting peers by distance to the target key
    /// in descending order.
    fn key_func(&self) -> impl Fn(&PeerId) -> Distance {
        let target_key = self.target_key.clone();
        move |&peer_id| !Key::from_peer_id(peer_id).distance(&target_key)
    }
}

/// Calculates the correctness of the obtained results
/// (number of nodes included in the correct answer).
///
/// # Arguments
///
/// * `target_key` - The key used in the query.
/// * `result` - The list of peers returned by the query.
///
/// # Returns
///
/// The number of peers that are included in the correct answer.
pub fn evaluate_closest_peers(target_key: Key, result: &[PeerId]) -> usize {
    let correct_result = KEYS_TREE.find_closest_peers(&target_key, result.len());
    result
        .iter()
        .filter(|&id| correct_result.contains(id))
        .count()
}
