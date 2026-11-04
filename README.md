# jam-sample-services

Small JAM services built from the Gray Paper and the polkavm crates only, without the JAM SDK.
They give JAM clients a known blob, a known input and a known result to check refine and
accumulate end to end.

Work in progress: the services currently target Gray Paper v0.7.2 (polkavm 0.30). The v0.8.0
port and a full guide come next.

## Genesis

`chainspecs/tiny.json` is a JIP-4 chain spec for the tiny parameters with the six JIP-5 dev
validators. Its services:

| Service | Code | Code hash |
|---|---|---|
| 0 | `blobs/null_authorizer.pvm` (authorizer only, CC0) | `f8d86b97d65319a078e5840f1614c296a5254217794dcc910e72ca174e3c2e86` |
| 7 | `blobs/hash_chain.jam` | `de1e859a1732ed5c2680697b74bf72720e32353aedc37462f231d05e8c09932d` |

Every core's authorizer pool and queue hold the null authorizer, so any work package that names
it (code host service 0, empty configuration) is authorized.

`tools/genesis.exs` regenerates it from a Jamixir checkout, which provides the state encoding:

    cd jamixir && MIX_ENV=test mix run ../jam-sample-services/tools/genesis.exs

## hash_chain

- Refine: hashes the payload 1000 times with Blake2b-256 and returns the 32-byte result.
- Accumulate: stores that result in service storage under the Blake2b-256 hash of the payload.

Check, with the payload `hello jam` sent to service 7:

| | |
|---|---|
| refine output | `b6ba2c4f8af4c504711a66f5f39921c8222da18e55de018a9911233237e21ed4` |
| storage key | `d582eb39b397d24e5d14d935fbc08ffe533ed47fd559ee697c6669e6547ba461` |
| storage value | the refine output |

With jamt (GP v0.7.2 build) against a node's RPC:

    jamt --rpc ws://127.0.0.1:19800/ws -c 0 item 7 "hello jam"
    jamt --rpc ws://127.0.0.1:19800/ws inspect storage 7 0xd582eb39b397d24e5d14d935fbc08ffe533ed47fd559ee697c6669e6547ba461

## Build

Needs nightly Rust with `rust-src`.

    ./build.sh hash_chain    # writes blobs/hash_chain.jam

## Licence

Apache-2.0, see [LICENSE](LICENSE).
