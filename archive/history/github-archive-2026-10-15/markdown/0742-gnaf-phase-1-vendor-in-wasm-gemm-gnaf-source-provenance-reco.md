# #742 — GNAF phase 1: vendor in WASM-GEMM-GNAF source + provenance record

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-16T20:31:50Z
- **Updated:** 2026-10-09T13:09:09Z
- **Closed:** 2026-08-16T20:53:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/742
- **Merged:** 2026-08-16T20:53:09Z
- **Merge commit:** 0b8654c0d423c2457738f4462f74c6c819a878f9
- **Base / Head:** main / UOR-Foundation:issue-653-1-vendor-gnaf
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Phase 1 of #653: vendor in the upstream `afflom/WASM-GEMM-GNAF` repository
under `proofs/wasm-gemm-gnaf/`, and record independently-verified import
provenance. This is a **source-and-provenance-only** slice -- no wiring into
r4's own build, serve, or CI paths yet.

## What this PR does

- Imports `proofs/wasm-gemm-gnaf/` via `git subtree add` (no `--squash`) at
  pinned upstream commit `171652cd95c0b8e8620f76151b7e0c485e30ccfc`, preserving
  the 5 real upstream commits as parents in this repo's DAG (verified via
  `git log --graph`). No extra checkout step is needed for a fresh clone.
- Marks the vendored tree `linguist-vendored` in `.gitattributes` so GitHub's
  language stats and diff/blame treat it as third-party code.
- Adds `docs/gnaf_import_provenance.md`, the phase-1 provenance record:
  pinned commit, independently re-verified authority-document SHA-256
  (`5c342373b2ff809bfd607c413cafd0582d32bb097544c6597ff7d674fe99200a`, matches
  the value quoted in #653's issue text), Lean toolchain pin
  (`leanprover/lean4:v4.30.0`), dual MIT/Apache-2.0 licenses, import method,
  size (~3.5MB, no LFS needed), update policy (pin doesn't float), and an
  explicit "not in this phase" list.

## Why

The GNAF track (per the maintainer's direction) is the current priority
lever toward getting the transformerless engines to produce coherent,
syntactically correct text generation -- the practical purpose of vendoring
this project in is "language context and syntax" help, not abstract formal
verification for its own sake. Landing the source with verified provenance
first lets later phases (adapter wiring into the #602-#606 pipeline seam, CI
Lean toolchain wiring, Atlas discipline, falsifiers) build on a known-good
foundation.

## Verification

- `python3 scripts/check_claim_wording.py`: clean
- `cargo fmt --check` (workspace-wide): clean
- `cargo clippy --all-targets -- -D warnings`: clean
- `cargo run -p xtask -- audit-limits`: "no shipped crate returns an
  unsanctioned error (R5)"
- `cargo run -p xtask -- check-model`: "CONFORMANCE.md equals the model, 29
  ids (CM-01)"
- `cargo metadata --no-deps`: package count unchanged (15) -- the vendored
  Lean project has no `Cargo.toml` and is not picked up by the Cargo
  workspace

## Not in this phase

Deferred to future #653 phases: the full integration matrix
(`docs/gnaf_integration_653.md`), adapter wiring, CI Lean toolchain wiring,
Atlas discipline, falsifiers. The vendored tree is deliberately dead weight
from r4's own build/serve perspective today.

References #653

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

