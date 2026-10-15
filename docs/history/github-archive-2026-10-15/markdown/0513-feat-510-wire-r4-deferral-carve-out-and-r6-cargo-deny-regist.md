# #513 — feat(#510): wire R4 (deferral carve-out) and R6 (cargo-deny) register gates

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T23:23:20Z
- **Updated:** 2026-10-09T13:08:51Z
- **Closed:** 2026-08-08T23:54:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/513
- **Merged:** 2026-08-08T23:54:29Z
- **Merge commit:** 2d2ceb4b91f9e4f8191db32260c998abfb1148be
- **Base / Head:** main / UOR-Foundation:feat/510-r4-r6-gates
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #273. R1/R2/R3 landed with the register machinery; this brings **R4** and **R6** from advisory-stub to enforced, and assesses **R5**.

## R4 — audit-deferral, the open-claim carve-out
A deferral marker is a violation only when its line does not cite an `open` claim in `model/ledger.toml`. R4 now enforces *nothing deferred **and unregistered***. Falsifiable: an unregistered marker still fails, a cited one passes. Real violations resolved honestly:
- `graph.rs` **kept** — `build_graph()` (never-wired Phase-2/4 builder, no call sites) is restored and registered as the `open` claim `graph-construction-dormant`, its `unimplemented!` marker citing the claim. Preserved per maintainer direction (keep every mechanism, align by registration not removal), same pattern as the packed-routing placeholders.
- `packed_kernels.rs` `evaluate_routing_program`/`evaluate_typed_transition` (#159 Phase-2 placeholders, unreferenced by serving) registered as the `open` claim `packed-routing-dormant`; markers cite it. **Kept, not deleted** — the geometric-routing surface is a live direction (matches the four existing dormant `open` claims).
- two "for now" comments reworded to present tense (they described current scope).
- `window_paths.rs` bad-magic test bytes `b"XXXX"`→`b"badmagic"` (false positive on test data).

## R6 — cargo-deny over the real graph
`deny.toml` allow-lists the two pinned git sources (uor-addr, UOR-Framework) — the revision, not the org, is trusted. Reconciled: MIT declared on the workspace crates (repo LICENSE); BSD-2-Clause + BSL-1.0 added to the licence allow-list (six permissive third-party crates, named in deny.toml); internal path + pinned git deps given explicit version reqs so the wildcard ban stays meaningful. `cargo deny check` green across advisories/bans/licences/sources.

## R5 — assessed, not yet wired
`audit-limits` reports **352** shipped `Result` returns over unsanctioned error types — an incremental register-as-you-touch migration. Tracked as the remaining #510 work; the register-conformance CI job stays **advisory** until R5 lands, then the whole job is promoted to required.

## Wiring
`just audit-deferral`, `just deny`; `vv-register` now runs R1/R2/R3/R4/R6; the CI job runs `audit-deferral` + `cargo-deny-action`.

Local: register gate green (R1/R2/R3/R4/R6), workspace builds, touched-crate tests pass, Cargo.lock unchanged.

Refs #510 (stays open for R5).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

