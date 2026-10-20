# #792 — fix(#785-C1): publish node scores from the R4G1 distribution pass so the candidate walk is real

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T15:59:40Z
- **Updated:** 2026-08-18T16:42:56Z
- **Closed:** 2026-08-18T16:21:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/792
- **Merged:** 2026-08-18T16:21:04Z
- **Merge commit:** e6a18dbed4001f1ade0f976686ac643dd1dd0735
- **Base / Head:** main / UOR-Foundation:issue-785-r4g1-node-scores
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

#785 C1 (the populate arm): make the R4G1 ask-path candidate machinery real. The engine's distribution pass now publishes each scored target node's best final emission score into the caller's `node_scores` buffer (bounds-checked; n-gram hits leave it untouched so absence stays visible), `chat.rs` resets the buffer per beam step, and two contract tests pin the behavior. Before this, the buffer was never written anywhere, `predict_candidates_with_signature_lanes` skipped every node, each beam expanded to exactly one child, and the repeat penalty + `truncate(4)` downstream were dead code — the audited "beam search" was width-1 greedy (baseline audit AUD-QUAL-001; `docs/project_baseline_audit_2026_08_18.md` §14).

## Why now

#783's bundle refresh isolated the failure cleanly: the refreshed `smollm2-135m-instruct` produces grammatical English on the plain path but a prompt-invariant `regarding regarding …` loop through the R4G1 path (5/5 probes byte-identical) — the path `r4 ask` prefers. With candidates real, beam diversity exists at node-path steps and the existing repeat penalty can actually steer; the decisive live re-probe on the refreshed bundle happens on #783 right after this merges, and the C4 decode-policy experiment stays pre-declared there.

## Honesty notes

- This changes generated output on the R4G1 ask path (width-1 → genuine beam-4 at node-path steps). Current output on that path is the degenerate loop above; the live before/after lands on #783 either way.
- Candidates beyond the distribution winner carry raw emission scores (pre-existing asymmetry, deliberately not expanded here).
- No serving default, tier, profile, or artifact-byte changes; P-4 surface unchanged; zero-alloc kernels unchanged (writes into a caller-owned slice).

## Verification

`cargo fmt --check` · `cargo clippy --workspace --all-targets --all-features -- -D warnings` · `cargo test -p uor-r4-graph-runtime` (incl. the two new contract tests) · `cargo test -p uor-r4-core --test allocation_census` · root `--lib` tests · `cargo build --target wasm32-unknown-unknown --lib` · `cargo run -p xtask -- audit-limits` — all green locally; the merge queue re-runs the full ladder as the binding verdict.

References #785 #783

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

