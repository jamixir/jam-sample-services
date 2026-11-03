# jam-sample-services

Small JAM services built from the Gray Paper and the polkavm crates only, without the JAM SDK.
They give JAM clients a known blob, a known input and a known result to check refine and
accumulate end to end.

Work in progress: the services currently target Gray Paper v0.7.2 (polkavm 0.30). The v0.8.0
port, a genesis chain spec and a full guide come next.

## hash_chain

- Refine: hashes the payload 1000 times with Blake2b-256 and returns the 32-byte result.
- Accumulate: stores that result in service storage under the Blake2b-256 hash of the payload.

## Build

Needs nightly Rust with `rust-src`.

    ./build.sh hash_chain    # writes blobs/hash_chain.jam

## Licence

Apache-2.0, see [LICENSE](LICENSE).
