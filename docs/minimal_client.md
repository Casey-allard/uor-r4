# Minimalist R⁴ Terminal Client & Local Vendor API

R⁴ includes a minimalist interactive terminal client (`r4 client`) and vendor-compatible HTTP API endpoints (`POST /v1/chat/completions`, `GET /v1/models`) served 100% locally by the multiplication-free R⁴ engine.

No external LLM providers, remote APIs, or cloud services are used.

---

## 1. Single-Command Launcher (`./r4-app.sh`)

The simplest way to run R⁴ interactively is with the zero-dependency `./r4-app.sh` launcher:

```bash
./r4-app.sh
```

### What `./r4-app.sh` does automatically:
1. **Background Server**: Launches `r4 serve` in the background on port 8000 (or `$PORT`).
2. **Health Check & Loading Animation**: Displays a live braille spinner (`[*] ⠋ Initializing R⁴ local engine... (6s)`) while polling `GET /v1/models` for endpoint readiness.
3. **Interactive Client**: Automatically opens the interactive client once ready.
4. **Signal Cleanup**: Traps `EXIT`, `INT` (`Ctrl-C`), `TERM`, `HUP` (window close), and `QUIT` (`Ctrl-\`) signals to cleanly terminate the background server process and free the socket port when you exit.

---

## 2. Interactive Terminal Client (`r4 client`)

You can also run the client against an existing local server instance:

```bash
cargo run --release -- client --remote http://127.0.0.1:8000/v1
```

Or using `r4 chat`:
```bash
cargo run --release -- chat --remote http://127.0.0.1:8000/v1
```

### Features
- **Turn Prompts**: `you >` and `r4 >`.
- **Live Cooking Spinner**: Displays an animated braille spinner and seconds counter (`r4 > ⠋ cooking... (3s)`) while awaiting local server responses.
- **Turn Statistics**: Displays token count, latency, generation throughput, and engine mode upon completion:
  ```text
  you > Tell me a joke!
  r4 > Why did the scarecrow win an award? Because he was outstanding in his field.
  [stats: 14 tokens | 21846.12 ms | 0.6 tok/s | mode: teacher-oracle-fallback]
  ```

---

## 3. Local Vendor-Style HTTP API Endpoints

The local R⁴ server implements OpenAI/vendor-compatible endpoints for local tools, IDE integrations, and custom frontends.

### `GET /v1/models`
Returns the local model availability manifest:
```json
{
  "object": "list",
  "data": [
    {
      "id": "uor-r4",
      "object": "model"
    }
  ]
}
```

### `POST /v1/chat/completions`
Accepts standard vendor Chat Completions requests:

```bash
curl -X POST http://127.0.0.1:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "uor-r4",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Explain quantum routing."}
    ],
    "max_tokens": 128,
    "temperature": 0.7
  }'
```

**Response Format**:
```json
{
  "id": "chatcmpl-uor-r4-1753531200",
  "object": "chat.completion",
  "created": 1753531200,
  "model": "uor-r4",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Quantum routing leverages multiresolution topological field state transitions..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 12,
    "completion_tokens": 32,
    "total_tokens": 44
  },
  "system_fingerprint": "fp_uor_r4_local"
}
```
