use super::QueryState;
use crate::{
    message::FindNodeRequest,
    query::{QueryId, QueryPool},
    Key, PeerId, ALPHA_VALUE, K_VALUE,
};
use arrayvec::ArrayVec;

pub struct FindNodeQuery {
    target_key: Key,
    peers_responded: Vec<PeerId>,
    peers_waiting: ArrayVec<PeerId, ALPHA_VALUE>,
    peers_next: Vec<PeerId>,
    pub state: QueryState<Vec<PeerId>>,
}

impl FindNodeQuery {
    pub fn new_query(
        queries: &mut QueryPool,
        target_key: Key,
        local_closest_peers: Vec<PeerId>,
    ) -> (QueryId, Vec<(PeerId, FindNodeRequest)>) {
        assert!(
            !local_closest_peers.is_empty(),
            "Failed to start FindNodeQuery: k-buckets table is empty"
        );
        let query_id = queries.add_find_node_query(FindNodeQuery {
            target_key: target_key.clone(),
            peers_responded: vec![],
            peers_waiting: ArrayVec::new(),
            peers_next: local_closest_peers,
            state: QueryState::InProgress,
        });
        let query = queries.get_mut_find_node_query(query_id).unwrap();
        let requests = (0..ALPHA_VALUE)
            .filter_map(|_| query.pop_next_peer())
            .map(|peer_id| {
                (
                    peer_id,
                    FindNodeRequest {
                        query_id,
                        key: target_key.clone(),
                    },
                )
            })
            .collect::<Vec<_>>();
        (query_id, requests)
    }

    pub fn on_response(
        &mut self,
        src_id: PeerId,
        query_id: QueryId,
        closest_peers: Vec<PeerId>,
    ) -> Option<(PeerId, FindNodeRequest)> {
        if matches!(self.state, QueryState::Completed(_)) {
            return None;
        }
        self.peers_waiting.retain(|id| *id != src_id);
        self.peers_responded.push(src_id);
        for peer_id in closest_peers {
            if !self.peers_responded.contains(&peer_id) && !self.peers_waiting.contains(&peer_id) {
                self.peers_next.push(peer_id);
            }
        }
        if self.check_termination() || self.peers_waiting.len() >= ALPHA_VALUE {
            return None;
        }
        self.pop_next_peer().map(|peer_id| {
            (
                peer_id,
                FindNodeRequest {
                    query_id,
                    key: self.target_key.clone(),
                },
            )
        })
    }

    fn pop_next_peer(&mut self) -> Option<PeerId> {
        let next_peer = self
            .peers_next
            .iter()
            .min_by_key(|&&peer_id| Key::from_peer_id(peer_id).distance(&self.target_key))
            .copied();
        if let Some(peer_id) = next_peer {
            self.peers_next.retain(|&id| id != peer_id);
            self.peers_waiting.push(peer_id);
        }
        next_peer
    }

    fn check_termination(&mut self) -> bool {
        if self.peers_responded.len() >= K_VALUE {
            self.peers_responded
                .select_nth_unstable_by_key(K_VALUE - 1, |peer_id| {
                    Key::from_peer_id(*peer_id).distance(&self.target_key)
                });
        }

        if self.peers_waiting.is_empty() && self.peers_next.is_empty() {
            self.state = QueryState::Completed(self.peers_responded[..K_VALUE].to_vec());
            return true;
        }
        if self.peers_responded.len() < K_VALUE {
            return false;
        }

        if let Some(&peer_id) = self
            .peers_next
            .iter()
            .min_by_key(|&&peer_id| Key::from_peer_id(peer_id).distance(&self.target_key))
        {
            let dist_next = Key::from_peer_id(peer_id).distance(&self.target_key);
            let worst_dist_responded =
                Key::from_peer_id(self.peers_responded[K_VALUE - 1]).distance(&self.target_key);
            if worst_dist_responded < dist_next {
                self.state = QueryState::Completed(self.peers_responded[..K_VALUE].to_vec());
                return true;
            }
        }
        false
    }
}
