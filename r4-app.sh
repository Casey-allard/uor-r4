#!/usr/bin/env bash
set -e

# R⁴ Single-Command Launcher
# Starts local uor-r4 server in background, waits for endpoint readiness, launches client, and cleans up on exit.

PORT=${PORT:-8000}
REMOTE_URL="http://127.0.0.1:${PORT}/v1"
LOG_FILE="/tmp/r4_server_${PORT}.log"

# Binary resolution
if [ -f "./target/release/r4" ]; then
    R4_BIN="./target/release/r4"
else
    echo "[*] Building r4 in release mode..."
    cargo build --release
    R4_BIN="./target/release/r4"
fi

echo "[*] Starting R⁴ local backend server on port ${PORT}..."
"$R4_BIN" serve --port "$PORT" > "$LOG_FILE" 2>&1 &
SERVER_PID=$!

# Cleanup trap covering EXIT, INT (Ctrl-C), TERM, HUP (window close), and QUIT (Ctrl-\)
cleanup() {
    if kill -0 "$SERVER_PID" 2>/dev/null; then
        echo -e "\n[*] Shutting down R⁴ local backend server (PID ${SERVER_PID})..."
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT INT TERM HUP QUIT

# Wait for server readiness via GET /v1/models
frames=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')
i=0
SEC=0

while true; do
    if curl -s "${REMOTE_URL}/models" | grep -q "uor-r4" 2>/dev/null; then
        echo -e "\r\033[K[+] R⁴ local backend server ready on port ${PORT}!"
        break
    fi

    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        echo -e "\n[-] Server failed to start. Logs from ${LOG_FILE}:"
        cat "$LOG_FILE"
        exit 1
    fi

    frame="${frames[i % ${#frames[@]}]}"
    echo -ne "\r[*] ${frame} Initializing R⁴ local engine... (${SEC}s)\033[K"
    sleep 0.5
    i=$((i + 1))
    if [ $((i % 2)) -eq 0 ]; then
        SEC=$((SEC + 1))
    fi
done

echo ""
# Launch interactive client
"$R4_BIN" client --remote "$REMOTE_URL"
