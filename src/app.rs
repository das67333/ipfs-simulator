use crate::{network::NetworkAgent, peer::Peer, PeerId, CONFIG};
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
            let mut peer = self.peers[i as usize].borrow_mut();
            for j in (0..n).map(|_| self.sim.gen_range(0..n)) {
                peer.add_peer(j, 0.0);
            }
        }
    }

    pub fn run(&mut self) {
        // for peer in self.peers.iter_mut() {
        //     peer.borrow_mut().find_random_node(QueryTrigger::Manual);
        // }
        let key = self.peers[0].borrow_mut().publish_data("hahaha".to_owned());
        println!("Key: {:?}", key);

        // let mut steps_cnt = 0;
        // while self.sim.step() && self.sim.time() < 3600.0 {
        //     steps_cnt += 1;
        // }
        self.sim.step_for_duration(3600.);
        
        self.peers[0].borrow_mut().retrieve_data(key);
        self.sim.step_for_duration(3600.);

        // println!("Simulation finished in {} steps", steps_cnt);
        println!("Simulation time: {:.3} seconds", self.sim.time());

        let mut stats = crate::query::QueriesStats::new();
        for peer in self.peers.iter() {
            stats.merge(&peer.borrow_mut().stats());
        }
        let (total, correct) = (
            stats.find_node_queries_completed,
            stats.closest_peers_correct,
        );
        println!(
            "Correctness: {}/{} = {:.3}",
            correct,
            total * *crate::K_VALUE,
            correct as f64 / (total * *crate::K_VALUE) as f64
        );
        println!("Stats: {:#?}", stats);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
