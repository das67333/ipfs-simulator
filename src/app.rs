use crate::{message::Message, peer::Peer};
use dslab_core::{Id, Simulation};
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::cell::RefCell;
use std::rc::Rc;

pub struct App {
    sim: Simulation,
    rng_crypt: ChaCha20Rng,
    peers: Vec<Rc<RefCell<Peer>>>,
    peer_ids: Vec<Id>,
}

impl App {
    pub fn new(seed: u64) -> Self {
        Self {
            sim: Simulation::new(seed),
            rng_crypt: ChaCha20Rng::seed_from_u64(seed),
            peers: vec![],
            peer_ids: vec![],
        }
    }

    pub fn add_peers(&mut self, peers_cnt: usize) {
        for _ in 0..peers_cnt {
            let peer = Rc::new(RefCell::new(Peer::new(&mut self.sim, &mut self.rng_crypt)));
            self.peer_ids.push(
                self.sim
                    .add_handler(peer.borrow().ipfs_peer_id(), peer.clone()),
            );
            self.peers.push(peer);
        }
    }

    pub fn run(&mut self) {
        // TODO
        let msg = Message { info: 3.8 };
        let dst = 1;
        self.peers[0].borrow_mut().send_msg(msg, dst, 0.1);
        self.sim.step_until_no_events();
        log::info!("{}", self.sim.time());
    }
}
