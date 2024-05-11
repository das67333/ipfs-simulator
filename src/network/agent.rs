use dslab_core::SimulationContext;

use crate::PeerId;
use std::{cell::RefCell, rc::Rc};

use super::{DelayDistribution, Topology};

type Agent = dyn FnMut(&SimulationContext, PeerId, PeerId) -> Option<f64>;

#[derive(Clone)]
/// Represents an agent responsible for managing network communication
/// between peers in a simulation.
///
/// Default network agent sends all messages with a delay of 1 time unit.
pub struct NetworkAgent {
    filter: Rc<RefCell<Agent>>,
}

impl NetworkAgent {
    /// Creates a new `NetworkAgent` with the specified filter function.
    ///
    /// The `filter` function takes three parameters: a `SimulationContext` reference
    /// used for sampling from distributions, and two `PeerId` parameters representing
    /// the source and destination of a message.
    ///
    /// If the filter function returns `None`, it means the message is filtered out
    /// and will not be sent. Otherwise, the returned `f64` value represents the
    /// network delay of the message. If the source and destination are the same,
    /// the message is guaranteed to be delivered instantly.
    pub fn from_function(
        filter: impl FnMut(&SimulationContext, PeerId, PeerId) -> Option<f64> + 'static,
    ) -> Self {
        Self {
            filter: Rc::new(RefCell::new(filter)),
        }
    }

    /// Creates a new `NetworkAgent` with the specified topology and delay distribution.
    ///
    /// The `topology` parameter represents the network topology, which determines the
    /// connectivity between peers. The `distr` parameter represents the delay distribution,
    /// which is used to sample network delays.
    pub fn from_topology_and_delay_distribution(
        topology: Topology,
        distr: DelayDistribution,
    ) -> Self {
        let filter = move |ctx: &SimulationContext, src: PeerId, dst: PeerId| {
            if topology.check_access(src, dst) {
                Some(ctx.sample_from_distribution(&distr))
            } else {
                None
            }
        };
        Self::from_function(filter)
    }

    /// Samples the delay of a message between two peers.
    ///
    /// If the function returns `None`, it means the message is filtered out
    /// and will not be sent. Otherwise, the returned `f64` value represents the
    /// network delay of the message. If the source and destination are the same,
    /// the function returns `Some(0.)`.
    pub fn sample_message_delay(
        &mut self,
        ctx: &SimulationContext,
        src: PeerId,
        dst: PeerId,
    ) -> Option<f64> {
        if src == dst {
            return Some(0.);
        }
        self.filter.borrow_mut()(ctx, src, dst)
    }
}

impl Default for NetworkAgent {
    fn default() -> Self {
        Self::from_function(|_, _, _| Some(1.))
    }
}
