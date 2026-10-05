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

# ---- binaries for this commit and compute capability
BIN=/workspace/bin/$SHA-sm$CAP
T_BUILD=0 T_PARITY=0 BUILT=0
cache_ok() { [ -f "$BIN/BUILD.json" ] && grep -q '"parity": "PASS"' "$BIN/BUILD.json"; }
if cache_ok; then
  log "cache hit: $BIN"
else
  # The lock only avoids duplicate builds (results land by atomic rename). A
  # flock on the network volume can outlive a deleted pod, so wait only while
  # the holder's heartbeat file is fresh.
  LOCK=/workspace/bin/.$SHA-sm$CAP.lock
  exec 8>"$LOCK"
  if ! flock -w 5 8; then
    held=$(( $(date +%s) - $(stat -c %Y "$LOCK.holder" 2>/dev/null || echo 0) ))
    if [ "$held" -lt 3600 ]; then
      log "another pod is building this commit ($(cat "$LOCK.holder" 2>/dev/null)); waiting up to 45 min"
      flock -w 2700 8 || log "build lock still held; building anyway"
    else
      log "stale build lock (no holder heartbeat for ${held}s); building anyway"
    fi
  fi
  echo "$POD $(date -u +%FT%TZ)" > "$LOCK.holder"
  if cache_ok; then
    log "cache filled by another pod meanwhile: $BIN"
  else
    if [ -e "$BIN" ]; then  # a cached build whose parity failed: keep it for inspection, rebuild
      log "cached build without parity PASS: moving it to $BIN.failed-$(date -u +%Y%m%dT%H%M%SZ)"
      mv "$BIN" "$BIN.failed-$(date -u +%Y%m%dT%H%M%SZ)"
    fi
    BUILT=1
    SRC=/root/build/src-$SHA
    if [ ! -d "$SRC/.git" ]; then
      log "fetching $SHA"
      rm -rf "$SRC"; mkdir -p "$SRC"
      git -C "$SRC" init -q
      git -C "$SRC" fetch -q --depth 1 "$REPO_URL" "$SHA"
      git -C "$SRC" checkout -q FETCH_HEAD
    fi
    export CARGO_TARGET_DIR=/root/build/target-sm$CAP CUDA_COMPUTE_CAP=$CAP
    export LD_LIBRARY_PATH=$CUDA/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
    ex=() bi=()
    for b in $BINS_EXAMPLES; do ex+=(--example "$b"); done
    for b in $BINS_BINS; do bi+=(--bin "$b"); done
    log "cargo build --release (sm_$CAP)"
    t=$(date +%s)
    (cd "$SRC" && cargo "+$RUST_VERSION" build --release -p uor-r4-training --features cuda "${ex[@]}" "${bi[@]}")
    T_BUILD=$(( $(date +%s) - t ))
    log "build ${T_BUILD}s; parity test"
    STAGE=/workspace/bin/.$SHA-sm$CAP.tmp.$$
    rm -rf "$STAGE"; mkdir -p "$STAGE"
    t=$(date +%s)
    PARITY=FAIL
    if (cd "$SRC" && UOR_REQUIRE_CUDA=1 cargo "+$RUST_VERSION" test --release -p uor-r4-training --features cuda \
          --test cuda_stack_ops_parity -- --test-threads=1) > "$STAGE/parity.log" 2>&1; then
      PARITY=PASS
    fi
    T_PARITY=$(( $(date +%s) - t ))
    PARITY_SUMMARY=$(grep -E '^test result:' "$STAGE/parity.log" | tail -1 || true)
    log "parity $PARITY in ${T_PARITY}s: $PARITY_SUMMARY"
    for b in $BINS_EXAMPLES; do cp "$CARGO_TARGET_DIR/release/examples/$b" "$STAGE/"; done
    for b in $BINS_BINS; do cp "$CARGO_TARGET_DIR/release/$b" "$STAGE/"; done
    # shellcheck disable=SC2086  # word lists
    (cd "$STAGE" && sha256sum $BINS_EXAMPLES $BINS_BINS > SHA256SUMS)
    python3 - "$STAGE/BUILD.json" <<PY
import json, sys
json.dump({
  "sha": "$SHA", "compute_cap": $CAP, "gpu": "$GPU_NAME", "rustc": "$RUSTC_VERSION",
  "nvcc": "$NVCC_VERSION", "features": "cuda", "profile": "release",
  "examples": "$BINS_EXAMPLES".split(), "bins": "$BINS_BINS".split(),
  "build_seconds": $T_BUILD, "parity": "$PARITY", "parity_summary": "$PARITY_SUMMARY",
  "parity_seconds": $T_PARITY, "built_on_pod": "$POD", "built_at": "$(date -u +%FT%TZ)",
}, open(sys.argv[1], "w"), indent=1)
PY
    mv "$STAGE" "$BIN"
    log "cached $BIN"
    if tar -C "$CARGO_HOME" -cf "$REG_TAR.tmp.$$" registry git 2>/dev/null; then
      mv -f "$REG_TAR.tmp.$$" "$REG_TAR"
    else
      rm -f "$REG_TAR.tmp.$$"
    fi
  fi
  exec 8>&-
fi
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
