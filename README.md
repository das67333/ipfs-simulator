# IPFS Simulator
#### Supported by IPFS:
1) Multibase algorithms: [multibase.csv](https://github.com/multiformats/multibase/blob/master/multibase.csv); **implemented**: 8/8 "final" + 7/7 "draft" + 7/9 "experimental" + 1/4 "reserved" (identity encodings).
2) Multicodecs: [table.csv](https://github.com/multiformats/multicodec/blob/master/table.csv) (with tag = "ipld"); **included in enum**: all with status = "permanent" except for cryptocurrencies (Ethereum*, Bitcoin*, Zcash*).
3) Multihashes: [table.csv](https://github.com/multiformats/multicodec/blob/master/table.csv) (with tag = "multihash"); **included in enum**: all with status = "permanent"; **implemented**: sha2-* (7/15).