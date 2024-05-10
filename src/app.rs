use crate::{network::NetworkAgent, peer::Peer, PeerId};
use dslab_core::Simulation;
use std::{cell::RefCell, rc::Rc};

pub struct App {
    sim: Simulation,
    peers: Vec<Rc<RefCell<Peer>>>,
    peer_ids: Vec<PeerId>,
    network: NetworkAgent,
}

impl App {
    pub fn new(seed: u64) -> Self {
        Self {
            sim: Simulation::new(seed),
            peers: vec![],
            peer_ids: vec![],
            network: NetworkAgent::default(),
        }
    }

    pub fn set_network_filter(
        &mut self,
        filter: impl FnMut(PeerId, PeerId) -> Option<f64> + 'static,
    ) {
        self.network = NetworkAgent::new(filter);
    }

    pub fn add_peers(&mut self, peers_cnt: usize) {
        for i in 0..peers_cnt {
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
        for i in 0..peers_cnt {
            const K: usize = 40; ///////////////////////////////////////////////////////////
            let mut peer = self.peers[i].borrow_mut();
            for _ in 0..K {
                peer.add_peer(self.peer_ids[self.sim.gen_range(0..peers_cnt)]);
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
        //     .map(|peer| peer.borrow().stats())
        //     .sum::<usize>();
        // println!("Peers stats: {}", t);
    }
}
