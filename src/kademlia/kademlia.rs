const K: usize = 20;
const BUCKETS_NUM: usize = 256;

type Identifier = [u8; BUCKETS_NUM / 8];
type Address = dslab_core::Id;

// https://docs.ipfs.tech/concepts/dht/#routing-tables
// TODO: avoid heap allocations by changing Vec to SmallVec
pub struct KademliaNode {
    rooting_table: [Vec<(Identifier, Address)>; BUCKETS_NUM],
}

impl KademliaNode {
    pub fn add_node(&mut self, id: Identifier) {
        todo!()
    }

    pub fn lookup(&self, id: Identifier) -> Identifier {
        todo!()
    }

    // It is invoked in IPFS every 10 minutes
    pub fn refresh() {}
}