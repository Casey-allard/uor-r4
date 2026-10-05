#!/usr/bin/env bash
# Pod-side bootstrap for the shared UOR-R4 GPU pods (#820; docs/labs/compute.md).
# `uor-pod up`/`uor-pod bootstrap` copies this file to /root and runs it as root:
#
#   bash /root/uor-pod-bootstrap.sh --sha FULL_SHA [--pod ID] [--with-ollama] [--off-volume]
#
# Everything slow is cached on the shared network volume (/workspace) so the
# second pod of a kind starts in seconds:
#   /workspace/toolchain/rust-<ver>-x86_64.tar, cargo-registry.tar   Rust, unpacked to /root
#   /workspace/bin/<sha>-sm<cap>/          release binaries + BUILD.json + parity.log
#   /workspace/toolchain/ollama, /workspace/ollama   Ollama binary and models (--with-ollama)
# The build itself runs on the container disk (/root/build); the source is a
# shallow fetch of the public repository at exactly --sha.
# It writes /root/.uor-pod-env (sourced by login shells and `uor-pod run` jobs)
# and starts /root/uor-reaper.sh. Prints one `BOOTSTRAP_RESULT {json}` line.
set -euo pipefail

SHA='' POD='' OLLAMA=0 OFF=0
while [ $# -gt 0 ]; do
  case $1 in
    --sha) SHA=$2; shift 2;;
    --pod) POD=$2; shift 2;;
    --with-ollama) OLLAMA=1; shift;;
    --off-volume) OFF=1; shift;;
    *) echo "unknown argument $1" >&2; exit 2;;
  esac
done
[ ${#SHA} = 40 ] || { echo "--sha must be a full 40-character commit" >&2; exit 2; }

RUST_VERSION=1.97.1
REPO_URL=https://github.com/UOR-Foundation/uor-r4.git
BINS_EXAMPLES="geometric-stack m-world mqar-bench"
BINS_BINS="chat-grade dialogue-recall-corpus mix-chat-corpus"
T0=$(date +%s)
log() { echo "[bootstrap $(date -u +%T) +$(( $(date +%s) - T0 ))s] $*"; }
# ---- binary cache for this commit and compute capability (functions; run below)
# /workspace/bin/<sha>-sm<cap>/ is shared by every pod on the volume. A build
# lock only avoids duplicate work; it never makes a pod wait long for another
# pod: a flock on the network volume can outlive a deleted pod, and a slow or
# dead builder must not idle a paid GPU pod. The holder refreshes
# <lock>.holder every HEARTBEAT_EVERY s with its phase and progress. A waiter
# polls for at most BUILD_WAIT_S (default 180 s); it stops earlier when the
# heartbeat is older than HEARTBEAT_STALE_S or shows no progress for that long,
# and then builds itself on the container disk into a private staging dir.
# Publishing is one rename(2) onto the cache path, which fails when another
# pod's build is already there, so two finished builds never mix: the first
# PASS build wins, the other is discarded (or kept privately when the cached one
# did not pass parity and cannot be moved aside).
BINROOT=${UOR_BOOTSTRAP_BINROOT:-/workspace/bin}
BUILD_WAIT_S=${UOR_BUILD_WAIT_S:-180}
HEARTBEAT_STALE_S=${UOR_BUILD_HEARTBEAT_STALE_S:-90}
HEARTBEAT_EVERY=${UOR_BUILD_HEARTBEAT_EVERY:-20}
POLL_S=${UOR_BUILD_POLL_S:-10}
POD_TAG=${POD:-$(hostname 2>/dev/null || echo pod)}
BIN='' LOCK='' HB_PID='' BUILT=0 HAVE_LOCK=0
mtime() { stat -c %Y "$1" 2>/dev/null || stat -f %m "$1" 2>/dev/null || echo 0; }
cache_ok() { [ -f "$BIN/BUILD.json" ] && grep -q '"parity": "PASS"' "$BIN/BUILD.json"; }
rename_dir() {  # SRC DST -> rename(2); fails (never merges) when DST exists and is not empty
  python3 -c 'import os, sys; os.rename(sys.argv[1], sys.argv[2])' "$1" "$2" 2>/dev/null
}
set_phase() { echo "$1" > "$LOCK.phase.$POD_TAG" 2>/dev/null || true; }
build_progress() {  # what a waiter compares between polls
  echo "phase=$(cat "$LOCK.phase.$POD_TAG" 2>/dev/null || echo start) files=$(find "${CARGO_TARGET_DIR:-/nonexistent}" -type f 2>/dev/null | wc -l | tr -d ' ')"
}
heartbeat_start() {
  # (the loop closes the lock descriptor: only this script holds the lock, and
  # the EXIT trap stops the loop when a failed build ends the script)
  ( exec 8>&-; while :; do
      echo "$POD_TAG $(date -u +%FT%TZ) $(build_progress)" > "$LOCK.holder.tmp.$POD_TAG" &&
        mv -f "$LOCK.holder.tmp.$POD_TAG" "$LOCK.holder"
      sleep "$HEARTBEAT_EVERY"
    done ) </dev/null >/dev/null 2>&1 &
  HB_PID=$!
  trap heartbeat_stop EXIT
}
heartbeat_stop() {
  if [ -n "$HB_PID" ]; then kill "$HB_PID" 2>/dev/null || true; wait "$HB_PID" 2>/dev/null || true; fi
  HB_PID=''
  rm -f "$LOCK.phase.$POD_TAG"
}

wait_for_other_build() {  # -> 0 when the cache filled meanwhile; 1 = build here (HAVE_LOCK=1 if the lock came free)
  local start now age hb last='' last_change
  start=$(date +%s); last_change=$start
  while :; do
    if cache_ok; then return 0; fi
    if flock -n 8; then HAVE_LOCK=1; log "build lock released by its holder; building here"; return 1; fi
    now=$(date +%s)
    if [ $((now - start)) -ge "$BUILD_WAIT_S" ]; then
      log "waited $((now - start))s (limit ${BUILD_WAIT_S}s) for another pod's build; building here (private staging, atomic publish)"
      return 1
    fi
    if [ -e "$LOCK.holder" ]; then age=$((now - $(mtime "$LOCK.holder"))); else age=$((now - start)); fi
    if [ "$age" -gt "$HEARTBEAT_STALE_S" ]; then
      log "build lock holder heartbeat is ${age}s old (stale: holder gone?); building here"
      return 1
    fi
    hb=$(cut -d' ' -f3- "$LOCK.holder" 2>/dev/null || true)
    if [ "$hb" != "$last" ]; then
      last=$hb; last_change=$now
    elif [ $((now - last_change)) -ge "$HEARTBEAT_STALE_S" ]; then
      log "no visible build progress from the lock holder for $((now - last_change))s (${hb:-no progress field}); building here"
      return 1
    fi
    sleep "$POLL_S"
  done
}

publish_stage() {  # STAGE -> publish to the cache path, or keep privately; sets BIN to what this pod uses
  local stage=$1 plock got=0 aside priv
  plock="$BINROOT/.$SHA-sm$CAP.publish.lock"
  exec 7>"$plock"
  if flock -w 60 7; then got=1; else log "publish lock busy for 60 s; publishing by rename only"; fi
  if cache_ok; then
    log "another pod published $BIN first (parity PASS); discarding this pod's build"
    rm -rf "$stage"
  else
    if [ -e "$BIN" ] && [ "$got" = 1 ]; then  # a cached build whose parity failed: keep it for inspection
      aside="$BIN.failed-$(date -u +%Y%m%dT%H%M%SZ)-$POD_TAG"
      log "cached build without parity PASS: moving it to $aside"
      mv "$BIN" "$aside" || true
    fi
    if rename_dir "$stage" "$BIN"; then
      log "cached $BIN"
    elif cache_ok; then
      log "another pod published $BIN first (parity PASS); discarding this pod's build"
      rm -rf "$stage"
    else
      priv="$BIN.private-$POD_TAG-$(date -u +%Y%m%dT%H%M%SZ)"
      mv "$stage" "$priv"
      log "could not publish to $BIN (occupied by a build without parity PASS); this pod uses $priv"
      BIN=$priv
    fi
  fi
  exec 7>&-
}

obtain_bin() {  # -> BIN holds release binaries + BUILD.json for SHA/CAP (built here when needed)
  local stage
  BIN=$BINROOT/$SHA-sm$CAP
  LOCK=$BINROOT/.$SHA-sm$CAP.lock
  if cache_ok; then log "cache hit: $BIN"; return 0; fi
  HAVE_LOCK=0
  exec 8>"$LOCK"
  if flock -w 5 8; then
    HAVE_LOCK=1
  else
    log "another pod is building this commit ($(head -c 200 "$LOCK.holder" 2>/dev/null || true)); waiting at most ${BUILD_WAIT_S}s"
    if wait_for_other_build; then log "cache filled by another pod meanwhile: $BIN"; exec 8>&-; return 0; fi
  fi
  if [ "$HAVE_LOCK" = 1 ]; then heartbeat_start; fi
  if cache_ok; then
    log "cache filled by another pod meanwhile: $BIN"
  else
    BUILT=1
    stage="$BINROOT/.$SHA-sm$CAP.tmp.$POD_TAG.$$"
    rm -rf "$stage"; mkdir -p "$stage"
    build_into "$stage"
    publish_stage "$stage"
  fi
  heartbeat_stop
  exec 8>&-
}
[ "${UOR_BOOTSTRAP_LIB:-0}" != 1 ] || return 0  # the dry-run tests source the functions above and stop here

mkdir -p /root/leases /workspace/uor-r4/jobs /workspace/uor-r4/pods /workspace/bin /workspace/toolchain

# ---- CUDA 12.8 toolkit (the standard image ships it; apt is the slow fallback)
CUDA=''
for c in /usr/local/cuda-12.8 /usr/local/cuda; do
  if [ -x "$c/bin/nvcc" ] && "$c/bin/nvcc" --version | grep -q 'release 12\.8'; then CUDA=$c; break; fi
done
T_CUDA=0
if [ -z "$CUDA" ]; then
  log "nvcc 12.8 missing: installing cuda-toolkit-12-8 from NVIDIA's apt repository (~15 min)"
  t=$(date +%s)
  # shellcheck disable=SC1091
  . /etc/os-release
  repo="ubuntu${VERSION_ID//./}"
  curl -fsSL -o /tmp/cuda-keyring.deb "https://developer.download.nvidia.com/compute/cuda/repos/$repo/x86_64/cuda-keyring_1.1-1_all.deb"
  dpkg -i /tmp/cuda-keyring.deb >/dev/null
  apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq cuda-toolkit-12-8 >/dev/null
  CUDA=/usr/local/cuda-12.8
  T_CUDA=$(( $(date +%s) - t ))
fi
NVCC_VERSION=$("$CUDA/bin/nvcc" --version | sed -n 's/.*release \([0-9.]*\).*/\1/p')
log "CUDA toolkit $CUDA (nvcc $NVCC_VERSION)"
for tool in cc git curl; do command -v $tool >/dev/null || { log "installing build-essential git curl"; apt-get update -qq; DEBIAN_FRONTEND=noninteractive apt-get install -y -qq build-essential git curl >/dev/null; break; }; done

# ---- GPU -> CUDA_COMPUTE_CAP (the binary cache is keyed by it)
GPU_NAME=$(nvidia-smi --query-gpu=name --format=csv,noheader | head -1)
GPU_COUNT=$(nvidia-smi --query-gpu=name --format=csv,noheader | wc -l | tr -d ' ')
case $GPU_NAME in
  *"RTX 5090"*) CAP=120;;
  *"RTX 4090"*|*L40S*|*"RTX 6000 Ada"*) CAP=89;;
  *A100*) CAP=80;;
  *H100*|*H200*) CAP=90;;
  *B200*) CAP=100;;
  *) CAP=$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader | head -1 | tr -d ' .');;
esac
log "GPU: $GPU_COUNT x $GPU_NAME -> CUDA_COMPUTE_CAP=$CAP"

# ---- Rust toolchain: cached on the volume as tarballs, unpacked to the container
# disk. Compiling straight from the network filesystem (RUSTUP_HOME on
# /workspace) left rustc IO-bound at ~20 % of one core, so the volume only holds
# the archives: rust-<ver>-x86_64.tar (rustup + toolchain) and
# cargo-registry.tar (crate sources, refreshed after each cold build).
RUST_TAR=/workspace/toolchain/rust-$RUST_VERSION-x86_64.tar
REG_TAR=/workspace/toolchain/cargo-registry.tar
export RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo
export PATH=$CARGO_HOME/bin:$CUDA/bin:$PATH
T_RUST=0
if ! cargo "+$RUST_VERSION" --version >/dev/null 2>&1; then
  t=$(date +%s)
  if [ -s "$RUST_TAR" ]; then
    log "unpacking cached Rust $RUST_VERSION from $RUST_TAR"
    tar -C /root -xf "$RUST_TAR"
  else
    log "installing Rust $RUST_VERSION (first pod on this volume) and caching it"
    curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain "$RUST_VERSION" >/dev/null 2>&1
    tar -C /root -cf "$RUST_TAR.tmp.$$" .rustup .cargo/bin .cargo/env && mv -f "$RUST_TAR.tmp.$$" "$RUST_TAR"
  fi
  T_RUST=$(( $(date +%s) - t ))
fi
if [ -s "$REG_TAR" ] && [ ! -d "$CARGO_HOME/registry" ]; then tar -C "$CARGO_HOME" -xf "$REG_TAR"; fi
RUSTC_VERSION=$(rustc "+$RUST_VERSION" --version)
# the pod's CPU quota, not the host's core count (nproc reports the host)
JOBS=$(tr '\0' '\n' < /proc/1/environ | sed -n 's/^RUNPOD_CPU_COUNT=//p' | head -1)
export CARGO_BUILD_JOBS=${JOBS:-$(nproc)}
log "Rust: $RUSTC_VERSION (local, $CARGO_BUILD_JOBS build jobs)"

# ---- binaries for this commit and compute capability (obtain_bin above)
T_BUILD=0 T_PARITY=0
build_into() {  # STAGE -> fetch, build, parity-test and stage the binaries (container-disk build)
  local stage=$1 t b ex=() bi=()
  SRC=/root/build/src-$SHA
  if [ ! -d "$SRC/.git" ]; then
    set_phase fetch
    log "fetching $SHA"
    rm -rf "$SRC"; mkdir -p "$SRC"
    git -C "$SRC" init -q
    git -C "$SRC" fetch -q --depth 1 "$REPO_URL" "$SHA"
    git -C "$SRC" checkout -q FETCH_HEAD
  fi
  export CARGO_TARGET_DIR=/root/build/target-sm$CAP CUDA_COMPUTE_CAP=$CAP
  export LD_LIBRARY_PATH=$CUDA/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
  for b in $BINS_EXAMPLES; do ex+=(--example "$b"); done
  for b in $BINS_BINS; do bi+=(--bin "$b"); done
  set_phase build
  log "cargo build --release (sm_$CAP)"
  t=$(date +%s)
  (cd "$SRC" && cargo "+$RUST_VERSION" build --release -p uor-r4-training --features cuda "${ex[@]}" "${bi[@]}")
  T_BUILD=$(( $(date +%s) - t ))
  set_phase parity
  log "build ${T_BUILD}s; parity test"
  t=$(date +%s)
  PARITY=FAIL
  if (cd "$SRC" && UOR_REQUIRE_CUDA=1 cargo "+$RUST_VERSION" test --release -p uor-r4-training --features cuda \
        --test cuda_stack_ops_parity -- --test-threads=1) > "$stage/parity.log" 2>&1; then
    PARITY=PASS
  fi
  T_PARITY=$(( $(date +%s) - t ))
  PARITY_SUMMARY=$(grep -E '^test result:' "$stage/parity.log" | tail -1 || true)
  log "parity $PARITY in ${T_PARITY}s: $PARITY_SUMMARY"
  set_phase stage
  for b in $BINS_EXAMPLES; do cp "$CARGO_TARGET_DIR/release/examples/$b" "$stage/"; done
  for b in $BINS_BINS; do cp "$CARGO_TARGET_DIR/release/$b" "$stage/"; done
  # shellcheck disable=SC2086  # word lists
  (cd "$stage" && sha256sum $BINS_EXAMPLES $BINS_BINS > SHA256SUMS)
  python3 - "$stage/BUILD.json" <<PY
import json, sys
json.dump({
  "sha": "$SHA", "compute_cap": $CAP, "gpu": "$GPU_NAME", "rustc": "$RUSTC_VERSION",
  "nvcc": "$NVCC_VERSION", "features": "cuda", "profile": "release",
  "examples": "$BINS_EXAMPLES".split(), "bins": "$BINS_BINS".split(),
  "build_seconds": $T_BUILD, "parity": "$PARITY", "parity_summary": "$PARITY_SUMMARY",
  "parity_seconds": $T_PARITY, "built_on_pod": "$POD", "built_at": "$(date -u +%FT%TZ)",
}, open(sys.argv[1], "w"), indent=1)
PY
  if tar -C "$CARGO_HOME" -cf "$REG_TAR.tmp.$POD_TAG.$$" registry git 2>/dev/null; then
    mv -f "$REG_TAR.tmp.$POD_TAG.$$" "$REG_TAR"
  else
    rm -f "$REG_TAR.tmp.$POD_TAG.$$"
  fi
}
obtain_bin
PARITY=$(python3 -c "import json; print(json.load(open('$BIN/BUILD.json'))['parity'])")

# ---- optional: Ollama judge (binary and models on the volume)
T_OLLAMA=0
if [ "$OLLAMA" = 1 ]; then
  t=$(date +%s)
  O=/workspace/toolchain/ollama
  if [ ! -x "$O/bin/ollama" ]; then
    log "installing Ollama into $O"
    mkdir -p "$O.tmp.$$"
    curl -fsSL https://ollama.com/download/ollama-linux-amd64.tgz | tar -xzf - -C "$O.tmp.$$"
    rm -rf "$O"; mv "$O.tmp.$$" "$O"
  fi
  mkdir -p /workspace/ollama
  if ! pgrep -x ollama >/dev/null; then
    OLLAMA_MODELS=/workspace/ollama nohup setsid "$O/bin/ollama" serve > /root/ollama.log 2>&1 < /dev/null &
    sleep 3
  fi
  OLLAMA_MODELS=/workspace/ollama "$O/bin/ollama" pull qwen2.5:7b > /root/ollama-pull.log 2>&1 || log "WARNING: ollama pull failed (see /root/ollama-pull.log)"
  T_OLLAMA=$(( $(date +%s) - t ))
  log "Ollama ready (${T_OLLAMA}s)"
fi

# ---- environment for shells and jobs
cat > /root/.uor-pod-env <<ENV
export RUSTUP_HOME=$RUSTUP_HOME CARGO_HOME=$CARGO_HOME CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS
export PATH=$CARGO_HOME/bin:$CUDA/bin:/workspace/toolchain/ollama/bin:\$PATH
export LD_LIBRARY_PATH=$CUDA/lib64\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}
export CUDA_COMPUTE_CAP=$CAP OLLAMA_MODELS=/workspace/ollama
export UOR_BIN=$BIN UOR_SHA=$SHA
ENV
grep -q uor-pod-env /root/.bashrc 2>/dev/null || echo '[ -f /root/.uor-pod-env ] && . /root/.uor-pod-env' >> /root/.bashrc

# ---- reaper
REAPER=not-installed
if [ -x /root/uor-reaper.sh ]; then
  if /root/uor-reaper.sh --check; then REAPER=api-ok; else REAPER=api-FAILED; fi
  if ! pgrep -f 'uor-reaper[.]sh --loop' >/dev/null; then
    nohup setsid /root/uor-reaper.sh --loop > /dev/null 2>&1 < /dev/null &
  fi
  log "reaper running (pod API check: $REAPER)"
fi

TOTAL=$(( $(date +%s) - T0 ))
echo "BOOTSTRAP_RESULT {\"pod\":\"$POD\",\"sha\":\"$SHA\",\"cap\":$CAP,\"gpu\":\"$GPU_NAME\",\"gpus\":$GPU_COUNT,\"nvcc\":\"$NVCC_VERSION\",\"off_volume\":$OFF,\"cache_hit\":$([ $BUILT = 0 ] && echo true || echo false),\"parity\":\"$PARITY\",\"reaper\":\"$REAPER\",\"seconds\":{\"total\":$TOTAL,\"cuda_install\":$T_CUDA,\"rust_install\":$T_RUST,\"build\":$T_BUILD,\"parity\":$T_PARITY,\"ollama\":$T_OLLAMA}}"
[ "$PARITY" = PASS ] || { echo "parity did not pass: see $BIN/parity.log" >&2; exit 3; }
