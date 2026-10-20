# #678 — api: pin the OpenAI spec + compatibility matrix (#654 phase A)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T23:00:30Z
- **Updated:** 2026-10-09T14:17:07Z
- **Closed:** 2026-08-13T23:16:24Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/678
- **Merged:** 2026-08-13T23:16:24Z
- **Merge commit:** fd3d4c22b46445206717ee84e85ed76de9e31c6b
- **Base / Head:** main / UOR-Foundation:i654a-openai-pin-matrix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #677 (phase-A sub-issue of #654). **Definition-only foundation** for the R4 OpenAI wire-compatibility profile — no server route or wire DTO changes (those are phases B+, tracked under #654).

## What lands
- **`profiles/openai/openapi.yaml`** — the upstream `openai/openai-openapi` spec vendored byte-for-byte at commit `11854aef` (`openapi.yaml` git blob `b4e4080c`, OpenAPI 3.1.0 / API 2.3.0, MIT). Vendored, not fetched at build or serve time (per the #654 non-goal).
- **`profiles/openai/compatibility_matrix.json`** — every one of the **288** spec operations classified `supported` / `unsupported` / `not-applicable` with rationale:
  - **4 supported** = the phase-1 text-serving profile: `createChatCompletion` (`POST /chat/completions`), `createResponse` (`POST /responses`), `listModels` (`GET /models`), `retrieveModel` (`GET /models/{model}`).
  - **20 unsupported** = text/inference-adjacent ops deliberately out of phase 1 (legacy `POST /completions`, embeddings, stored/stateful chat+response management, `?beta=true` variants) — recorded so support is **never implied by omission**.
  - **264 not-applicable** = OpenAI platform operations outside a local R4 text model.
- **`scripts/gen_openai_compat_matrix.py`** — deterministic, dependency-free generator; regenerate on a reviewed spec bump.
- **`tests/openai_profile_pin.rs`** — the **CI drift gate**: the vendored spec is byte-identical to the pinned blake3; the matrix covers **exactly** the spec's operation set (a spec bump or matrix edit that diverges fails, both directions); the `supported` set is exactly the pinned phase-1 profile so it cannot silently widen.
- **`profiles/openai/README.md`** — the pin, retrieval/update procedure, the reviewed spec-bump process, and an honest claim-status statement.

## Why phase A first
This is the anchor the rest of #654 references — no wire DTO, route, or "OpenAI-compatible" claim may point at anything but this pin, and CI now detects unreviewed drift. It maps directly to #654's first acceptance criterion ("a pinned, versioned compatibility matrix covers every operation… CI detects unreviewed drift").

## Verification (local, pre-push)
- `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` (clean)
- `cargo build --target wasm32-unknown-unknown --lib` (clean)
- `cargo test --test openai_profile_pin` — 3 pass (byte-identity, exact coverage, pinned supported set)
- Register conformance R1–R5; the `check_claim_wording.py` gate passes. No new dependencies.

Next phases under #654: **B** truthful DTOs + error envelope + `/v1/models` required fields; **C** chat-completions conformance + tokenizer usage; **D** SSE; **E** `/v1/responses`; **F** official-SDK smoke tests; **G** control-route separation.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

