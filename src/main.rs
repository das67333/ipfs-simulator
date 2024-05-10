use ipfs_simulator::app::App;
use std::time::Instant;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub fn heapsize() -> usize {
    let epoch: jemalloc_ctl::epoch_mib = jemalloc_ctl::epoch::mib().unwrap();
    let allocated: jemalloc_ctl::stats::allocated_mib =
        jemalloc_ctl::stats::allocated::mib().unwrap();

    // update jemalloc's stats
    epoch.advance().unwrap();

    // get the memory usage
    allocated.read().unwrap()
}

// Simulation setup and execution
#[inline(never)]
fn main() {
    // env_logger::builder()
    //     .filter_level(log::LevelFilter::Trace)
    //     .format_target(false)
    //     .format_timestamp(None)
    //     .init();

    let timer = Instant::now();
    let mut app = App::new(42);

    app.set_network_filter(move |_, _| Some(1.));
    app.add_peers(10_000);
    app.run();
    println!("Simulation finished in {:?}", timer.elapsed());

    println!("Heap size: {} GB", heapsize() as f64 * 1e-9);
}
