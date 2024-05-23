use crate::{network::NetworkAgent, peer::Peer, query::QueryTrigger, PeerId, CONFIG};
use dslab_core::{Simulation, SimulationContext};
use std::{cell::RefCell, rc::Rc};

/// Represents the application that runs the IPFS simulator.
pub struct App {
    sim: Simulation,
    peers: Vec<Rc<RefCell<Peer>>>,
    peer_ids: Vec<PeerId>,
    network: NetworkAgent,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            sim: Simulation::new(CONFIG.seed),
            peers: vec![],
            peer_ids: vec![],
            network: NetworkAgent::from_topology_and_delay_distribution(
                CONFIG.topology.clone(),
                CONFIG.delay_distribution.clone(),
            ),
        };
        app.add_peers();
        app
    }

    pub fn set_network_filter(
        &mut self,
        filter: impl FnMut(&SimulationContext, PeerId, PeerId) -> Option<f64> + 'static,
    ) {
        self.network = NetworkAgent::from_function(filter);
    }

    fn add_peers(&mut self) {
        let n = CONFIG.num_peers;
        for i in 0..n {
            let name = format!("peer-{}", i);
            let peer = Rc::new(RefCell::new(Peer::new(
                &mut self.sim,
                &name,
                self.network.clone(),
            )));
            self.peer_ids
                .push(self.sim.add_handler(&name, peer.clone()));
            self.peers.push(peer);
        }
        for i in 0..n {
            self.peers[i as usize].borrow_mut().fill_kbuckets_unfair();
        }
    }

    pub fn summarize_stats(&self) {
        let mut stats = crate::query::QueriesStats::new();
        for peer in self.peers.iter() {
            stats.merge(&peer.borrow_mut().stats());
        }
        println!("{:#?}", stats);
    }

    pub fn run(&mut self) {
        // let duration = std::env::var("DURATION")
        //     .ok()
        //     .and_then(|s| s.parse::<f64>().ok())
        //     .unwrap();
        const KEYS_CNT: usize = 10_000;

        for i in 0..60 {
            // [-0.35, 0.25]
            let duration = (i - 35) as f64 * 0.01;
            let blocks = (0..KEYS_CNT)
                .map(|i| format!("file_{}", i))
                .collect::<Vec<_>>();
            let keys = blocks
                .iter()
                .map(|block| crate::Key::from_sha256(block.as_bytes()))
                .collect::<Vec<_>>();

            if duration >= 0. {
                for block in blocks.iter().cloned() {
                    self.peers[self.sim.gen_range(0..CONFIG.num_peers) as usize]
                        .borrow_mut()
                        .publish_data(block);
                }
                self.sim.step_until_time(self.sim.time() + duration);
                for key in keys.iter().cloned() {
                    self.peers[self.sim.gen_range(0..CONFIG.num_peers) as usize]
                        .borrow_mut()
                        .retrieve_data(key);
                }
            } else {
                for key in keys.iter().cloned() {
                    self.peers[self.sim.gen_range(0..CONFIG.num_peers) as usize]
                        .borrow_mut()
                        .retrieve_data(key);
                }
                self.sim.step_until_time(self.sim.time() - duration);
                for block in blocks.iter().cloned() {
                    self.peers[self.sim.gen_range(0..CONFIG.num_peers) as usize]
                        .borrow_mut()
                        .publish_data(block);
                }
            }

            self.sim.step_until_no_events();
            for peer in self.peers.iter() {
                peer.borrow_mut().clear_storage();
            }

            // println!("Simulation time: {:.3} seconds", self.sim.time());
            let mut stats = crate::query::QueriesStats::new();
            for peer in self.peers.iter() {
                stats.merge(&peer.borrow_mut().stats());
            }
            println!(
                "{:.3} {} {} {}",
                duration,
                stats.retrieve_data_queries_started,
                stats.retrieve_data_queries_completed,
                stats.retrieve_data_queries_failed
            );
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
