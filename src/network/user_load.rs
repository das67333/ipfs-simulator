use crate::{peer::Peer, Key, CONFIG};
use dslab_core::{cast, Event, EventHandler, Simulation, SimulationContext};
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

pub struct UserLoadGenerator {
    ctx: SimulationContext,
    peers: Vec<Rc<RefCell<Peer>>>, // peers to publish and retrieve data
    blocks: Vec<String>,           // data to publish and retrieve
    keys: Vec<Key>,                // keys of the data
}

#[derive(Clone, Serialize)]
pub struct UserLoadTimer {}

impl UserLoadGenerator {
    pub fn register(sim: &mut Simulation, peers: Vec<Rc<RefCell<Peer>>>) -> Rc<RefCell<Self>> {
        let name = "user_load_generator";
        let ctx = sim.create_context(name);
        ctx.emit_self(UserLoadTimer {}, CONFIG.user_load_events_interval.unwrap());
        let blocks = (0..CONFIG.user_load_blocks_pool_size.unwrap())
            .map(|_| ctx.random_string(CONFIG.user_load_block_size.unwrap()))
            .collect::<Vec<_>>();
        let keys = blocks
            .iter()
            .map(|data| Key::from_sha256(data.as_bytes()))
            .collect::<Vec<_>>();
        let generator = Rc::new(RefCell::new(Self {
            ctx,
            blocks,
            keys,
            peers,
        }));
        sim.add_handler(name, generator.clone());
        generator
    }
}

impl EventHandler for UserLoadGenerator {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            UserLoadTimer {} => {
                let peer = self.peers[self.ctx.gen_range(0..self.peers.len())].clone();
                if self.ctx.rand() < 0.5 {
                    let random_block =
                        self.blocks[self.ctx.gen_range(0..self.blocks.len())].clone();
                    peer.borrow_mut().publish_data(random_block);
                } else {
                    let random_key = self.keys[self.ctx.gen_range(0..self.keys.len())].clone();
                    peer.borrow_mut().retrieve_data(random_key);
                }
                self.ctx
                    .emit_self(UserLoadTimer {}, CONFIG.user_load_events_interval.unwrap());
            }
        })
    }
}
