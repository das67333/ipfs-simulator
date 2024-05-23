use ipfs_simulator::app::App;
use std::time::Instant;

// Simulation setup and execution
fn main() {
    // env_logger::builder()
    //     .filter_level(log::LevelFilter::Trace)
    //     .format_target(false)
    //     .format_timestamp(None)
    //     .init();

    let mut app = App::new();
    let timer = Instant::now();
    app.run();
    println!("Simulation finished in {:?}", timer.elapsed());
}
