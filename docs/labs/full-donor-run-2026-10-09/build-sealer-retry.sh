#!/usr/bin/env bash
set -euo pipefail
ROOT=/workspace/uor-r4/codex/sol-full-donor-20261009
export PATH=/root/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/root/codex/full-donor-target CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2
export UOR_BUILD_SOURCE_COMMIT=4f7eee35b250b6d5bf9e250fc0ea00d356998695
cd /root/codex/full-donor-src
test "$(git rev-parse HEAD)" = "$UOR_BUILD_SOURCE_COMMIT"
test -z "$(git status --porcelain)"
mkdir "$ROOT/evidence/sealer-retry-attempt1"
/usr/bin/time -v cargo build --release --locked -p uor-r4-core --example native_historical_version > "$ROOT/evidence/sealer-retry-attempt1/build.log" 2>&1
mkdir -p /root/codex/prototype-target/release/examples
cp "$CARGO_TARGET_DIR/release/examples/native_historical_version" /root/codex/prototype-target/release/examples/native_historical_version
cp "$CARGO_TARGET_DIR/release/examples/native_historical_version" "$ROOT/runtime/native_historical_version"
sha256sum "$ROOT/runtime/native_historical_version" > "$ROOT/evidence/sealer-retry-attempt1/sealer.sha256"
printf 'SEALER_RETRY_PASS\n'
