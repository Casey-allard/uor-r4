#!/usr/bin/env bash
set -euo pipefail

# Within-Cell Writer Qualification, Stage T Decomposition & Zero-Update B0 Job (#3267)
# Runs on 1x RTX 5090 (sm_120) leased via scripts/pod/uor-pod.

WORK_ROOT="/workspace/within-cell-writer"
DATA_ROOT="/workspace/uor-r4/local/driver/data"
DL_DIR="/workspace/dl-wcw"
STAGE_T_ARCHIVE_DIR="/workspace/stage-t-archive"
STAGE_T_DIR="/workspace/stage-t-extracted/stage-t-out"
OUT1="${WORK_ROOT}/build1-out"
OUT2="${WORK_ROOT}/build2-out"
BIN_DIR="${WORK_ROOT}/bin"

mkdir -p "${WORK_ROOT}" "${DATA_ROOT}/address-cells" "${DL_DIR}" "${STAGE_T_ARCHIVE_DIR}" "${OUT1}" "${OUT2}" "${BIN_DIR}"

echo "[$(date -u +%FT%TZ)] Unpacking driver corpus & fetching address-cells tables..."
if [ ! -f "${DATA_ROOT}/mw-balp/train/tokens.u16" ]; then
    tar -xf /workspace/uor-r4/data/ft-balp.tar -C "${DATA_ROOT}" mw-balp/train
fi
if [ ! -f "${DATA_ROOT}/ft/dev/tokens.u16" ]; then
    tar -xf /workspace/uor-r4/data/ft-dev.tar -C "${DATA_ROOT}" ft/dev
fi
if [ ! -f "${DATA_ROOT}/tokenizer.json" ]; then
    tar -xf /workspace/uor-r4/data/tokenizer.json.tar -C "${DATA_ROOT}" tokenizer.json
fi
if [ ! -f "${DATA_ROOT}/address-cells/address-structured-v1.json" ]; then
    /root/uor-store get results/claude/address-key-2026-10-25/tables/ "${DL_DIR}"
    cp "${DL_DIR}/results/claude/address-key-2026-10-25/tables/structured.json" "${DATA_ROOT}/address-cells/address-structured-v1.json"
    cp "${DL_DIR}/results/claude/address-key-2026-10-25/tables/full-vector.json" "${DATA_ROOT}/address-cells/address-full-vector-v1.json"
fi

cat > "${WORK_ROOT}/data-pins.sha256" <<EOF
3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8  ${DATA_ROOT}/mw-balp/train/tokens.u16
b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3  ${DATA_ROOT}/mw-balp/train/response_mask.u8
0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b  ${DATA_ROOT}/ft/dev/tokens.u16
a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078  ${DATA_ROOT}/ft/dev/response_mask.u8
6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1  ${DATA_ROOT}/address-cells/address-structured-v1.json
2c886736c858778677d6a7b87253af188d8787cedc5fe5acd9b3c899a10a90e2  ${DATA_ROOT}/address-cells/address-full-vector-v1.json
EOF
sha256sum -c "${WORK_ROOT}/data-pins.sha256"

echo "[$(date -u +%FT%TZ)] Fetching sealed Stage T archive from uor-store..."
if [ ! -f "${STAGE_T_ARCHIVE_DIR}/stage-t-20261025.tar.gz" ]; then
    /root/uor-store get results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/ "${STAGE_T_ARCHIVE_DIR}"
fi
# Handle either flat or nested uor-store download path
if [ -f "${STAGE_T_ARCHIVE_DIR}/results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/stage-t-20261025.tar.gz" ]; then
    cp "${STAGE_T_ARCHIVE_DIR}/results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/"* "${STAGE_T_ARCHIVE_DIR}/"
fi
(cd "${STAGE_T_ARCHIVE_DIR}" && sha256sum -c SHA256SUMS)
mkdir -p /workspace/stage-t-extracted
tar -xzf "${STAGE_T_ARCHIVE_DIR}/stage-t-20261025.tar.gz" -C /workspace/stage-t-extracted

SRC_DIR=$(ls -d /root/build/src-* | head -n 1)
echo "[$(date -u +%FT%TZ)] Syncing latest branch commit in ${SRC_DIR} and building Build 1 & Build 2..."
cd "${SRC_DIR}"
git fetch https://github.com/UOR-Foundation/uor-r4.git antigravity/m1-within-cell-writer
git checkout -f FETCH_HEAD
git rev-parse HEAD > "${WORK_ROOT}/source-head.txt"

export PATH="/usr/local/cuda/bin:/root/.cargo/bin:${PATH}"
export LD_LIBRARY_PATH="/usr/local/cuda/lib64:${LD_LIBRARY_PATH:-}"
export UOR_NVCC_ARCH=compute_120

CARGO_TARGET_DIR=/root/build/target-wcw-1 cargo build --release --locked -p uor-r4-discrete --features cuda --bin within-cell-writer-run
cp /root/build/target-wcw-1/release/within-cell-writer-run "${BIN_DIR}/within-cell-writer-run-build1"

CARGO_TARGET_DIR=/root/build/target-wcw-2 cargo build --release --locked -p uor-r4-discrete --features cuda --bin within-cell-writer-run
cp /root/build/target-wcw-2/release/within-cell-writer-run "${BIN_DIR}/within-cell-writer-run-build2"

sha256sum "${BIN_DIR}/within-cell-writer-run-build1" "${BIN_DIR}/within-cell-writer-run-build2" > "${BIN_DIR}/SHA256SUMS"
cat "${BIN_DIR}/SHA256SUMS"

echo "[$(date -u +%FT%TZ)] Running Build 1: qualify..."
"${BIN_DIR}/within-cell-writer-run-build1" qualify "${OUT1}/within-cell-qualification.json"

echo "[$(date -u +%FT%TZ)] Running Build 1: decompose-stage-t..."
"${BIN_DIR}/within-cell-writer-run-build1" decompose-stage-t "${STAGE_T_DIR}" "${DATA_ROOT}" "${OUT1}/stage-t-decomposition.json"

echo "[$(date -u +%FT%TZ)] Running Build 1: b0..."
"${BIN_DIR}/within-cell-writer-run-build1" b0 "${DATA_ROOT}" "${OUT1}/within-cell-qualification.json" "${OUT1}/stage-t-decomposition.json" "${OUT1}/b0-grade.json"

echo "[$(date -u +%FT%TZ)] Running Build 2: qualify & b0 determinism replication..."
"${BIN_DIR}/within-cell-writer-run-build2" qualify "${OUT2}/within-cell-qualification.json"
cp "${OUT1}/stage-t-decomposition.json" "${OUT2}/stage-t-decomposition.json"
"${BIN_DIR}/within-cell-writer-run-build2" b0 "${DATA_ROOT}" "${OUT1}/within-cell-qualification.json" "${OUT1}/stage-t-decomposition.json" "${OUT2}/b0-grade.json"

(cd "${WORK_ROOT}" && find build1-out build2-out bin -type f -exec sha256sum {} + | sort > SHA256SUMS)
echo "WITHIN_CELL_WRITER_JOB_COMPLETE" > "${WORK_ROOT}/EXIT_OK"
echo "[$(date -u +%FT%TZ)] Job complete."
