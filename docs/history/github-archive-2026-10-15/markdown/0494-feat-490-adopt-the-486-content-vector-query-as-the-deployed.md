# #494 — feat(#490): adopt the #486 content-vector query as the deployed default

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T02:29:34Z
- **Updated:** 2026-10-09T13:08:50Z
- **Closed:** 2026-08-08T02:48:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/494
- **Merged:** 2026-08-08T02:48:56Z
- **Merge commit:** 96bf9fd2bd30968be73e5267d43258aeeaf657f0
- **Base / Head:** main / UOR-Foundation:feat/adopt-content-query-490
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #490. Minimal variant: flip the retrieval query vector to the content construction; `DEFAULT_LEXICAL_WEIGHT` unchanged at 100 (one behaviour change).

## Measured gain (docs/geometry_selfmatch_486.md, 500k fixture, 46,342 windows, 500 probes)
| ranking | top-1 | MRR | recall@20 |
|---|---:|---:|---:|
| deployed (routing query, W=100) | 0.6240 | 0.7179 | 0.9720 |
| **content query, W=100 (this PR)** | **0.7840** | **0.8542** | **0.9900** |

+0.1363 MRR / +0.16 top-1 / +0.018 recall from one change, vs a +0.05 bar.

## The serde-default fix (why flipping new() alone is not enough)
The knob is `#[serde(skip)]` and the server restores router state via `import_state` on startup (`src/server.rs`), so a plain skip resets the field to `false` on the deserialize path — silently disabling the fix on the live server, the exact surface #490 targets. The default is a named function wired into **both** `new()` and `#[serde(default = ...)]`, so the deployed and deserialized routers both carry it.

## Blast radius, checked
Only `get_top_resonances_native` consumes the knob: `src/server.rs` (chat viz / welcome / RAG-context), `src/tless_uor.rs` (token priors), the WASM dashboard facade — every one wants content relevance and keeps identity/session scoping separate, so all strictly improve. `router_reconnect`, the `memory_lift`/`zeta` family, and the score pipeline build their own content-cosine and never call it → **invariant to the flip by construction**.

## Verification
fmt clean; `clippy --all-targets --all-features -D warnings` clean; all four CI tests that hit `get_top_resonances_native` pass (`provenance`, `selective_backoff`, `zeta_projection_quality`, lib `test_indexing_and_resonance_retrieval`). The `zeta_projection_quality` CI probe directly shows the effect: routed retrieval rises to **top1 1.000 / MRR 1.000** under the flip (baseline MRR 0.208).

## Binding gate
`router_reconnect` against a freshly-built scored R4G1 artifact (the run #480 could never make; artifact κ `blake3:8ef32e06…`, 7743 FWDA rows) is completing on the 500k fixture; its #421 anchor rows are **invariant to this flip by construction** (it never calls `get_top_resonances_native`) and will be appended to the record. The `W=0` full variant (+0.022 MRR more) stays available as a separately-gated follow-up.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

