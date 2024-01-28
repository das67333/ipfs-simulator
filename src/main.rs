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
    let s_v0 = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n";
    let s_v1 = "bafybeihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku";
    let cids = [
        IpfsCid::from_str(s_v0)?,
        IpfsCid::from_chunk(
            CidVersion::V0,
            Multicodec::DagPb,
            MultihashType::Sha2_256,
            &[],
            &ha,
        )?,
        IpfsCid::from_str(s_v1)?,
        IpfsCid::from_chunk(
            CidVersion::V1,
            Multicodec::DagPb,
            MultihashType::Sha2_256,
            &[],
            &ha,
        )?,
    ];
    let cids_v1 = cids.clone().map(|cid| cid.into_v1().unwrap());
    assert!(cids_v1.iter().all(|cid| cid == &cids_v1[0]));
    assert_eq!(cids[0].to_string(Multibase::Base58Btc)?, s_v0);
    assert_eq!(cids[1].to_string(Multibase::Base58Btc)?, s_v0);
    assert_eq!(cids[2].to_string(Multibase::Base32Lower)?, s_v1);
    assert_eq!(cids[3].to_string(Multibase::Base32Lower)?, s_v1);
    Ok(())
}
