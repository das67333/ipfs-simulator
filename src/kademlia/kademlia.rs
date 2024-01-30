use super::bitarray::BitArray;

const BUCKET_MAX_SIZE: usize = 20;
const KEYSPACE_SIZE: usize = 256;
const BITARR_LEN_BYTES: usize = KEYSPACE_SIZE / 8;

type Key = BitArray<BITARR_LEN_BYTES>;
type Address = dslab_core::Id;

// https://docs.ipfs.tech/concepts/dht/#routing-tables
// TODO: reduce memory consumption
pub struct KademliaNode {
    key: Key,
    rooting_table: [Vec<(Key, Address)>; KEYSPACE_SIZE],
}

impl KademliaNode {
    pub fn try_insert_node(&mut self, key: Key, addr: Address) {
        if key == self.key {
            return;
        }
        let i = (self.key ^ key).leading_zeros();
        if self.rooting_table[i].len() < BUCKET_MAX_SIZE {
            self.rooting_table[i].push((key, addr));
        }
    }

    pub fn lookup(&self, key: Key) -> Key {
        // Load the K closest peers to X from our routing table into the query-queue.
        // Allowing up to 10 concurrent queries, grab the peer closest to X and ask them who are the K closest peers to X?
        // When a query to a peer finishes, add those results to the query-queue.
        // Pull the next-closest peer off the queue and query them.
        // The query terminates whenever the closest known three peers to X have been successfully queried without any timeouts or errors.
        // After the query is done, take the K closest peers that have not failed and return them.

        todo!()
    }

    // It is invoked in IPFS every 10 minutes
    pub fn refresh() {
        // for every non-empty bucket up to 15th
        // select a random key that fits in bucket and do a lookup
        // search for ourselves
    }
}
