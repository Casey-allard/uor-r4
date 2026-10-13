#!/usr/bin/env bash
# Pod-side bootstrap for the shared UOR-R4 GPU pods (#820; docs/labs/compute.md).
# `uor-pod up`/`uor-pod bootstrap` copies this file to /root and runs it as root:
#
#   bash /root/uor-pod-bootstrap.sh --sha FULL_SHA [--pod ID] [--with-ollama] [--off-volume] [--non-canonical]
#        [--repo-url URL] [--hf-store NAME]   (values limited to [A-Za-z0-9:/._@-])
#
# Everything slow is cached on /workspace (the shared network volume on Runpod,
# the instance disk on Vast) so the second pod of a kind starts in seconds:
#   /workspace/toolchain/rust-<ver>-x86_64.tar, cargo-registry.tar   Rust, unpacked to /root
#   /workspace/bin/<sha>-sm<cap>/          release binaries + BUILD.json + d29.log
#   /workspace/toolchain/ollama, /workspace/ollama   Ollama binary and models (--with-ollama)
# The private HF store (caseyallard/uor-r4-store, or --hf-store) backs the first
# two for pods without the volume: a missing toolchain/<file> or
# bin/<sha>-sm<cap>.tar is downloaded from it, and a pod that had to make one
# uploads it (only a complete build; no CPU parity run is required - D24). The CUDA toolkit is not mirrored there
# (NVIDIA's licence covers only listed runtime files): it comes from the image,
# else from NVIDIA's apt repository. Best effort: a store failure only costs time.
# The build itself runs on the container disk (/root/build); the source is a
# shallow fetch of the public repository at exactly --sha.
# It writes /root/.uor-pod-env (sourced by login shells and `uor-pod run` jobs)
# and starts /root/uor-reaper.sh. Prints one `BOOTSTRAP_RESULT {json}` line.
set -euo pipefail

# `--check-args` validates the arguments exactly as a real run would and exits 0
# without touching anything: `uor-pod up` runs it on the laptop before creating
# (and paying for) a pod, so a bad argument never reaches a billed pod.
SHA='' POD='' OLLAMA=0 OFF=0 NONCANON=0 CHECK_ARGS=0 ARG_REPO_URL='' ARG_HF_STORE=''
while [ $# -gt 0 ]; do
  case $1 in
    --sha|--pod) [ $# -ge 2 ] || { echo "$1 needs a value" >&2; exit 2; }
      if [ "$1" = --sha ]; then SHA=$2; else POD=$2; fi; shift 2;;
    --with-ollama) OLLAMA=1; shift;;
    --off-volume) OFF=1; shift;;
    --non-canonical) NONCANON=1; shift;;
    --repo-url|--hf-store) [ $# -ge 2 ] || { echo "$1 needs a value" >&2; exit 2; }
      # uor-pod passes these unquoted inside a remote shell string: allow no shell metacharacters
      case $2 in *[!A-Za-z0-9:/._@-]*|'') echo "$1 may contain only [A-Za-z0-9:/._@-] (got '$2')" >&2; exit 2;; esac
      if [ "$1" = --repo-url ]; then ARG_REPO_URL=$2; else ARG_HF_STORE=$2; fi; shift 2;;
    --check-args) CHECK_ARGS=1; shift;;
    *) echo "unknown argument $1" >&2; exit 2;;
  esac
done
case $SHA in
  *[!0-9a-f]*|'') echo "--sha must be a full 40-character commit (lower-case hex; got '$SHA')" >&2; exit 2;;
esac
[ ${#SHA} = 40 ] || { echo "--sha must be a full 40-character commit (got ${#SHA} characters)" >&2; exit 2; }
case $POD in *[!a-zA-Z0-9-]*) echo "--pod must be alphanumeric (got '$POD')" >&2; exit 2;; esac
if [ "$CHECK_ARGS" = 1 ]; then echo "bootstrap arguments OK"; exit 0; fi

RUST_VERSION=1.97.1
REPO_URL=${ARG_REPO_URL:-https://github.com/UOR-Foundation/uor-r4.git}
BINS_EXAMPLES="geometric-stack m-world mqar-bench"
BINS_BINS="chat-grade dialogue-recall-corpus mix-chat-corpus"
T0=$(date +%s)
log() { echo "[bootstrap $(date -u +%T) +$(( $(date +%s) - T0 ))s] $*"; }
# ---- the HF store (functions; best effort). The token is the copy uor-pod writes
# to /root: the image's HF_HOME is on /workspace (a shared volume on Runpod).
STORE=${ARG_HF_STORE:-${UOR_HF_STORE:-caseyallard/uor-r4-store}}
STORE_PUSH=${UOR_STORE_PUSH:-1}   # 0: never upload (a read-only token, a test)
if [ -f /root/.cache/huggingface/token ]; then export HF_TOKEN_PATH=/root/.cache/huggingface/token; fi
store_ready() {  # -> 0 when the hf CLI is available (installed on first use)
  command -v hf >/dev/null 2>&1 ||
    pip install -q --break-system-packages "huggingface_hub[cli]" >/dev/null 2>&1 || true
  command -v hf >/dev/null 2>&1
}
store_get() {  # REMOTE FILE -> 0 when the store had REMOTE and it is now at FILE
  local r=$1 f=$2 d
  store_ready || return 1
  mkdir -p "$(dirname "$f")" && d=$(mktemp -d "$(dirname "$f")/.store.XXXXXX") || return 1
  if hf download "$STORE" "$r" --repo-type dataset --local-dir "$d" >/dev/null 2>&1 && [ -s "$d/$r" ]; then
    mv -f "$d/$r" "$f"; rm -rf "$d"; return 0
  fi
  rm -rf "$d"; return 1
}
store_put() {  # FILE REMOTE -> 0 when uploaded
  [ "$STORE_PUSH" = 1 ] || return 1
  store_ready || return 1
  hf upload "$STORE" "$1" "$2" --repo-type dataset --commit-message "pod ${POD:-?}: $2" >/dev/null 2>&1
}
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
# is not a complete build and cannot be moved aside).
BINROOT=${UOR_BOOTSTRAP_BINROOT:-/workspace/bin}
BUILD_WAIT_S=${UOR_BUILD_WAIT_S:-180}
HEARTBEAT_STALE_S=${UOR_BUILD_HEARTBEAT_STALE_S:-90}
HEARTBEAT_EVERY=${UOR_BUILD_HEARTBEAT_EVERY:-20}
POLL_S=${UOR_BUILD_POLL_S:-10}
POD_TAG=${POD:-$(hostname 2>/dev/null || echo pod)}
BIN='' LOCK='' HB_PID='' BUILT=0 HAVE_LOCK=0
mtime() { stat -c %Y "$1" 2>/dev/null || stat -f %m "$1" 2>/dev/null || echo 0; }
# A build is usable when it was BUILT and its compiled modules passed the D29 interlock. `PASS` and the
# older `parity` key both stay accepted so builds published before this change remain valid caches
# rather than being silently rebuilt.
cache_ok() { [ -f "$BIN/BUILD.json" ] && grep -Eq '"(parity|d29)": "(PASS|NOT_RUN)' "$BIN/BUILD.json"; }
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
    log "another pod published $BIN first (build complete); discarding this pod's build"
    rm -rf "$stage"
  else
    if [ -e "$BIN" ] && [ "$got" = 1 ]; then  # an incomplete cached build: keep it for inspection
      aside="$BIN.failed-$(date -u +%Y%m%dT%H%M%SZ)-$POD_TAG"
      log "cached build is not a complete build: moving it to $aside"
      mv "$BIN" "$aside" || true
    fi
    if rename_dir "$stage" "$BIN"; then
      log "cached $BIN"
    elif cache_ok; then
      log "another pod published $BIN first (build complete); discarding this pod's build"
      rm -rf "$stage"
    else
      priv="$BIN.private-$POD_TAG-$(date -u +%Y%m%dT%H%M%SZ)"
      mv "$stage" "$priv"
      log "could not publish to $BIN (occupied by an incomplete build); this pod uses $priv"
      BIN=$priv
    fi
  fi
  exec 7>&-
}

# bin/<sha>-sm<cap>.tar in the HF store: unpacked into staging and published like
# a local build, but only when it is a complete build whose SHA256SUMS verify.
store_fetch_bin() {  # -> 0 when the cache now holds the store's build
  local r=bin/$SHA-sm$CAP.tar tarf=$BINROOT/.$SHA-sm$CAP.store.$POD_TAG.$$.tar stage=$BINROOT/.$SHA-sm$CAP.store.$POD_TAG.$$
  store_get "$r" "$tarf" || return 1
  rm -rf "$stage"; mkdir -p "$stage"
  # only plain files and directories (no links that could point a job's binary
  # elsewhere); BUILD.json must name this commit and GPU; and
  # SHA256SUMS must cover exactly the expected binaries, each one verifying
  if ! tar -tvf "$tarf" 2>/dev/null | cut -c1 | grep -qv '^[-d]$' &&
     tar --no-same-owner -xf "$tarf" -C "$stage" 2>/dev/null && store_build_ok "$stage"; then
    rm -f "$tarf"
    publish_stage "$stage"
    if cache_ok; then return 0; fi
  else
    log "WARNING: the store's $r is not a complete build with matching checksums; building here"
  fi
  rm -rf "$stage" "$tarf"; return 1
}
store_build_ok() {  # STAGE -> 0 when it is a build of SHA/CAP whose SHA256SUMS name exactly the binaries and verify
  local st=$1 want have
  [ -f "$st/BUILD.json" ] && [ -s "$st/SHA256SUMS" ] || return 1
  grep -Eq '"(parity|d29)": "(PASS|NOT_RUN)' "$st/BUILD.json" && grep -q "\"sha\": \"$SHA\"" "$st/BUILD.json" &&
    grep -Eq "\"compute_cap\": $CAP([,} ]|\$)" "$st/BUILD.json" || return 1
  # shellcheck disable=SC2086  # word lists
  want=$(printf '%s\n' $BINS_EXAMPLES $BINS_BINS | sort)
  have=$(awk '{ f = $2; sub(/^\*/, "", f); print f }' "$st/SHA256SUMS" | sort)
  [ "$want" = "$have" ] || return 1
  (cd "$st" && sha256sum -c --quiet SHA256SUMS >/dev/null 2>&1)
}
store_push_bin() {  # after a complete build here: upload it for the next pod
  local r=bin/$SHA-sm$CAP.tar tarf=$BINROOT/.$SHA-sm$CAP.push.$POD_TAG.$$.tar
  cache_ok || return 0
  if tar -C "$BIN" -cf "$tarf" . && store_put "$tarf" "$r"; then log "uploaded $r to the HF store $STORE"
  else log "WARNING: could not upload $r to the HF store (the next pod builds again)"; fi
  rm -f "$tarf"
}

REG_PUSH=0
store_push_registry() {  # after the build is published: the crate cache, when the store had none
  [ "$REG_PUSH" = 1 ] && [ -s "${REG_TAR:-}" ] || return 0
  if store_put "$REG_TAR" toolchain/cargo-registry.tar; then log "uploaded toolchain/cargo-registry.tar to the HF store"; fi
  REG_PUSH=0
}

obtain_bin() {  # -> BIN holds release binaries + BUILD.json for SHA/CAP (built here when needed)
  local stage
  BIN=$BINROOT/$SHA-sm$CAP
  LOCK=$BINROOT/.$SHA-sm$CAP.lock
  if cache_ok; then log "cache hit: $BIN"; return 0; fi
  if store_fetch_bin; then log "cache hit from the HF store: $BIN"; return 0; fi
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
    store_push_bin
    store_push_registry
  fi
  heartbeat_stop
  exec 8>&-
}
# ---- optional Ollama judge (functions; run below). Never fatal: a failed
# Ollama install or pull leaves the pod usable for training, reports
# "ollama": "FAILED"/"pull-failed" in BOOTSTRAP_RESULT and `uor-pod` records it.
# ollama.com moved its Linux build from .tgz to .tar.zst (the .tgz URL returned
# 404 on 2026-10-05 and failed 12 bootstraps), so the current asset is tried
# first, the legacy one next, then the official install.sh, whose files are
# copied onto the volume so the next pod reuses them.
OLLAMA_ASSETS=${UOR_OLLAMA_ASSETS:-https://ollama.com/download/ollama-linux-amd64.tar.zst https://ollama.com/download/ollama-linux-amd64.tgz}
OLLAMA_INSTALL_SH=${UOR_OLLAMA_INSTALL_SH:-https://ollama.com/install.sh}
OLLAMA_STATUS=off T_OLLAMA=0 OLOG=${UOR_OLLAMA_LOGDIR:-/root}
ollama_extract() {  # ARCHIVE DIR -> unpack .tar.zst or .tgz
  case $1 in
    *.zst)  # tar --zstd also needs the zstd binary, which the base image may lack
      if ! command -v zstd >/dev/null 2>&1; then
        log "installing zstd (apt) to unpack the Ollama archive"
        { apt-get install -y -qq zstd || { apt-get update -qq && apt-get install -y -qq zstd; }; } > "$OLOG/ollama-zstd.log" 2>&1 ||
          { log "WARNING: could not install zstd (see $OLOG/ollama-zstd.log)"; return 1; }
      fi
      zstd -dc "$1" | tar -xf - -C "$2";;
    *) tar -xzf "$1" -C "$2";;
  esac
}
ollama_install() {  # DEST -> 0 when DEST/bin/ollama is executable (each step checked: set -e is off here)
  local dest=$1 url tmp f
  [ -x "$dest/bin/ollama" ] && return 0
  tmp=$dest.tmp.$$
  for url in $OLLAMA_ASSETS; do
    rm -rf "$tmp"; mkdir -p "$tmp" || return 1
    f=$tmp/$(basename "$url")
    log "installing Ollama into $dest from $url"
    if curl -fsSL --retry 2 -o "$f" "$url" && ollama_extract "$f" "$tmp" && rm -f "$f" && [ -x "$tmp/bin/ollama" ]; then
      rm -rf "$dest" && mv "$tmp" "$dest" && return 0
    fi
    log "WARNING: Ollama asset $url failed"
  done
  rm -rf "$tmp"
  log "trying the official installer $OLLAMA_INSTALL_SH"
  if ! { curl -fsSL "$OLLAMA_INSTALL_SH" -o "$OLOG/ollama-install.sh" && sh "$OLOG/ollama-install.sh" > "$OLOG/ollama-install.log" 2>&1 &&
          command -v ollama >/dev/null 2>&1; }; then
    log "WARNING: Ollama installer failed (see $OLOG/ollama-install.log)"; return 1
  fi
  mkdir -p "$tmp/bin" && cp "$(command -v ollama)" "$tmp/bin/ollama" || return 1
  if [ -d /usr/local/lib/ollama ]; then mkdir -p "$tmp/lib" && cp -R /usr/local/lib/ollama "$tmp/lib/" || return 1; fi
  rm -rf "$dest" && mv "$tmp" "$dest"
}
ollama_step() {  # never fails; sets OLLAMA_STATUS ready|pull-failed|FAILED
  local t O=${UOR_OLLAMA_DIR:-/workspace/toolchain/ollama} models=${UOR_OLLAMA_MODELS:-/workspace/ollama}
  t=$(date +%s)
  if ! ollama_install "$O"; then
    OLLAMA_STATUS=FAILED T_OLLAMA=$(( $(date +%s) - t ))
    log "WARNING: Ollama install FAILED; continuing without it (the pod stays usable for training)"
    return 0
  fi
  mkdir -p "$models" || { OLLAMA_STATUS=FAILED; log "WARNING: cannot create $models"; return 0; }
  if ! pgrep -x ollama >/dev/null; then
    OLLAMA_MODELS=$models nohup setsid "$O/bin/ollama" serve > "$OLOG/ollama.log" 2>&1 < /dev/null &
    sleep 3
  fi
  if OLLAMA_MODELS=$models "$O/bin/ollama" pull qwen2.5:7b > "$OLOG/ollama-pull.log" 2>&1; then
    OLLAMA_STATUS=ready
  else
    OLLAMA_STATUS=pull-failed
    log "WARNING: ollama pull failed (see $OLOG/ollama-pull.log)"
  fi
  T_OLLAMA=$(( $(date +%s) - t ))
  log "Ollama: $OLLAMA_STATUS (${T_OLLAMA}s)"
}
[ "${UOR_BOOTSTRAP_LIB:-0}" != 1 ] || return 0  # the dry-run tests source the functions above and stop here

mkdir -p /root/leases /workspace/uor-r4/jobs /workspace/uor-r4/pods /workspace/bin /workspace/toolchain

# ---- non-canonical volume: fetch the small data set from the private Hugging
# Face dataset store (token copied by `uor-pod up`). Download only, never
# upload; any failure is a warning and the bootstrap continues. --hf-store (a
# user's own volume outside the project) also fetches, from that dataset.
if [ "$NONCANON" = 1 ] || [ -n "$ARG_HF_STORE" ]; then
  DATA=/workspace/uor-r4/data missing=''
  for f in tokenizer.json.tar ft-balp.tar ft-dev.tar step5-inputs.tar MD5SUMS hot/sieve-panel.tar; do
    if [ ! -e "$DATA/$f" ]; then missing="$missing $f"; fi
  done
  if [ -n "$missing" ]; then
    store=${ARG_HF_STORE:-${UOR_HF_STORE:-caseyallard/uor-r4-store}}
    log "non-canonical volume: fetching$missing from HF dataset $store"
    # (HF_TOKEN_PATH is set above: the image's HF_HOME is not where uor-pod puts
    # the token; without it the store answered 401 on 13 October)
    if pip install -q --break-system-packages "huggingface_hub[cli]" &&
       hf download "$store" --repo-type dataset $(for f in $missing; do printf -- "--include data/%s " "$f"; done) --local-dir /workspace/uor-r4/; then
      fetched=''
      for f in $missing; do if [ -e "$DATA/$f" ]; then fetched="$fetched $f"; fi; done
      log "HF store fetched:${fetched:- nothing}"
      if [ -f "$DATA/MD5SUMS" ] && [ -n "$fetched" ]; then
        # check only the files fetched now; files that were already there are not re-checked
        sums=$(grep -E "^[0-9a-f]+  \*?($(echo $fetched | sed 's/ /|/g; s/\./\\./g'))\$" "$DATA/MD5SUMS" || true)
        if [ -n "$sums" ]; then
          (cd "$DATA" && echo "$sums" | md5sum -c --quiet -) || log "WARNING: HF store files fail MD5SUMS (see above); continuing"
        fi
      fi
    else
      log "WARNING: HF store fetch failed; continuing without it"
    fi
  fi
fi

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
  RUST_OK=0
  if [ -s "$RUST_TAR" ] || store_get "toolchain/${RUST_TAR##*/}" "$RUST_TAR"; then
    log "unpacking cached Rust $RUST_VERSION from $RUST_TAR"
    if tar -C /root -xf "$RUST_TAR" && cargo "+$RUST_VERSION" --version >/dev/null 2>&1; then RUST_OK=1
    else log "WARNING: $RUST_TAR did not give a working Rust $RUST_VERSION; installing it with rustup"; rm -f "$RUST_TAR"; fi
  fi
  if [ "$RUST_OK" = 0 ]; then
    log "installing Rust $RUST_VERSION (not on the volume or in the store) and caching it"
    curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain "$RUST_VERSION" >/dev/null 2>&1
    tar -C /root -cf "$RUST_TAR.tmp.$$" .rustup .cargo/bin .cargo/env && mv -f "$RUST_TAR.tmp.$$" "$RUST_TAR"
    if store_put "$RUST_TAR" "toolchain/${RUST_TAR##*/}"; then log "uploaded toolchain/${RUST_TAR##*/} to the HF store"; fi
  fi
  T_RUST=$(( $(date +%s) - t ))
fi
# the crate cache: from the volume, else the store (then cargo needs crates.io
# only for crates added since); a pod that had neither uploads its own after the build
REG_FROM_STORE=0
if [ ! -s "$REG_TAR" ] && store_get toolchain/cargo-registry.tar "$REG_TAR"; then REG_FROM_STORE=1; log "crate cache from the HF store"; fi
if [ -s "$REG_TAR" ] && [ ! -d "$CARGO_HOME/registry" ] && ! tar -C "$CARGO_HOME" -xf "$REG_TAR"; then
  log "WARNING: $REG_TAR did not unpack; cargo fetches the crates from crates.io"
  rm -rf "$CARGO_HOME/registry" "$CARGO_HOME/git" "$REG_TAR"; REG_FROM_STORE=0
fi
RUSTC_VERSION=$(rustc "+$RUST_VERSION" --version)
# the pod's CPU quota, not the host's core count (nproc reports the host): Runpod
# names it; elsewhere (Vast) the cgroup's cpu.max "quota period" holds it (on
# 13 October a 16-CPU Vast pod saw nproc=128 and built 2x slower with 128 jobs)
JOBS=$(tr '\0' '\n' < /proc/1/environ | sed -n 's/^RUNPOD_CPU_COUNT=//p' | head -1)
if [ -z "$JOBS" ] && read -r CPU_Q CPU_P 2>/dev/null < /sys/fs/cgroup/cpu.max && [ "$CPU_Q" != max ] && [ "${CPU_P:-0}" -gt 0 ]; then
  JOBS=$(( (CPU_Q + CPU_P - 1) / CPU_P ))
fi
export CARGO_BUILD_JOBS=${JOBS:-$(nproc)}
log "Rust: $RUSTC_VERSION (local, $CARGO_BUILD_JOBS build jobs)"

# ---- binaries for this commit and compute capability (obtain_bin above)
T_BUILD=0 T_PARITY=0
build_into() {  # STAGE -> fetch, build and stage the binaries (container-disk build)
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
  set_phase d29
  # THE GATE IS THE D29 INTERLOCK, NOT A CPU-REFERENCE PARITY RUN.
  #
  # This step used to run `--test cuda_stack_ops_parity` on every `uor-pod up` and REFUSE the pod
  # unless it passed. Two things were wrong with that:
  #
  #   1. It is a CPU-reference test - "CPU Reference vs Native CUDA Stack Kernels" in its own header -
  #      so gating on it made every `up` depend on a prohibited execution (D24).
  #   2. It no longer compiles on main: two `StackConfig` initializers lack `rec_gates` (#2293), so the
  #      bootstrap failed before it could publish anything. **Every bootstrap was failing.**
  #
  # `d29_compiled_modules` is the compliant replacement and the stronger gate: it checks the sources
  # build.rs compiles for banned families, and with `--features cuda` it checks the built PTX itself.
  # It needs no CPU reference to be meaningful.
  log "build ${T_BUILD}s; D29 interlock (compiled modules, no runtime compiler)"
  t=$(date +%s)
  D29=FAIL
  if (cd "$SRC" && cargo "+$RUST_VERSION" test --release -p uor-r4-training --features cuda \
        --test d29_compiled_modules -- --test-threads=1) > "$stage/d29.log" 2>&1; then
    D29=PASS
  fi
  T_PARITY=$(( $(date +%s) - t ))
  PARITY_SUMMARY=$(grep -E '^test result:' "$stage/d29.log" | tail -1 || true)
  log "d29 $D29 in ${T_PARITY}s: $PARITY_SUMMARY"
  # A pod whose compiled modules contain a banned mechanism is not usable for D29 work, so this gate
  # REFUSES rather than warning - the whole point of the interlock is that presence counts.
  if [ "$D29" != PASS ]; then
    echo "D29 interlock did not pass: see $stage/d29.log" >&2
    grep -E 'panicked|assertion|FAILED' "$stage/d29.log" | head -5 >&2 || true
    exit 4
  fi
  PARITY="$D29"
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
  "build_seconds": $T_BUILD, "d29": "$PARITY", "d29_summary": "$PARITY_SUMMARY",
  "d29_seconds": $T_PARITY, "built_on_pod": "$POD", "built_at": "$(date -u +%FT%TZ)",
}, open(sys.argv[1], "w"), indent=1)
PY
  if tar -C "$CARGO_HOME" -cf "$REG_TAR.tmp.$POD_TAG.$$" registry git 2>/dev/null; then
    mv -f "$REG_TAR.tmp.$POD_TAG.$$" "$REG_TAR"
    [ "$REG_FROM_STORE" = 1 ] || REG_PUSH=1   # uploaded once the build is published
  else
    rm -f "$REG_TAR.tmp.$POD_TAG.$$"
  fi
}
obtain_bin
# The key was `parity` before the D29 gate; both are read so a cached BUILD.json written by an older
# bootstrap still reports rather than raising KeyError on a pod that has been up for hours.
PARITY=$(python3 -c "
import json
d = json.load(open('$BIN/BUILD.json'))
print(d.get('d29', d.get('parity', 'UNKNOWN')))
")

# ---- optional: Ollama judge (binary and models on the volume; never fatal)
if [ "$OLLAMA" = 1 ]; then ollama_step; fi

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
echo "BOOTSTRAP_RESULT {\"pod\":\"$POD\",\"sha\":\"$SHA\",\"cap\":$CAP,\"gpu\":\"$GPU_NAME\",\"gpus\":$GPU_COUNT,\"nvcc\":\"$NVCC_VERSION\",\"off_volume\":$OFF,\"cache_hit\":$([ $BUILT = 0 ] && echo true || echo false),\"parity\":\"$PARITY\",\"reaper\":\"$REAPER\",\"ollama\":\"$OLLAMA_STATUS\",\"seconds\":{\"total\":$TOTAL,\"cuda_install\":$T_CUDA,\"rust_install\":$T_RUST,\"build\":$T_BUILD,\"parity\":$T_PARITY,\"ollama\":$T_OLLAMA}}"
# The pod is ready when its build completed. There is no CPU parity run to wait for (D24).
[ -f "$BIN/BUILD.json" ] || { echo "no build published at $BIN: see $BIN/d29.log" >&2; exit 3; }
