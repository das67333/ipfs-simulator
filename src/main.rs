use ipfs_simulator::app::App;
use std::time::Instant;

fn main() {
    let mut app = App::new();
    let timer = Instant::now();
    // app.run();
    app.run_scenario_publishing_retrieving_race(-0.2);
    println!("Simulation finished in {} (real) seconds", timer.elapsed().as_secs_f64());
}
