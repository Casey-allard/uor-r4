#!/usr/bin/env bash

# R4 Prime Router Startup Script
# Automatically launches Ollama, pulls the model, checks dependencies, and starts the server.

PROJECT_DIR="/Users/adminamn/gemini-dev"
cd "$PROJECT_DIR" || exit 1

echo "=========================================================="
echo "          R4 Prime Router — Startup Sequence              "
echo "=========================================================="

# 1. Start Ollama if not running
if ! curl -s http://localhost:11434/api/tags > /dev/null; then
    echo "[*] Ollama is not running. Launching Ollama app..."
    open -a Ollama
    
    echo "[*] Waiting for Ollama to initialize on port 11434..."
    success=0
    for i in {1..20}; do
        if curl -s http://localhost:11434/api/tags > /dev/null; then
            echo "[+] Ollama is online!"
            success=1
            break
        fi
        sleep 1
    done
    if [ $success -eq 0 ]; then
        echo "[-] Warning: Ollama did not start within 20 seconds. Make sure Ollama is installed."
    fi
else
    echo "[+] Ollama is already running."
fi

# 2. Check/pull Gemma 4 model
if command -v ollama &> /dev/null; then
    echo "[*] Checking if gemma4:e2b is installed..."
    if ! ollama list | grep -q "gemma4:e2b"; then
        echo "[*] Model gemma4:e2b not found. Pulling now (7.2 GB)..."
        ollama pull gemma4:e2b
    else
        echo "[+] Model gemma4:e2b is ready."
    fi
else
    echo "[-] Warning: 'ollama' CLI command not found. Skipping model check."
fi

# 3. Check Python dependencies
echo "[*] Verifying Python environment dependencies..."
if ! command -v python3 &> /dev/null; then
    echo "[-] Error: python3 is not installed or not in PATH."
    exit 1
fi

python3 -c "import numpy, psutil" &> /dev/null
if [ $? -ne 0 ]; then
    echo "[*] Installing missing packages (numpy, psutil, opentelemetry)..."
    pip3 install numpy psutil opentelemetry-api opentelemetry-sdk
else
    echo "[+] Python dependencies are satisfied."
fi

# 4. Run the server
echo "[*] Starting the R4 Prime Router Server..."
echo "[*] Access the dashboard at: http://localhost:8000"
echo "=========================================================="
python3 server.py
