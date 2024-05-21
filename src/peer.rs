use crate::{
    kbucket::KBucketsTable,
    message::{
        BootstrapTimer, FindNodeRequest, FindNodeResponse, GetValueRequest, GetValueResponse,
        PingRequest, PingResponse, PutValueRequest, RepublishTimer, RetrieveDataRequest,
        RetrieveDataResponse,
    },
    network::NetworkAgent,
    query::{
        FindNodeQuery, GetValueQuery, PutValueQuery, QueriesPool, QueriesStats, QueryId,
        QueryState, QueryTrigger,
    },
    storage::{LocalDHTStorage, LocalFileStorage, Record, RecordData},
    Key, PeerId, CONFIG, K_VALUE,
};
use dslab_core::{cast, Event, EventData, EventHandler, Simulation, SimulationContext};

/// Represents a peer in the IPFS simulator.
pub struct Peer {
    ctx: SimulationContext,
    kbuckets: KBucketsTable,
    queries: QueriesPool,
    network: NetworkAgent,
    dht_storage: LocalDHTStorage,
    file_storage: LocalFileStorage,
    stats: QueriesStats,
}

impl Peer {
    /// Creates a new peer within the given simulation. Its `name` should be unique.
    pub fn new(sim: &mut Simulation, name: impl AsRef<str>, network: NetworkAgent) -> Self {
        let ctx = sim.create_context(name);
        let local_key = Key::from_peer_id(ctx.id());

        if CONFIG.enable_bootstrap {
            // Schedule the first refresh of the k-buckets table.
            let delay = ctx.sample_from_distribution(&rand::distributions::Uniform::new(
                0.0,
                CONFIG.kbuckets_refresh_interval,
            ));
            ctx.emit_self(BootstrapTimer {}, delay);
        }
        Self {
            ctx,
            kbuckets: KBucketsTable::new(local_key),
            queries: QueriesPool::new(),
            network,
            dht_storage: LocalDHTStorage::new(),
            file_storage: LocalFileStorage::new(),
            stats: QueriesStats::new(),
        }
    }

    /// Adds a peer to the k-buckets table.
    pub fn add_peer(&mut self, peer_id: PeerId, curr_time: f64) {
        self.kbuckets.add_peer(peer_id, curr_time);
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

    /// Puts a record into the DHT.
    pub fn put_value(&mut self, record: Record) -> QueryId {
        let query_id = self.queries.next_query_id();
        let query = PutValueQuery::new(record);
        let key = query.key();
        self.queries.add_put_value_query(query_id, query);
        self.stats.put_value_queries_started += 1;
        self.find_node(&key, QueryTrigger::PutValue(query_id));
        query_id
    }

    pub fn publish_data(&mut self, data: String) -> Key {
        let key = Key::from_sha256(data.as_bytes());
        let record = Record::new_provider_record(self.id(), key.clone(), self.ctx.time());
        self.file_storage.put(key.clone(), data);
        self.dht_storage.put(key.clone(), record.clone());
        self.put_value(record);
        self.ctx.emit_self(
            RepublishTimer { key: key.clone() },
            CONFIG.record_publication_interval,
        );
        key
    }

    pub fn remove_data(&mut self, key: Key) {
        if let (Some(_), Some(_)) = (self.dht_storage.get(&key), self.file_storage.get(&key)) {
            self.dht_storage.remove(&key);
            self.file_storage.remove(&key);
        }
    }

    pub fn retrieve_data(&mut self, key: Key) -> QueryId {
        let query_id = self.get_value(key);
        self.queries.add_retrieve_data_query(query_id);
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

                    for &id in peers.iter() {
                        self.kbuckets.add_peer(id, self.ctx.time());
                    }

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
        let record = self.dht_storage.get(&key).cloned();
        self.send_message(GetValueResponse { query_id, record }, src_id);
    }

    fn on_get_value_response(&mut self, src_id: PeerId, query_id: QueryId, record: Option<Record>) {
        if let Some(query) = self.queries.get_mut_get_value_query(query_id) {
            match query.on_response(src_id, record) {
                QueryState::InProgress(()) => {}
                QueryState::Completed((record, requests)) => {
                    for (dst, request) in requests {
                        self.send_message(request, dst);
                    }
                    self.queries.remove_get_value_query(query_id);
                    self.stats.get_value_queries_completed += 1;
                    match record.data {
                        RecordData::ProviderRecord { key, providers } => {
                            for provider in providers {
                                self.send_message(
                                    RetrieveDataRequest {
                                        query_id,
                                        key: key.clone(),
                                    },
                                    provider,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn on_put_value_request(&mut self, key: Key, record: Record) {
        self.dht_storage.put(key, record);
    }

    fn on_retrieve_data_request(&mut self, src_id: PeerId, query_id: QueryId, key: Key) {
        if let Some(data) = self.file_storage.get(&key) {
            self.send_message(
                RetrieveDataResponse {
                    query_id,
                    data: Some(data.clone()),
                },
                src_id,
            );
        }
    }

    fn on_retrieve_data_response(&mut self, query_id: QueryId, data: Option<String>) {
        if let Some(data) = data {
            if self.queries.remove_retrieve_data_query(query_id) {
                // TODO: log that received data
            }
        }
    }

    fn on_ping_request(&mut self, src_id: PeerId) {
        self.stats.ping_requests_cnt += 1;
        self.send_message(PingResponse {}, src_id);
    }

    fn on_ping_response(&mut self) {
        self.stats.ping_responses_cnt += 1;
    }

    fn refresh_kbuckets_table(&mut self) {
        self.dht_storage.remove_expired(self.ctx.time());
        for i in 0..self.kbuckets.buckets_count().min(15) {
            let key = Key::random_in_bucket(&self.ctx, i);
            self.find_node(&key, QueryTrigger::Bootstrap);
        }
        let local_key = self.kbuckets.local_key();
        self.find_node(&local_key, QueryTrigger::Bootstrap);
        self.ctx
            .emit_self(BootstrapTimer {}, CONFIG.kbuckets_refresh_interval);
    }

    fn on_republish_timer(&mut self, key: Key) {
        if let (Some(record), Some(_)) = (
            self.dht_storage.get(&key).cloned(),
            self.file_storage.get(&key),
        ) {
            self.dht_storage.remove(&key);
            self.put_value(record.refreshed(self.ctx.time()));
            self.ctx
                .emit_self(RepublishTimer { key }, CONFIG.record_publication_interval);
        }
    }
}

impl EventHandler for Peer {
    fn on(&mut self, event: Event) {
        self.kbuckets.add_peer(event.src, self.ctx.time());

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
            RetrieveDataRequest { query_id, key } => {
                self.on_retrieve_data_request(event.src, query_id, key);
            }
            RetrieveDataResponse { query_id, data } => {
                self.on_retrieve_data_response(query_id, data);
            }
            PingRequest {} => {
                self.on_ping_request(event.src);
            }
            PingResponse {} => {
                self.on_ping_response();
            }
            BootstrapTimer {} => {
                self.refresh_kbuckets_table();
            }
            RepublishTimer { key } => {
                self.on_republish_timer(key);
            }
        });
    }
}
