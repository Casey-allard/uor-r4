#!/usr/bin/env bash
# Build source only. The workspace-relative Cargo dependency is identical on
# Wick and the workstation. Supply git or a full source custody manifest.
set -euo pipefail
umask 077
[[ $# == 3 || $# == 5 ]] || { echo "usage: build-for-wick.sh REVIEWED_ROOT REVIEWED_SHA ABS_NEW_DIR [SOURCE_MANIFEST MANIFEST_SHA]" >&2; exit 2; }
repo=$(cd "$1" && pwd -P)
revision="$2"
out="$3"
study="$repo/docs/loop/tasks/written-state-count-model/codex-17/study"
[[ "$out" == /* && "$revision" =~ ^[0-9a-f]{40}$ ]]
if [[ $# == 5 ]]; then
    [[ "$4" == /* && "$5" =~ ^[0-9a-f]{64}$ ]]
    printf '%s  %s\n' "$5" "$4" | sha256sum --check --status
    (cd "$repo" && sha256sum --check --status "$4")
else
    [[ "$(git -C "$repo" rev-parse HEAD)" == "$revision" ]]
    git -C "$repo" diff --quiet "$revision" -- crates/uor-r4-integer "$study"
    [[ -z "$(git -C "$repo" ls-files --others --exclude-standard "$study")" ]]
fi
mkdir "$out"
export CARGO_TARGET_DIR="$out/target"
cargo test --manifest-path "$study/Cargo.toml" --locked --offline
cargo build --manifest-path "$study/Cargo.toml" --locked --offline --release
cd "$out"
printf '%s\n' "$revision" > source-revision.txt
sha256sum "$study"/{Cargo.toml,Cargo.lock,src/main.rs,run-wick.sh,reuse-capture.sh} source-revision.txt target/release/uor-rank-study-bind > source-executable.sha256
