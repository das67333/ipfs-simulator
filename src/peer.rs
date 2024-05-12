use crate::{
    kbucket::{KBucketsTable, OnFullKBucket},
    message::{FindNodeRequest, FindNodeResponse, PingRequest, PingResponse, PutValueRequest},
    network::NetworkAgent,
    query::{
        FindNodeQuery, PutValueQuery, QueriesPool, QueriesStats, QueryId, QueryState, QueryTrigger,
    },
    storage::LocalDHTStorage,
    Key, PeerId, CONFIG, K_VALUE,
};
use dslab_core::{cast, Event, EventData, EventHandler, Simulation, SimulationContext};

/// Represents a peer in the IPFS simulator.
pub struct Peer {
    ctx: SimulationContext,
    kbuckets: KBucketsTable,
    queries: QueriesPool,
    network: NetworkAgent,
    storage: LocalDHTStorage,
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
            queries: QueriesPool::new(),
            network,
            storage: LocalDHTStorage::new(),
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
    /// # Arguments
    ///
    /// * `trigger` - The trigger that initiated the query.
    ///
    /// # Returns
    ///
    /// The ID of the initiated query.
    pub fn find_random_node(&mut self, trigger: QueryTrigger) -> QueryId {
        let key = Key::random(&self.ctx);
        self.find_node(&key, trigger)
    }

    /// Finds the closest nodes to the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to find the closest nodes to.
    /// * `trigger` - The trigger that initiated the query.
    ///
    /// # Returns
    ///
    /// The ID of the initiated query.
    pub fn find_node(&mut self, key: &Key, trigger: QueryTrigger) -> QueryId {
        let query_id = self.queries.next_query_id();
        let (query_request, request) =
            FindNodeQuery::new(query_id, trigger, key.clone(), self.ctx.id());
        self.queries.add_find_node_query(query_id, query_request);
        self.send_message(request, self.ctx.id());
        self.stats.find_node_queries_started += 1;
        query_id
    }

    /// Puts a value into the DHT.
    pub fn put_value(&mut self, value: String) -> QueryId {
        let query_id = self.queries.next_query_id();
        let (query, key) = PutValueQuery::new(value);
        self.queries.add_put_value_query(query_id, query);
        self.stats.put_value_queries_started += 1;
        self.find_node(&key, QueryTrigger::PutValue(query_id));
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
                if let Some(query) = self.queries.get_mut_find_node_query(query_id) {
                    match query.on_response(event.src, query_id, closest_peers) {
                        QueryState::InProgress(requests) => {
                            for (dst, request) in requests {
                                self.send_message(request, dst);
                            }
                        }
                        QueryState::Completed((target_key, peers)) => {
                            self.stats.find_node_queries_completed += 1;
                            self.stats.evaluate(target_key, &peers);

                            if let QueryTrigger::PutValue(query_id) = query.trigger() {
                                if let Some(query) = self.queries.remove_put_value_query(query_id) {
                                    self.stats.put_value_queries_completed += 1;
                                    for peer in peers {
                                        self.send_message(
                                            PutValueRequest {
                                                value: query.value(),
                                                expires_at: self.ctx.time()
                                                    + CONFIG.provider_record_expiration_interval,
                                            },
                                            peer,
                                        );
                                    }
                                }
                            }

                            self.queries.remove_find_node_query(query_id);
                        }
                    }
                }
            }
            PutValueRequest { value, expires_at } => {
                self.storage.put(value, expires_at);
            }
            PingRequest {} => {
                self.stats.ping_requests_cnt += 1;
                self.send_message(PingResponse {}, event.src);
            }
            PingResponse {} => {
                self.stats.ping_responses_cnt += 1;
            }
        });
    }
}
