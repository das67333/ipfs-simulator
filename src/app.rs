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
        let n = CONFIG.num_peers as usize;
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
            const K: usize = 40; ///////////////////////////////////////////////////////////
            let mut peer = self.peers[i].borrow_mut();
            for _ in 0..K {
                peer.add_peer(self.peer_ids[self.sim.gen_range(0..n)]);
            }
        }
    }

    pub fn run(&mut self) {
        for peer in self.peers.iter_mut() {
            peer.borrow_mut().find_random_node();
        }

        let mut steps_cnt = 0;
        while self.sim.step() {
            steps_cnt += 1;
        }
        println!("Simulation finished in {} steps", steps_cnt);

        // let t = self
        //     .peers
        //     .iter()
        //     .map(|peer| peer.borrow_mut().evaluate_queries())
        //     .sum::<f64>()
        //     / self.peers.len() as f64;
        // println!("Peers stats: {}", t);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
