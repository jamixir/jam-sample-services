#!/bin/sh
# Builds a service into blobs/<service>.jam, e.g. ./build.sh hash_chain
set -e
cd "$(dirname "$0")"
SERVICE=${1:?usage: ./build.sh <service>}
cargo build -q --release --manifest-path pack/Cargo.toml
PACK=pack/target/release/jam-pack
TARGET=$(RUSTUP_TOOLCHAIN=nightly $PACK target)
RUSTFLAGS="" cargo +nightly build -q --release --manifest-path "$SERVICE/Cargo.toml" \
  --target "$TARGET" -Zjson-target-spec -Zbuild-std=core,alloc
BIN=$(sed -n 's/^name = "\(.*\)"/\1/p' "$SERVICE/Cargo.toml" | head -1)
ELF=$SERVICE/target/$(basename "$TARGET" .json)/release/$BIN
$PACK "$ELF" "blobs/$SERVICE.jam"
