# #222 — feat(cli): add minimalist r4 client wrapper, vendor chat API, and single-command launcher

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-26T12:48:55Z
- **Updated:** 2026-10-09T14:17:04Z
- **Closed:** 2026-07-26T13:01:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/222
- **Merged:** 2026-07-26T13:01:19Z
- **Merge commit:** 67fb1496fb366f24fb575bfd7e360928b85256d7
- **Base / Head:** main / UOR-Foundation:feature/minimal-r4-client
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

This PR adds a minimalist terminal client (`r4 client` / `r4 chat --remote`), local vendor-style HTTP API endpoints (`POST /v1/chat/completions` and `GET /v1/models`), and a single-command launcher script (`./r4-app.sh`).

### Key Features
- **Local Vendor-Style API Endpoints (`src/server.rs`)**: Exposes `POST /v1/chat/completions` and `GET /v1/models` served 100% locally by the local R⁴ Rust engine.
- **Minimalist Interactive Terminal Client (`src/chat.rs`)**: Features turn prompts (`you >` and `r4 >`), a non-blocking worker thread architecture, an animated braille spinner with a live `cooking... (Ns)` timer, and turn metrics.
- **Single-Command Launcher (`./r4-app.sh`)**: Starts the server in the background, animates engine initialization, launches the interactive client, and traps `EXIT`, `INT` (`Ctrl-C`), `TERM`, `HUP`, and `QUIT` signals for clean server process teardown.
- **Documentation (`docs/minimal_client.md` & `README.md`)**: Comprehensive usage guide and quickstart.

### Verification & Compliance
- `cargo check --workspace`: Passed clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: Passed clean (zero warnings).
- `cargo fmt --check`: Passed clean.
- `python3 scripts/check_claim_wording.py`: Passed claim-wording gate.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

