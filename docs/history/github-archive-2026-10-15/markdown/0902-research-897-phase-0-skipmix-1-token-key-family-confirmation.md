# #902 — research(#897): phase-0 skipmix (1-token) key-family confirmation — SELECT-1-token

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T14:51:38Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-08-21T15:15:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/902
- **Merged:** 2026-08-21T15:15:12Z
- **Merge commit:** 9bc2b592951b80a6698a842a6b57af6b38d4b588
- **Base / Head:** main / UOR-Foundation:issue-897-skipmix-phase0
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

The S1 redesign D1 instrument SELECTED a backed-off mix and the maintainer §6-4 decision (#822, 2026-08-21) **ACTIVATED the lowering track #897**, pinning its **phase 0**: a cheap, pre-registered confirmation run that decides the lowered **key family** before any lowering engineering. The D1 comparison arm `d4skip` (1-token conditioning, +49.6‰) was stronger than the D1-selected 2-token `mix` (+30.6‰) but sat *outside* D1's consultation order, so a fresh run with the 1-token mix as PRIMARY keeps the selection disciplined.

**Outcome: `SELECT-1-token`.** The 1-token-conditioning backed-off mix (`skipmix`) clears the 25‰ opening bar and beats the D1-selected 2-token mix head-to-head. #897 lowers the 1-token skip-mix key family.

## Scope and non-goals

- **In scope:** the pre-registered phase-0 confirmation run only — a new offline reference instrument, its CID-bound result JSON, and the §6 evidence record.
- **Non-goals:** the lowering engineering itself (bounded `(token, last-token)` packed tables, composite-key hash, P-4 kernel, allocation-free steady state, witness replay, #886-style deployed-fidelity spot-check, end-to-end measurement against the frozen 20‰ floor). Table layout and mixing weights remain the maintainer design call named on #897. This PR **does not close #897** — it decides the key family and stops at that design gate.

## Acceptance-criteria status (phase-0 contract on #897)

- ✅ Confirmation run in the D1 harness mold, offline, zero teacher compute, λ = 1.0 fixed before evaluation.
- ✅ Reproduction gates reproduce the recorded D1 values **exactly**: base 246.6‰, D1 mix 277.2‰ (+30.6 [28.6, 32.5]), D1 d4skip 296.2‰ (+49.6 [46.8, 52.4]), minimal pairs 4,722, base-follow 0; double-run 2,000 positions identical.
- ✅ Head-to-head reported: **skipmix-vs-mix +25.6‰ [+23.1, +28.1]**.
- ✅ Planted conditioning-specificity null changes ≥1 prediction and sits below the positive arm: **165.3‰ (−81.3‰, 47,185 changed)**.
- ✅ Exit rule evaluated: skipmix-vs-base LB **53.6‰ ≥ 25‰** AND skipmix-vs-mix LB **23.1‰ > 0** → `SELECT-1-token`.
- ✅ Pinned to the attested #833 bundle (asserted corpus.meta CID prefix `blake3:aa9d1767…`); result recorded in §6 + CID-bound JSON.

## Tests and verification (all local gates green)

- Issue-specific: `cargo test -p uor-r4-api --release --test skipmix_confirm_897 -- --ignored` → PASS (5.2 s; result_cid `blake3:4a13095b…`).
- `cargo fmt --check` ✅ · `cargo clippy --workspace --all-targets --all-features -- -D warnings` ✅
- no_std ladder: `cargo check -p uor-r4-graph-format --no-default-features` (+ `--features alloc`) ✅
- `python3 scripts/check_claim_wording.py` ✅
- Register conformance: `xtask validate` (R1/R4/R5/gnaf) ✅ · `repo-conformance` (R2/R3) ✅
- `cargo test -p uor-r4-api` ✅ (the heavy instrument is correctly `#[ignore]`d in the default run — CI without the bundle skips it, no vacuous pass).
- The full workspace test ladder, κ-reproduction, deterministic rebuild, Gate C trend, wasm, and fuzz run in the merge queue (the binding verdict per `AGENTS.md`).

## Evidence artifacts

- `crates/uor-r4-api/tests/skipmix_confirm_897.rs` — the reference instrument (result_cid `blake3:4a13095b…`).
- `docs/skipmix_confirm_897_result.json` — CID-bound numbers on the attested #833 bundle.
- `docs/s1_redesign_instruments_822.md` §6 — the readable record.

## Compatibility and migration

None. Additive: one `#[ignore]`d integration test + two docs artifacts. No library/runtime/format/CLI change; no artifact-byte, wire-format, or serving-semantics impact.

## Conformance effects

None. No new served capability, no `model/ids.toml` / `model/ledger.toml` change, no `CONFORMANCE.md` regeneration. This is a compiler/certifier-side **reference** instrument (floats/allocation permitted); it touches no P-4 deployed kernel and no shipped error surface (R5 scans non-test code only). wasm build is unaffected (no `uor-r4-core`/router change).

## Claim-boundary changes

The claim is **Empirical**, off-serving, at the recorded execution scope (teacher-grounded held-out top-1 on the #833 bundle). It licenses only the **key-family selection** for the #897 lowering; it makes **no** deployed-serving or end-to-end claim. The frozen 20‰ end-to-end floor remains the promotion gate.

## Negative / unavailable results

None negative. The instrument is falsifiable and its teeth engaged: reproduction gates are hard asserts, the null must change ≥1 prediction and sit below a positive arm, and the corpus-CID pin fails loudly on a wrong bundle (a missing bundle SKIPs rather than vacuously passing).

## Intentionally excluded

- The `#836`-shaped lowering engineering (the rest of #897) — parked at the maintainer design gate.
- `obs-text/`, `tf1-pkg/` — untracked, never staged.

References #897

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01PHLz6svHCW6YBrsMsPMsCY


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

