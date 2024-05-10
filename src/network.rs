use crate::PeerId;
use std::{cell::RefCell, rc::Rc};

type Filter = dyn FnMut(PeerId, PeerId) -> Option<f64>;

#[derive(Clone)]
/// Represents a network agent responsible for filtering messages between peers.
///
/// By default, the network agent sends all messages with a delay of 1 time unit.
pub struct NetworkAgent {
    filter: Rc<RefCell<Filter>>,
}

impl NetworkAgent {
    /// Creates a new `NetworkAgent` with the specified filter function.
    ///
    /// The `filter` function takes two `PeerId` parameters representing the
    /// source and destination of a message, and returns an optional `f64`
    /// value representing the network delay of the message. If the filter
    /// function returns `None`, it means the message is filtered out and will
    /// not be sent. If the source and destination are the same, the message
    /// is always delivered anyway without delay.
    pub fn new(filter: impl FnMut(PeerId, PeerId) -> Option<f64> + 'static) -> Self {
        Self {
            filter: Rc::new(RefCell::new(filter)),
        }
    }

    /// Sends a message from the source peer to the destination peer and
    /// returns the network delay.
    ///
    /// If the filter function returns `None`, it means the message is filtered
    /// out and will not be sent. Otherwise, the returned `f64` value
    /// represents the network delay of the message.
    pub fn get_message_latency(&mut self, src: PeerId, dst: PeerId) -> Option<f64> {
        if src == dst {
            return Some(0.);
        }
        self.filter.borrow_mut()(src, dst)
    }
}

impl Default for NetworkAgent {
    fn default() -> Self {
        Self::new(|_, _| Some(1.))
    }
}
