use crate::{
    network::{NetworkAgent, UserLoadGenerator},
    peer::Peer,
    Key, PeerId, CONFIG,
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
        log::error!("{:#?}", stats);
    }

    /// Runs the simulation.
    /// You're expected to override this function to define the simulation scenario.
    pub fn run(&mut self) {
        // self.sim.step_until_time(CONFIG.kbuckets_refresh_interval);
        // for peer in self.peers.iter() {
        //     peer.borrow_mut().clear_storage();
        // }

        self.summarize_stats();
    }

    /// Runs the simulation with no queries.
    /// This is useful to measure the overhead of the simulation or
    /// to explore background tasks.
    pub fn run_scenario_no_queries(&mut self) {
        self.sim.step_until_time(3600.);
        self.summarize_stats();

        for peer in self.peers.iter() {
            peer.borrow_mut().clear_storage();
        }
    }

    /// Runs the simulation with intensive publishing.
    pub fn run_scenario_intensive_publishing(&mut self) {
        const PUBLISHING_DELAY: f64 = 0.1;
        const SIMULATION_DURATION: f64 = 3600.;
        let mut i = 0;
        while self.sim.time() < SIMULATION_DURATION {
            let idx = self.sim.gen_range(0..CONFIG.num_peers as usize);
            self.peers[idx]
                .borrow_mut()
                .publish_data(format!("data-{}", i));
            i += 1;
            self.sim.step_until_time(self.sim.time() + PUBLISHING_DELAY);
        }
        self.summarize_stats();

        for peer in self.peers.iter() {
            peer.borrow_mut().clear_storage();
        }
    }

    /// Runs the simulation with intensive retrieving.
    pub fn run_scenario_intensive_retrieving(&mut self) {
        const BLOCKS_COUNT: usize = 1_000;
        const SIMULATION_DURATION: f64 = 3600.;
        const PROPAGATION_BLOCKS_TIME_RESERVE: f64 = 10.;
        const RETRIEVING_DELAY: f64 = 0.1;
        let blocks = (0..BLOCKS_COUNT)
            .map(|i| format!("data-{}", i))
            .collect::<Vec<_>>();
        let keys = blocks
            .iter()
            .map(|block| Key::from_sha256(block.as_bytes()))
            .collect::<Vec<_>>();
        for block in blocks.iter().cloned() {
            let idx = self.sim.gen_range(0..CONFIG.num_peers as usize);
            let mut peer = self.peers[idx].borrow_mut();
            peer.publish_data(block);
        }

        self.sim.step_until_time(PROPAGATION_BLOCKS_TIME_RESERVE);

        while self.sim.time() < SIMULATION_DURATION {
            let idx = self.sim.gen_range(0..CONFIG.num_peers as usize);
            let key = keys[self.sim.gen_range(0..BLOCKS_COUNT)].clone();
            self.peers[idx].borrow_mut().retrieve_data(key);
            self.sim.step_until_time(self.sim.time() + RETRIEVING_DELAY);
        }
        self.summarize_stats();

        for peer in self.peers.iter() {
            peer.borrow_mut().clear_storage();
        }
    }

    /// Allows to measure the propagation delay of the network.
    /// Pay attention to the `retrieve_data_queries_completed` and
    /// `retrieve_data_queries_failed` fields of the statistics.
    /// 
    /// # Arguments
    /// 
    /// * `timedelta` - The time difference between the publishing and the retrieving.
    /// If `timedelta` is positive, the publishing happens first; otherwise, the retrieving.
    pub fn run_scenario_publishing_retrieving_race(&mut self, timedelta: f64) {
        const KEYS_CNT: usize = 10_000;

        let blocks = (0..KEYS_CNT)
            .map(|i| format!("file_{}", i))
            .collect::<Vec<_>>();
        let keys = blocks
            .iter()
            .map(|block| crate::Key::from_sha256(block.as_bytes()))
            .collect::<Vec<_>>();

        if timedelta >= 0. {
            for block in blocks.iter().cloned() {
                self.peers[self.sim.gen_range(0..CONFIG.num_peers) as usize]
                    .borrow_mut()
                    .publish_data(block);
            }
            self.sim.step_until_time(self.sim.time() + timedelta);
            for key in keys.iter().cloned() {
                self.peers[self.sim.gen_range(0..CONFIG.num_peers) as usize]
                    .borrow_mut()
                    .retrieve_data(key);
            }
        } else {
            for key in keys.iter().cloned() {
                self.peers[self.sim.gen_range(0..CONFIG.num_peers) as usize]
                    .borrow_mut()
                    .retrieve_data(key);
            }
            self.sim.step_until_time(self.sim.time() - timedelta);
            for block in blocks.iter().cloned() {
                self.peers[self.sim.gen_range(0..CONFIG.num_peers) as usize]
                    .borrow_mut()
                    .publish_data(block);
            }
        }

        // background tasks, such as republishing, bootstrap and
        // user load generation must be disabled
        self.sim.step_until_no_events();

        for peer in self.peers.iter() {
            peer.borrow_mut().clear_storage();
        }
        self.summarize_stats();
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
