use num_derive::FromPrimitive;    

/// Supported variants:
///
/// https://github.com/multiformats/multicodec/blob/master/table.csv
/// 
/// (with tag = "ipld" and status = "permanent";
/// 
/// Ethereum*, Bitcoin*, Zcash* are excluded)
#[repr(u64)]
#[derive(Debug, FromPrimitive)]
pub enum Multicodec {
    /// Raw binary
    Raw = 0x55,
    // JSON (UTF-8-encoded)
    Json = 0x0200,
    /// CBOR
    Cbor = 0x51,
    /// Libp2p Public Key
    Libp2pKey = 0x72,
    /// MerkleDAG protobuf
    DagPb = 0x70,
    /// MerkleDAG cbor
    DagCbor = 0x71,
    /// MerkleDAG JSON
    DagJson = 0x0129,
    /// Raw Git object
    GitRaw = 0x78,
}
