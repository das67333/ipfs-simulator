use crate::{
    kbucket::{KBucketsTable, OnFullKBucket},
    message::{
        FindNodeRequest, FindNodeResponse, GetValueRequest, GetValueResponse, PingRequest,
        PingResponse, PutValueRequest,
    },
    network::NetworkAgent,
    query::{
        FindNodeQuery, GetValueQuery, PutValueQuery, QueriesPool, QueriesStats, QueryId,
        QueryState, QueryTrigger,
    },
    storage::{LocalDHTStorage, Record},
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
        self.stats.find_node_queries_started += 1;
        self.send_message(request, self.ctx.id());
        query_id
    }

    pub fn get_value(&mut self, key: Key) -> QueryId {
        let query_id = self.queries.next_query_id();
        self.find_node(&key, QueryTrigger::GetValue(query_id));
        let query = GetValueQuery::new(key);
        self.queries.add_get_value_query(query_id, query);
        self.stats.get_value_queries_started += 1;
        query_id
    }

    /// Puts a value into the DHT.
    pub fn put_value(&mut self, value: String) -> QueryId {
        let query_id = self.queries.next_query_id();
        let record = Record {
            value,
            expires_at: self.ctx.time() + CONFIG.provider_record_expiration_interval,
        };
        // TODO: republish
        let query = PutValueQuery::new(record);
        let key = query.key();
        self.queries.add_put_value_query(query_id, query);
        self.stats.put_value_queries_started += 1;
        self.find_node(&key, QueryTrigger::PutValue(query_id));
        query_id
    }

    fn on_find_node_request(&mut self, src_id: PeerId, query_id: QueryId, key: Key) {
        let closest_peers = self
            .kbuckets
            .local_closest_peers_approximate(&key, *K_VALUE);
        self.send_message(
            FindNodeResponse {
                query_id,
                closest_peers,
            },
            src_id,
        );
    }

    fn on_find_node_response(
        &mut self,
        src_id: PeerId,
        query_id: QueryId,
        closest_peers: Vec<PeerId>,
    ) {
        if let Some(query) = self.queries.get_mut_find_node_query(query_id) {
            match query.on_response(src_id, query_id, closest_peers) {
                QueryState::InProgress(requests) => {
                    for (dst, request) in requests {
                        self.send_message(request, dst);
                    }
                }
                QueryState::Completed((target_key, peers)) => {
                    self.stats.evaluate(target_key, &peers);

                    match query.trigger() {
                        QueryTrigger::PutValue(query_id) => {
                            if let Some(query) = self.queries.remove_put_value_query(query_id) {
                                self.stats.put_value_queries_completed += 1;
                                for peer in peers {
                                    self.send_message(
                                        PutValueRequest {
                                            key: query.key(),
                                            record: query.record(),
                                        },
                                        peer,
                                    );
                                }
                            }
                        }
                        QueryTrigger::GetValue(query_id) => {
                            if let Some(query) = self.queries.get_mut_get_value_query(query_id) {
                                let key = query.key();
                                for peer in peers {
                                    self.send_message(
                                        GetValueRequest {
                                            query_id,
                                            key: key.clone(),
                                        },
                                        peer,
                                    );
                                }
                            }
                        }
                        _ => {}
                    }

                    self.queries.remove_find_node_query(query_id);
                    self.stats.find_node_queries_completed += 1;
                }
            }
        }
    }

    fn on_get_value_request(&mut self, src_id: PeerId, query_id: QueryId, key: Key) {
        let record = self.storage.get(&key).cloned();
        self.send_message(GetValueResponse { query_id, record }, src_id);
    }

    fn on_get_value_response(&mut self, src_id: PeerId, query_id: QueryId, record: Option<Record>) {
        if let Some(query) = self.queries.get_mut_get_value_query(query_id) {
            match query.on_response(src_id, record) {
                QueryState::InProgress(()) => {}
                QueryState::Completed((_value, requests)) => {
                    for (dst, request) in requests {
                        self.send_message(request, dst);
                    }
                    self.queries.remove_get_value_query(query_id);
                    self.stats.get_value_queries_completed += 1;
                }
            }
        }
    }

    fn on_put_value_request(&mut self, key: Key, record: Record) {
        self.storage.put(key, record);
    }

    fn on_ping_request(&mut self, src_id: PeerId) {
        self.stats.ping_requests_cnt += 1;
        self.send_message(PingResponse {}, src_id);
    }

    fn on_ping_response(&mut self) {
        self.stats.ping_responses_cnt += 1;
    }
}

impl EventHandler for Peer {
    fn on(&mut self, event: Event) {
        self.kbuckets
            .add_peer(event.src, OnFullKBucket::ReplaceLeastRecentlySeen);

        cast!(match event.data {
            FindNodeRequest { query_id, key } => {
                self.on_find_node_request(event.src, query_id, key);
            }
            FindNodeResponse {
                query_id,
                closest_peers,
            } => {
                self.on_find_node_response(event.src, query_id, closest_peers);
            }
            GetValueRequest { query_id, key } => {
                self.on_get_value_request(event.src, query_id, key);
            }
            GetValueResponse { query_id, record } => {
                self.on_get_value_response(event.src, query_id, record);
            }
            PutValueRequest { key, record } => {
                self.on_put_value_request(key, record);
            }
            PingRequest {} => {
                self.on_ping_request(event.src);
            }
            PingResponse {} => {
                self.on_ping_response();
            }
        });
    }
}
