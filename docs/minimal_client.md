<!-- freshness: sources=archive/legacy-cli/**,crates/uor-r4-wasm-router/src/chat.rs,crates/uor-r4-wasm-router/src/main.rs,crates/uor-r4-wasm-router/src/server.rs checked=a70d6189df3368575e745d351ed692cb000e9388 -->
# Minimalist R⁴ Terminal Client & Local Vendor API

> **Preserved interface reference:** this page documents the existing terminal
> client, launcher, and HTTP surfaces from the TLA/R4G1 serving era. It does not
> show that the route-native geometric-intelligence programme can answer a
> prompt. There is no working source-free geometric chat path yet; #961 -> #952
> -> #953 must establish lexical state, recursive attention, and generation
> before this interface can carry that claim. See the
> [current programme](geometric_intelligence_programme.md).

The interface exposes the `client` subcommand plus vendor-shaped endpoints such as
`POST /v1/chat/completions`, `GET /v1/models`, and `GET /uor/v1/status`.
Provider, engine, and multiplication-free claims apply only to the exact
legacy execution tier and artifact selected for a request; the existence of
the endpoint is not evidence about which mechanism produced its response.

---

## 1. The archived launcher (`archive/legacy-cli/r4-app.sh`)

The zero-dependency launcher `r4-app.sh` and the `uor-r4-cli` script it starts are kept in `archive/legacy-cli/`. They no longer run as they are: `uor-r4-cli` calls `cargo run --bin r4`, and no crate in the workspace builds a binary named `r4` any more. The list below records what the launcher did.

### What `r4-app.sh` did automatically:
1. **Model Selection Menu**: Presents an interactive choice between `smollm2-135m-instruct`, `smollm2-360m-instruct`, and `smollm2-1-7b-instruct`.
2. **4-Stage Pipeline Compilation & Live Progress**: Automatically runs any missing compilation stages (download, corpus compilation, R4G1 graph scoring) while displaying `claude-code`-styled live progress bars (`[█████████████░░░░░░░░] 65% (18s)`).
3. **Background Server**: Launches `r4 serve` in the background on port 8000 (or `$PORT`).
4. **Interactive Client**: Automatically opens the interactive client once ready.
5. **Signal Cleanup**: Traps `EXIT`, `INT` (`Ctrl-C`), `TERM`, `HUP` (window close), and `QUIT` (`Ctrl-\`) signals to cleanly terminate the background server process.

---

## 2. Interactive Terminal Client (`client`)

The client is the `client` subcommand of the `uor-r4-wasm-router` binary ([`crates/uor-r4-wasm-router/src/main.rs`](../crates/uor-r4-wasm-router/src/main.rs)). Run it against an existing local server instance:

```bash
cargo run --release -p uor-r4-wasm-router -- client --remote http://127.0.0.1:8000/v1
```

### Intro banner and slash commands
Upon launching, the client prints an ANSI ASCII banner and then a shortcuts guide ([`chat.rs`](../crates/uor-r4-wasm-router/src/chat.rs)):

```text
UOR-R4 Holographic Graph & Transformerless Engine v0.1.0
Zero-Multiply Local Intelligence Runtime • Pinned Multiplication-Free Execution

Connected to local vendor endpoint: http://127.0.0.1:8000/v1/chat/completions
Active teacher model             : smollm2-135m-instruct
Active synthesis engine          : <engine>

Commands & Shortcuts:
  • Type /help to view available slash commands (/status, /models, /engine, /corpus, /export, /reset, /clear, /quit)
  • Type / for interactive slash command suggestions & autocomplete
  • Type exit or press Ctrl-D to quit session
```

### Slash command completion
Typing `/` lists the slash commands: `/help`, `/status`, `/models`, `/switch`, `/engine`, `/corpus`, `/compile`, `/audit`, `/clear`, `/reset`, `/export`, `/quit` and `/exit` (another name for `/quit`). Typing more of a command narrows the list to the commands that start with what has been typed, for example `/mo` to `/models` and `/st` to `/status`.

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
