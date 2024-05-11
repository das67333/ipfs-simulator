use crate::{
    kbucket::{KBucketsTable, OnFullKBucket},
    message::{FindNodeRequest, FindNodeResponse},
    network::NetworkAgent,
    query::{FindNodeQuery, QueriesStats, QueryId, QueryPool, QueryState},
    Key, PeerId, K_VALUE,
};
use dslab_core::{cast, Event, EventData, EventHandler, Simulation, SimulationContext};

/// Represents a peer in the IPFS simulator.
pub struct Peer {
    ctx: SimulationContext,
    kbuckets: KBucketsTable,
    queries: QueryPool,
    network: NetworkAgent,
    stats: QueriesStats,
}

impl Peer {
    /// Creates a new peer within the given simulation. Its `name` should be unique.
    pub fn new(sim: &mut Simulation, name: impl AsRef<str>, network: NetworkAgent) -> Self {
        let ctx = sim.create_context(name);
        let local_key = Key::from_peer_id(ctx.id());
        Self {
            ctx,
            kbuckets: KBucketsTable::new(local_key),
            queries: QueryPool::new(),
            network,
            stats: QueriesStats::new(),
        }
    }

    /// Adds a peer to the k-buckets table.
    pub fn add_peer(&mut self, peer_id: PeerId, on_full: OnFullKBucket) {
        self.kbuckets.add_peer(peer_id, on_full);
    }

    /// Returns the statistics related to queries.
    pub fn stats(&mut self) -> QueriesStats {
        std::mem::take(&mut self.stats)
    }

    /// Returns the ID of the peer.
    pub fn id(&self) -> PeerId {
        self.ctx.id()
    }

    /// Sends a message to the specified destination peer.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to send as the message.
    /// * `dst` - The ID of the destination peer.
    fn send_message(&mut self, data: impl EventData, dst: PeerId) {
        if let Some(delay) = self
            .network
            .sample_message_delay(&self.ctx, self.ctx.id(), dst)
        {
            self.ctx.emit(data, dst, delay);
        }
    }

    /// Finds the closest peers to a random key.
    ///
    /// # Returns
    ///
    /// The ID of the initiated query.
    pub fn find_random_node(&mut self) -> QueryId {
        let key = Key::random(&self.ctx);
        self.find_node(&key)
    }

    /// Finds the closest nodes to the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to find the closest nodes to.
    ///
    /// # Returns
    ///
    /// The ID of the initiated query.
    pub fn find_node(&mut self, key: &Key) -> QueryId {
        let (query_id, request) =
            FindNodeQuery::new_query(&mut self.queries, key.clone(), self.ctx.id());
        self.send_message(request, self.ctx.id());
        query_id
    }
}

impl EventHandler for Peer {
    fn on(&mut self, event: Event) {
        self.kbuckets
            .add_peer(event.src, OnFullKBucket::ReplaceLeastRecentlySeen);

        cast!(match event.data {
            FindNodeRequest { query_id, key } => {
                let closest_peers = self
                    .kbuckets
                    .local_closest_peers_approximate(&key, *K_VALUE);
                self.send_message(
                    FindNodeResponse {
                        query_id,
                        closest_peers,
                    },
                    event.src,
                );
            }
            FindNodeResponse {
                query_id,
                closest_peers,
            } => {
                let mut to_remove = false;
                if let Some(query) = self.queries.get_mut_find_node_query(query_id) {
                    match query.on_response(event.src, query_id, closest_peers) {
                        QueryState::InProgress(requests) => {
                            for (dst, request) in requests {
                                self.send_message(request, dst);
                            }
                        }
                        QueryState::Completed((target_key, peers)) => {
                            self.stats.evaluate(target_key, &peers);
                            to_remove = true;
                        }
                    }
                }
                if to_remove {
                    self.queries.remove_find_node_query(query_id);
                }
            }
        });
    }
}
