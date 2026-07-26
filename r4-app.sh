#!/usr/bin/env bash
set -e

# Restore standard terminal settings immediately
stty sane 2>/dev/null || true

# R⁴ Single-Command Launcher & Model Compilation Orchestrator
# Supports model selection, live progress bars, 4-stage graph compilation, and background server teardown.

PORT=${PORT:-8000}
REMOTE_URL="http://127.0.0.1:${PORT}/v1"
LOG_FILE="/tmp/r4_server_${PORT}.log"
LAST_MODEL_FILE=".uor-models/last_model.txt"
FRAMES=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')

mkdir -p .uor-models

# Binary resolution
if [ -f "./target/release/r4" ]; then
    R4_BIN="./target/release/r4"
else
    echo "[*] Building r4 executable in release mode..."
    cargo build --release
    R4_BIN="./target/release/r4"
fi

MODEL_CHOICE=${MODEL_CHOICE:-""}

# If --menu flag is passed, force menu
if [ "$1" == "--menu" ] || [ "$1" == "-m" ]; then
    MODEL_CHOICE=""
fi

# Auto-load last model choice if available and not overridden
if [ -z "$MODEL_CHOICE" ] && [ -f "$LAST_MODEL_FILE" ]; then
    MODEL_CHOICE=$(cat "$LAST_MODEL_FILE" 2>/dev/null || echo "1")
    echo -e "[*] Auto-loading last used model setting: option [${MODEL_CHOICE}] (use './r4-app.sh --menu' to change)"
fi

# If no model choice established, show menu
if [ -z "$MODEL_CHOICE" ]; then
    echo -e "\n┌─────────────────────────────────────────────────────────────────────────────┐"
    echo -e "│  R⁴ Local Zero-Multiply Model Selection                                      │"
    echo -e "├─────────────────────────────────────────────────────────────────────────────┤"
    echo -e "│  1) SmolLM2-135M-Instruct  [Fast & Ultra-Light, ~270MB]                    │"
    echo -e "│  2) SmolLM2-360M-Instruct  [Balanced Quality, ~720MB]                      │"
    echo -e "│  3) SmolLM2-1.7B-Instruct  [High-Fidelity Teacher, ~3.4GB]                 │"
    echo -e "└─────────────────────────────────────────────────────────────────────────────┘"
    read -p "Select model [1-3, default 1]: " MODEL_INPUT
    MODEL_CHOICE=${MODEL_INPUT:-1}
fi

# Save choice to last_model.txt
echo "$MODEL_CHOICE" > "$LAST_MODEL_FILE"

case "$MODEL_CHOICE" in
    2)
        MODEL_NAME="smollm2-360m-instruct"
        HF_REPO="HuggingFaceTB/SmolLM2-360M-Instruct"
        HF_REV="9d9ff7299a9a3b6d289ff100d0246a48d88c0326"
        ;;
    3)
        MODEL_NAME="smollm2-1-7b-instruct"
        HF_REPO="HuggingFaceTB/SmolLM2-1.7B-Instruct"
        HF_REV="main"
        ;;
    *)
        MODEL_NAME="smollm2-135m-instruct"
        HF_REPO="HuggingFaceTB/SmolLM2-135M-Instruct"
        HF_REV="7e27bd9f95328f0f3b08261d1252705110c806f8"
        ;;
esac

SOURCE_DIR=".uor-models/sources/${MODEL_NAME}"
COMPILED_DIR=".uor-models/compiled/${MODEL_NAME}"
GRAPH_DIR="${COMPILED_DIR}/graph"
SCORE_FILE="${GRAPH_DIR}/score.r4g1"

# Function to render a claude-code styled progress bar
render_progress() {
    local stage_lbl="$1"
    local percent="$2"
    local sec="$3"
    local frame_idx="$4"

    local bar_width=20
    local filled=$(( percent * bar_width / 100 ))
    local empty=$(( bar_width - filled ))
    local bar=""
    for ((j=0; j<filled; j++)); do bar="${bar}█"; done
    for ((j=0; j<empty; j++)); do bar="${bar}░"; done

    local frame="${FRAMES[frame_idx % ${#FRAMES[@]}]}"
    echo -ne "\r[*] ${frame} ${stage_lbl}: [${bar}] ${percent}% (${sec}s)\033[K"
}

# Stage 1: Check or Download Source Weights
if [ ! -d "$SOURCE_DIR" ]; then
    echo -e "\n[*] [Stage 1/4] Downloading pinned HF teacher weights (${MODEL_NAME})..."
    SEC=0
    i=0
    
    "$R4_BIN" download --repository "$HF_REPO" --revision "$HF_REV" --name "$MODEL_NAME" > /tmp/r4_dl.log 2>&1 &
    DL_PID=$!
    
    while kill -0 "$DL_PID" 2>/dev/null; do
        PCT=0
        if [ -f /tmp/r4_dl.log ]; then
            PCT=$(grep -oE '[0-9]+%' /tmp/r4_dl.log | tail -n 1 | tr -d '%' || echo 0)
        fi
        render_progress "[Stage 1/4] Downloading HF source" "${PCT:-0}" "$SEC" "$i"
        sleep 0.5
        i=$((i + 1))
        if [ $((i % 2)) -eq 0 ]; then SEC=$((SEC + 1)); fi
    done
    wait "$DL_PID" || true
    echo -e "\r\033[K[+] [Stage 1/4] Download complete: ${SOURCE_DIR}"
else
    echo -e "[✓] [Stage 1/4] Pinned teacher source exists: ${SOURCE_DIR}"
fi

# Stage 2 & 3: Check or Compile Transformerless Bundle & Graph
if [ ! -f "${COMPILED_DIR}/tless_artifacts.bin" ] || [ ! -f "$SCORE_FILE" ]; then
    echo -e "\n[*] [Stage 2/4] Compiling zero-multiply observation corpus for ${MODEL_NAME}..."
    mkdir -p "$COMPILED_DIR" "$GRAPH_DIR"
    
    SEC=0
    i=0
    "$R4_BIN" compile --source "$SOURCE_DIR" --output "$COMPILED_DIR" --seconds 300 --target 50000 --sequence-length 128 > /tmp/r4_compile.log 2>&1 &
    CP_PID=$!
    
    while kill -0 "$CP_PID" 2>/dev/null; do
        PCT=0
        if [ -f /tmp/r4_compile.log ]; then
            PCT=$(grep -oE '[0-9]+%' /tmp/r4_compile.log | tail -n 1 | tr -d '%' || echo 0)
        fi
        render_progress "[Stage 2/4] Compiling observation corpus" "${PCT:-0}" "$SEC" "$i"
        sleep 0.5
        i=$((i + 1))
        if [ $((i % 2)) -eq 0 ]; then SEC=$((SEC + 1)); fi
    done
    wait "$CP_PID" || true
    echo -e "\r\033[K[+] [Stage 2/4] Transformerless bundle compilation complete!"

    echo -e "\n[*] [Stage 3/4] Inducing multiresolution cover & scoring R4G1 residual graph..."
    SEC=0
    i=0
    META_PATH="${COMPILED_DIR}/corpus.meta"
    if [ ! -f "$META_PATH" ]; then META_PATH="${COMPILED_DIR}/c_meta.bin"; fi
    RECS_PATH="${COMPILED_DIR}/corpus.records"
    if [ ! -f "$RECS_PATH" ]; then RECS_PATH="${COMPILED_DIR}/c_recs.bin"; fi

    "$R4_BIN" transformerless score \
        --corpus-meta "$META_PATH" \
        --corpus-recs "$RECS_PATH" \
        --artifacts "${COMPILED_DIR}/tless_artifacts.bin" \
        --out "$GRAPH_DIR" > /tmp/r4_score.log 2>&1 &
    SC_PID=$!

    while kill -0 "$SC_PID" 2>/dev/null; do
        render_progress "[Stage 3/4] Scoring R4G1 residual graph" "$(( (i * 10) % 100 ))" "$SEC" "$i"
        sleep 0.5
        i=$((i + 1))
        if [ $((i % 2)) -eq 0 ]; then SEC=$((SEC + 1)); fi
    done
    wait "$SC_PID" || true
    echo -e "\r\033[K[+] [Stage 3/4] R4G1 Scored graph ready: ${SCORE_FILE}"
else
    echo -e "[✓] [Stage 2/4] Transformerless bundle compiled: ${COMPILED_DIR}"
    echo -e "[✓] [Stage 3/4] Scored R4G1 residual graph compiled: ${SCORE_FILE}"
fi

# Stage 4: Launch Backend Server & Client
echo -e "\n[*] [Stage 4/4] Launching local R⁴ backend server on port ${PORT}..."
"$R4_BIN" serve --port "$PORT" \
    --tless-artifacts "${COMPILED_DIR}/tless_artifacts.bin" \
    --tless-store "${COMPILED_DIR}/tless_store.bin" \
    --r4g1-artifact "$SCORE_FILE" > "$LOG_FILE" 2>&1 &
SERVER_PID=$!

cleanup() {
    if kill -0 "$SERVER_PID" 2>/dev/null; then
        echo -e "\n[*] Shutting down R⁴ local backend server (PID ${SERVER_PID})...."
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
    stty sane 2>/dev/null || true
}
trap cleanup EXIT INT TERM HUP QUIT

i=0
SEC=0
while true; do
    if curl -s "${REMOTE_URL}/models" | grep -q "uor-r4" 2>/dev/null; then
        echo -e "\r\033[K[+] [Stage 4/4] R⁴ local backend server ready on port ${PORT} (${MODEL_NAME})!"
        break
    fi

    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        echo -e "\n[-] Server failed to start. Logs from ${LOG_FILE}:"
        cat "$LOG_FILE"
        exit 1
    fi

    frame="${FRAMES[i % ${#FRAMES[@]}]}"
    echo -ne "\r[*] ${frame} Initializing R⁴ zero-multiply engine... (${SEC}s)\033[K"
    sleep 0.5
    i=$((i + 1))
    if [ $((i % 2)) -eq 0 ]; then SEC=$((SEC + 1)); fi
done

echo ""
stty sane 2>/dev/null || true
"$R4_BIN" client --remote "$REMOTE_URL"
