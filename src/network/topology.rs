use crate::PeerId;

/// Represents different network topologies.
#[derive(Clone, Debug)]
pub enum Topology {
    /// A full network topology where all peers are connected to each other.
    Full,
    /// A ring network topology where each peer is connected to its two neighbors.
    /// The first and last peers are also connected.
    Ring { first_id: PeerId, last_id: PeerId },
    /// A star network topology where all peers are connected to a central peer.
    Star { center_id: PeerId },
}

impl Topology {
    /// Checks if access is allowed from one peer to another based on the network topology.
    ///
    /// # Arguments
    ///
    /// * `from` - The ID of the peer from which access is requested.
    /// * `to` - The ID of the peer to which access is requested.
    ///
    /// # Returns
    ///
    /// Returns `true` if access is allowed, `false` otherwise.
    pub fn check_access(&self, from: PeerId, to: PeerId) -> bool {
        match self {
            Topology::Full => true,
            Topology::Ring { first_id, last_id } => {
                let (mut a, mut b) = (from, to);
                if a > b {
                    std::mem::swap(&mut a, &mut b);
                }
                a + 1 == b || (a == *first_id && b == *last_id)
            }
            Topology::Star { center_id } => from == *center_id || to == *center_id,
        }
    }
}
