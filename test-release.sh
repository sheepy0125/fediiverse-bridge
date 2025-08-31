#!/bin/sh
# build the thingy! and run it in the emulator

set -ex
cargo 3ds build --release || exit $?
azahar target/armv6*/release/fediiverse-bridge.3dsx
