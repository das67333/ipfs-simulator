use crate::{
    network::{NetworkAgent, UserLoadGenerator},
    peer::Peer,
    PeerId, CONFIG,
};
use dslab_core::{Simulation, SimulationContext};
use std::{cell::RefCell, rc::Rc};

/// Represents the application that runs the IPFS simulator.
pub struct App {
    sim: Simulation,
    peers: Vec<Rc<RefCell<Peer>>>,
    peer_ids: Vec<PeerId>,
    network: NetworkAgent,
    user_load: Option<Rc<RefCell<UserLoadGenerator>>>,
}

impl App {
    /// Creates a new `App` instance and adds the peers to the simulation.
    pub fn new() -> Self {
        let mut app = Self {
            sim: Simulation::new(CONFIG.seed),
            peers: vec![],
            peer_ids: vec![],
            network: NetworkAgent::from_topology_and_delay_distribution(
                CONFIG.topology.clone(),
                CONFIG.delay_distribution.clone(),
            ),
            user_load: None,
        };
        if let Some(path) = CONFIG.log_file_path.as_ref() {
            simple_logging::log_to_file(path, CONFIG.log_level_filter).unwrap();
        } else {
            simple_logging::log_to_stderr(CONFIG.log_level_filter);
        }
        app.add_peers();
        if CONFIG.enable_user_load_generation {
            app.user_load = Some(UserLoadGenerator::register(&mut app.sim, app.peers.clone()));
        }
        app
    }

    /// Changes the network filter of the application.
    /// The filter is a function that takes the simulation context, the source peer ID, and the
    /// destination peer ID, and returns the delay between the two peers.
    ///
    /// The initial network filter is retrieved from the configuration file.
    pub fn set_network_filter(
        &mut self,
        filter: impl FnMut(&SimulationContext, PeerId, PeerId) -> Option<f64> + 'static,
    ) {
        self.network = NetworkAgent::from_function(filter);
    }

    /// Adds the peers to the simulation.
    /// The number of peers is retrieved from the configuration file.
    fn add_peers(&mut self) {
        let n = CONFIG.num_peers;
        let width = (n - 1).to_string().len();
        for i in 0..n {
            let name = format!("peer-{:01$}", i, width);
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

    /// Extracts the statistics from the peers and logs them.
    pub fn summarize_stats(&self) {
        let mut stats = crate::query::QueriesStats::new();
        for peer in self.peers.iter() {
            stats.merge(&peer.borrow_mut().stats());
        }
        log::info!("{:#?}", stats);
    }

    /// Runs the simulation.
    /// You're expected to override this function to define the simulation scenario.
    pub fn run(&mut self) {
        self.sim.step_until_time(CONFIG.kbuckets_refresh_interval);
        // for peer in self.peers.iter() {
        //     peer.borrow_mut().clear_storage();
        // }

        self.summarize_stats();
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
