# Minimalist R⁴ Terminal Client & Local Vendor API

> **Preserved interface reference:** this page documents the existing terminal
> client, launcher, and HTTP surfaces from the TLA/R4G1 serving era. It does not
> show that the route-native geometric-intelligence programme can answer a
> prompt. There is no working source-free geometric chat path yet; #961 -> #952
> -> #953 must establish lexical state, recursive attention, and generation
> before this interface can carry that claim. See the
> [current programme](geometric_intelligence_programme.md).

The interface exposes `r4 client` plus vendor-shaped endpoints such as
`POST /v1/chat/completions`, `GET /v1/models`, and `GET /uor/v1/status`.
Provider, engine, and multiplication-free claims apply only to the exact
legacy execution tier and artifact selected for a request; the existence of
the endpoint is not evidence about which mechanism produced its response.

---

## 1. Single-Command Launcher (`./r4-app.sh`)

The simplest way to run R⁴ interactively is with the zero-dependency `./r4-app.sh` launcher:

```bash
./r4-app.sh
```

### What `./r4-app.sh` does automatically:
1. **Model Selection Menu**: Presents an interactive choice between `smollm2-135m-instruct`, `smollm2-360m-instruct`, and `smollm2-1-7b-instruct`.
2. **4-Stage Pipeline Compilation & Live Progress**: Automatically runs any missing compilation stages (download, corpus compilation, R4G1 graph scoring) while displaying `claude-code`-styled live progress bars (`[█████████████░░░░░░░░] 65% (18s)`).
3. **Background Server**: Launches `r4 serve` in the background on port 8000 (or `$PORT`).
4. **Interactive Client**: Automatically opens the interactive client once ready.
5. **Signal Cleanup**: Traps `EXIT`, `INT` (`Ctrl-C`), `TERM`, `HUP` (window close), and `QUIT` (`Ctrl-\`) signals to cleanly terminate the background server process.

---

## 2. Interactive Terminal Client (`r4 client`)

You can also run the client against an existing local server instance:

```bash
cargo run --release -- client --remote http://127.0.0.1:8000/v1
```

### Rich Intro Banner & Slash Command Autocomplete
Upon launching, `r4 client` displays an ANSI R⁴ ASCII banner and shortcuts guide:

```text
  ██████╗ ██╗  ██╗     ██████╗██╗     ██╗
  ██╔══██╗██║  ██║    ██╔════╝██║     ██║
  ██████╔╝███████║    ██║     ██║     ██║
  ██╔══██╗╚════██║    ██║     ██║     ██║
  ██║  ██║     ██║    ╚██████╗███████╗██║
  ╚═╝  ╚═╝     ╚═╝     ╚═════╝╚══════╝╚═╝

R⁴ Holographic Graph & Transformerless Engine v0.1.0
Zero-Multiply Local Intelligence Runtime • Pinned Multiplication-Free Execution

Connected to local vendor endpoint: http://127.0.0.1:8000/v1/chat/completions
Active teacher model             : smollm2-135m-instruct

Commands & Shortcuts:
  • Type /help to view available slash commands (/status, /models, /clear, /quit)
  • Type / for interactive slash command suggestions & autocomplete
  • Type exit or press Ctrl-D to quit session
```

### Interactive Slash Command Completion
Typing `/` triggers interactive command suggestions:
- Typing `/` $\rightarrow$ Displays available commands list.
- Typing `/m` or `/mo` $\rightarrow$ Auto-completes and runs `/models`.
- Typing `/s` or `/st` $\rightarrow$ Auto-completes and runs `/status`.
- Typing `/h` $\rightarrow$ Auto-completes and runs `/help`.
- Typing `/q` $\rightarrow$ Auto-completes and runs `/quit`.

---

## 3. Local Vendor-Style HTTP API Endpoints

The local R⁴ server implements OpenAI/vendor-compatible endpoints for local tools, IDE integrations, and custom frontends.

### `GET /uor/v1/status`
Returns 4-stage pipeline readiness JSON for the active teacher model. (The bare
`/v1/status` path is a deprecated alias — it still works but is served with a
`Deprecation` header pointing at `/uor/v1/status`, keeping `/v1` a pure OpenAI
surface.)

### `GET /v1/models`
Returns the local model availability manifest.

### `POST /v1/chat/completions`
Accepts standard vendor Chat Completions requests.
