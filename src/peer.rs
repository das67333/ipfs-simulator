use crate::{
    kbucket::KBucketsTable,
    message::{FindNodeRequest, FindNodeResponse},
    network::NetworkAgent,
    query::{FindNodeQuery, QueryId, QueryPool},
    Key, PeerId, K_VALUE,
};
use dslab_core::{cast, Event, EventData, EventHandler, Simulation, SimulationContext};

pub struct Peer {
    ctx: SimulationContext,
    kbuckets: KBucketsTable,
    queries: QueryPool,
    network: NetworkAgent,
}

impl Peer {
    pub fn new(sim: &mut Simulation, name: impl AsRef<str>, network: NetworkAgent) -> Self {
        let ctx = sim.create_context(name);
        let local_key = Key::from_peer_id(ctx.id());
        Self {
            ctx,
            kbuckets: KBucketsTable::new(local_key),
            queries: QueryPool::new(),
            network,
        }
    }

    pub fn add_peer(&mut self, peer_id: PeerId) {
        self.kbuckets.add_peer(peer_id);
    }

    pub fn id(&self) -> PeerId {
        self.ctx.id()
    }

    fn send_message(&mut self, data: impl EventData, dst: PeerId) {
        if let Some(delay) = self.network.get_message_latency(self.ctx.id(), dst) {
            self.ctx.emit(data, dst, delay);
        }
    }

    pub fn stats(&self) -> usize {
        self.queries.stats()
    }

    pub fn find_random_node(&mut self) -> QueryId {
        let key = Key::random(&self.ctx);
        self.find_node(&key)
    }

    /// Finding the closest nodes to the given key.
    pub fn find_node(&mut self, key: &Key) -> QueryId {
        let local_closest_peers = self.kbuckets.local_closest_peers(key, K_VALUE);
        let (query_id, requests) =
            FindNodeQuery::new_query(&mut self.queries, key.clone(), local_closest_peers);
        for (dst, request) in requests {
            self.send_message(request, dst);
        }
        query_id
    }
}

impl EventHandler for Peer {
    fn on(&mut self, event: Event) {
        self.kbuckets.add_peer(event.src);
        cast!(match event.data {
            FindNodeRequest { query_id, key } => {
                let closest_peers = self.kbuckets.local_closest_peers(&key, K_VALUE);
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
                    if let Some((dst, request)) =
                        query.on_response(event.src, query_id, closest_peers)
                    {
                        self.send_message(request, dst);
                    }
                }
            }
        })
    }
}
