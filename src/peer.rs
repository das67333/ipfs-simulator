use crate::message::Message;
use dslab_core::{log_info, Event, EventHandler, Id, Simulation, SimulationContext};
use rand_chacha::ChaCha20Rng;
use rsa::{traits::PublicKeyParts, RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256};

pub struct Peer {
    ctx: SimulationContext,
    priv_key: RsaPrivateKey,
    pub_key: RsaPublicKey,
    ipfs_peer_id: String,
}

impl Peer {
    const RSA_BITS: usize = 2048;

    pub fn new(sim: &mut Simulation, rng_crypt: &mut ChaCha20Rng) -> Self {
        let priv_key =
            RsaPrivateKey::new(rng_crypt, Self::RSA_BITS).expect("Failed to create RSA key");
        let pub_key = RsaPublicKey::from(&priv_key);
        let ipfs_peer_id = {
            let mut hasher = Sha256::new();
            hasher.update(&pub_key.n().to_bytes_le());
            hasher.update(&pub_key.e().to_bytes_le());
            format!("{:x}", hasher.finalize())
        };

        let ctx = sim.create_context(&ipfs_peer_id);
        Self {
            ctx,
            priv_key,
            pub_key,
            ipfs_peer_id,
        }
    }

    pub fn ipfs_peer_id(&self) -> &str {
        &self.ipfs_peer_id
    }

    pub fn send_msg(&mut self, msg: Message, dst: Id, delay: f64) {
        self.ctx.emit(msg, dst, delay);
    }
}

impl EventHandler for Peer {
    fn on(&mut self, event: Event) {
        log_info!(
            self.ctx,
            "Received message: src={}, dst={}",
            event.src,
            event.dst
        );
    }
}
