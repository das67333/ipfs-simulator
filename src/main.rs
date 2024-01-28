use ipfs_simulator::app::App;

// Simulation setup and execution
fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .format_target(false)
        .format_timestamp(None)
        .init();

    let mut app = App::new(42);
    app.add_peers(2);
    app.run();
}

#[test]
fn dev() -> Result<(), cid::Error> {
    use ipfs_simulator::cid::*;

    let ha = HashAlgorithms::new();
    for cid in [
        // IpfsCid::from_str("QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n")?,
        IpfsCid::from_str("bafybeihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku")?,
        IpfsCid::from_chunk(
            CidVersion::V1,
            Multicodec::DagPb,
            MultihashType::Sha2_256,
            &[],
            &ha,
        )?,
    ] {
        println!("{}", cid.to_string(Multibase::Base58Btc)?);
        println!("{}", cid.to_string(Multibase::Base32Lower)?);
    }
    Ok(())
}
