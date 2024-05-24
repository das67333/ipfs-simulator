use crate::{
    kbucket::KBucketsTable,
    message::{
        BootstrapTimer, FindNodeQueryTimeout, FindNodeRequest, FindNodeResponse,
        GetValueQueryTimeout, GetValueRequest, GetValueResponse, PingRequest, PingResponse,
        PingTimeout, PutValueQueryTimeout, PutValueRequest, RepublishTimer,
        RetrieveDataQueryTimeout, RetrieveDataRequest, RetrieveDataResponse,
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
use log::Level;

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
    /// Creates a new peer within the given simulation.
    ///
    /// # Arguments
    ///
    /// * `sim` - A mutable reference to the simulation.
    /// * `name` - The name of the peer. Should be unique.
    /// * `network` - The network agent associated with the peer.
    ///
    /// # Returns
    ///
    /// A new instance of `Peer`.
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
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to add.
    /// * `curr_time` - The current simulation time.
    pub fn add_peer(&mut self, peer_id: PeerId, curr_time: f64) {
        self.kbuckets.add_peer(peer_id, curr_time);
    }

    /// Clears the storage of the peer.
    ///
    /// This method clears both the DHT storage and the file storage.
    pub fn clear_storage(&mut self) {
        self.log(Level::Debug, "Cleared storage");
        self.dht_storage.clear();
        self.file_storage.clear();
    }

    /// Returns the statistics related to queries.
    ///
    /// # Returns
    ///
    /// The statistics related to queries.
    pub fn stats(&mut self) -> QueriesStats {
        std::mem::take(&mut self.stats)
    }

    /// Effectively fills the k-buckets table with random peers.
    /// This method uses information that is not available in the real world.
    pub fn fill_kbuckets_unfair(&mut self) {
        for i in 0..CONFIG.num_peers.ilog2() as usize {
            for _ in 0..*crate::K_VALUE {
                let key = Key::random_in_bucket(&self.ctx, self.kbuckets.local_key(), i);
                let peers = crate::KEYS_TREE.find_closest_peers(&key, 1);
                let peer_id = peers.iter().next().unwrap();
                self.kbuckets.add_peer(*peer_id, self.ctx.time());
            }
        }
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

    /// Initaites an iterative search for the closest nodes to the given key.
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
        self.log(
            Level::Debug,
            &format!("Initiated FindNodeQuery with id={}", query_id),
        );
        self.ctx
            .emit_self(FindNodeQueryTimeout { query_id }, CONFIG.query_timeout);
        let (query_request, request) =
            FindNodeQuery::new(query_id, trigger, key.clone(), self.ctx.id());
        self.queries.add_find_node_query(query_id, query_request);
        self.stats.find_node_queries_started += 1;
        self.send_message(request, self.ctx.id());
        query_id
    }

    /// Initiates a query to get the DHT record associated with a key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to get the record for.
    ///
    /// # Returns
    ///
    /// The ID of the initiated query.
    pub fn get_value(&mut self, key: Key) -> QueryId {
        let query_id = self.queries.next_query_id();
        self.log(
            Level::Debug,
            &format!("Initiated GetValueQuery with id={}", query_id),
        );
        self.ctx
            .emit_self(GetValueQueryTimeout { query_id }, CONFIG.query_timeout);
        self.find_node(&key, QueryTrigger::GetValue(query_id));
        let query = GetValueQuery::new(key);
        self.queries.add_get_value_query(query_id, query);
        self.stats.get_value_queries_started += 1;
        query_id
    }

    /// Initiates a query to put the given record into the DHT.
    ///
    /// # Arguments
    ///
    /// * `record` - The record to put into the DHT.
    ///
    /// # Returns
    ///
    /// The ID of the initiated query.
    pub fn put_value(&mut self, record: Record) -> QueryId {
        let query_id = self.queries.next_query_id();
        self.log(
            Level::Debug,
            &format!("Initiated PutValueQuery with id={}", query_id),
        );
        self.ctx
            .emit_self(PutValueQueryTimeout { query_id }, CONFIG.query_timeout);
        let query = PutValueQuery::new(record);
        let key = query.key();
        self.queries.add_put_value_query(query_id, query);
        self.stats.put_value_queries_started += 1;
        self.find_node(&key, QueryTrigger::PutValue(query_id));
        query_id
    }

    /// Publishes data into the IPFS network.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to publish.
    ///
    /// # Returns
    ///
    /// The key associated with the published data.
    pub fn publish_data(&mut self, data: String) -> Key {
        self.log(
            Level::Info,
            &format!("Initiated publishing data \"{}\"", data),
        );
        let key = Key::from_sha256(data.as_bytes());
        let record = Record::new_provider_record(self.id(), key.clone(), self.ctx.time());
        self.file_storage.put(key.clone(), data);
        self.dht_storage.put(key.clone(), record.clone());
        self.put_value(record);
        if CONFIG.enable_republishing {
            self.ctx.emit_self(
                RepublishTimer { key: key.clone() },
                CONFIG.record_publication_interval,
            );
        }
        key
    }

    /// Removes the data associated with the given key from the IPFS network
    /// by interrupting periodic republishing.
    ///
    /// # Arguments
    ///
    /// * `key` - The key associated with the data to remove.
    pub fn remove_data(&mut self, key: Key) {
        if let (Some(_), Some(_)) = (self.dht_storage.get(&key), self.file_storage.get(&key)) {
            self.log(Level::Info, &format!("Removed data by key \"{}\"", key));
            self.dht_storage.remove(&key);
            self.file_storage.remove(&key);
        }
    }

    /// Retrieves the data associated with the given key from IPFS network.
    ///
    /// # Arguments
    ///
    /// * `key` - The key associated with the data to retrieve.
    ///
    /// # Returns
    ///
    /// The ID of the initiated query.
    pub fn retrieve_data(&mut self, key: Key) -> QueryId {
        self.log(
            Level::Info,
            &format!("Initiated retrieving data by key \"{}\"", key),
        );
        let query_id = self.get_value(key);
        self.ctx
            .emit_self(RetrieveDataQueryTimeout { query_id }, CONFIG.query_timeout);
        self.queries.add_retrieve_data_query(query_id);
        self.stats.retrieve_data_queries_started += 1;
        query_id
    }

    /// Handles a `FindNodeRequest` message.
    ///
    /// # Arguments
    ///
    /// * `src_id` - The ID of the source peer.
    /// * `query_id` - The ID of the query that made the request.
    /// * `key` - The key to find the closest peers to.
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

    /// Handles a `FindNodeResponse` message.
    ///
    /// # Arguments
    ///
    /// * `src_id` - The ID of the source peer.
    /// * `query_id` - The ID of the query that made the request.
    /// * `closest_peers` - The closest peers to the target key.
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
                                self.log(
                                    Level::Debug,
                                    &format!("Completed PutValueQuery with id={}", query_id),
                                );
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
                    self.log(
                        Level::Debug,
                        &format!("Completed FindNodeQuery with id={}", query_id),
                    );
                    self.stats.find_node_queries_completed += 1;
                }
            }
        }
    }

    /// Removes a `FindNodeQuery` from the pool of queries if it hasn't completed yet.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query to remove.
    fn on_find_node_query_timeout(&mut self, query_id: QueryId) {
        if self.queries.remove_find_node_query(query_id).is_some() {
            self.log(
                Level::Warn,
                &format!("FindNodeQuery with id={} timed out", query_id),
            );
            self.stats.find_node_queries_failed += 1;
        }
    }

    /// Handles a `GetValueRequest` message.
    ///
    /// # Arguments
    ///
    /// * `src_id` - The ID of the source peer.
    /// * `query_id` - The ID of the query that made the request.
    /// * `key` - The key to get the value for.
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

    /// Removes a `GetValueQuery` from the pool of queries if it hasn't completed yet.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query to remove.
    fn on_get_value_query_timeout(&mut self, query_id: QueryId) {
        if self.queries.remove_get_value_query(query_id).is_some() {
            self.log(
                Level::Warn,
                &format!("GetValueQuery with id={} timed out", query_id),
            );
            self.stats.get_value_queries_failed += 1;
        }
    }

    /// Handles a `PutValueRequest` message.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to put the value for.
    /// * `record` - The record to put.
    fn on_put_value_request(&mut self, key: Key, record: Record) {
        self.dht_storage.put(key, record);
    }

    /// Removes a `PutValueQuery` from the pool of queries if it hasn't completed yet.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query to remove.
    fn on_put_value_query_timeout(&mut self, query_id: QueryId) {
        if self.queries.remove_put_value_query(query_id).is_some() {
            self.log(
                Level::Warn,
                &format!("PutValueQuery with id={} timed out", query_id),
            );
            self.stats.put_value_queries_failed += 1;
        }
    }

    /// Handles a `RetrieveDataRequest` message.
    ///
    /// # Arguments
    ///
    /// * `src_id` - The ID of the source peer.
    /// * `query_id` - The ID of the query that made the request.
    /// * `key` - The key to retrieve the data for.
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

    /// Handles a `RetrieveDataResponse` message.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query that made the request.
    /// * `data` - The data retrieved.
    fn on_retrieve_data_response(&mut self, query_id: QueryId, data: Option<String>) {
        if let Some(data) = data {
            if self.queries.remove_retrieve_data_query(query_id) {
                self.stats.retrieve_data_queries_completed += 1;
                self.log(Level::Info, &format!("Data retrieved: {}", data));
            }
        }
    }

    /// Removes a `RetrieveDataQuery` from the pool of queries if it hasn't completed yet.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query to remove.
    fn on_retrieve_data_query_timeout(&mut self, query_id: QueryId) {
        if self.queries.remove_retrieve_data_query(query_id) {
            self.log(
                Level::Warn,
                &format!("RetrieveDataQuery with id={} timed out", query_id),
            );
            self.stats.retrieve_data_queries_failed += 1;
        }
    }

    /// Handles a `PingRequest` message.
    ///
    /// # Arguments
    ///
    /// * `src_id` - The ID of the source peer.
    fn on_ping_request(&mut self, src_id: PeerId) {
        self.stats.ping_requests_cnt += 1;
        self.send_message(PingResponse {}, src_id);
    }

    /// Handles a `PingResponse` message.
    fn on_ping_response(&mut self) {
        self.stats.ping_responses_cnt += 1;
    }

    /// Ping timeouts are not used and not implemented to save memory.
    fn on_ping_timeout(&mut self) {
        self.stats.ping_requests_failed += 1;
    }

    /// Refreshes the k-buckets table by querying the peers closest to some
    /// random keys that fit in different buckets to keep the table up-to-date.
    ///
    /// This method is called periodically to refresh the k-buckets table.
    /// The local key is also queried to add the peers closest to the local key.
    ///
    /// The method also removes expired records from the DHT storage.
    fn refresh_kbuckets_table(&mut self) {
        self.dht_storage.remove_expired(self.ctx.time());
        for i in 0..self.kbuckets.buckets_count().min(15) {
            let key = Key::random_in_bucket(&self.ctx, self.kbuckets.local_key(), i);
            self.find_node(&key, QueryTrigger::Bootstrap);
        }
        let local_key = self.kbuckets.local_key();
        self.find_node(&local_key, QueryTrigger::Bootstrap);
        self.ctx
            .emit_self(BootstrapTimer {}, CONFIG.kbuckets_refresh_interval);
    }

    /// Republishes the record associated with the given key.
    /// This method is called periodically to republish the record.
    ///
    /// # Arguments
    ///
    /// * `key` - The key associated with the record to republish.
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

    /// Logs a message with the current time and the name of the peer.
    fn log(&self, level: Level, msg: &str) {
        log::log!(target: "simulation",level, "[{:.3} {}] {}", self.ctx.time(), self.ctx.name(), msg);
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
            FindNodeQueryTimeout { query_id } => {
                self.on_find_node_query_timeout(query_id);
            }
            GetValueRequest { query_id, key } => {
                self.on_get_value_request(event.src, query_id, key);
            }
            GetValueResponse { query_id, record } => {
                self.on_get_value_response(event.src, query_id, record);
            }
            GetValueQueryTimeout { query_id } => {
                self.on_get_value_query_timeout(query_id);
            }
            PutValueRequest { key, record } => {
                self.on_put_value_request(key, record);
            }
            PutValueQueryTimeout { query_id } => {
                self.on_put_value_query_timeout(query_id);
            }
            RetrieveDataRequest { query_id, key } => {
                self.on_retrieve_data_request(event.src, query_id, key);
            }
            RetrieveDataResponse { query_id, data } => {
                self.on_retrieve_data_response(query_id, data);
            }
            RetrieveDataQueryTimeout { query_id } => {
                self.on_retrieve_data_query_timeout(query_id);
            }
            PingRequest {} => {
                self.on_ping_request(event.src);
            }
            PingResponse {} => {
                self.on_ping_response();
            }
            PingTimeout {} => {
                self.on_ping_timeout();
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
