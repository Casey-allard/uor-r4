# #765 — gnaf: land the full integration matrix (#653 phase 2)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T13:58:48Z
- **Updated:** 2026-10-09T14:17:07Z
- **Closed:** 2026-08-17T14:17:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/765
- **Merged:** 2026-08-17T14:17:25Z
- **Merge commit:** 47604346b5636730138ba0a0bfc9fac804de32c8
- **Base / Head:** main / UOR-Foundation:i653-gnaf-matrix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

gnaf: land the full integration matrix (#653 phase 2)

Phase 1 (#742) vendored `proofs/wasm-gemm-gnaf/` at its audited pin,
inert and unwired. This is phase 2 of #653's own proposed shape: the
classification deliverable its Scope section names explicitly --
"Produce docs/gnaf_integration_<issue>.md with a complete integration
matrix. Classify every top-level GNAF layer and claim as adopt-active,
retain-formal-reference, defer-open, or reject."

`docs/gnaf_integration_653.md` classifies every top-level layer under
`proofs/wasm-gemm-gnaf/` (the WasmGemmGnaf Lean namespaces, `Tools/*.py`,
`.github/workflows/*.yml`, `authority/`, `model/*.json`,
`vendor/wasm-spec`, and the placeholder `Tests/`/`fixtures/`/
`artifacts/`), grounded in what the vendored tree actually contains --
read `README.md`/`CERTIFICATION.md`/`model/claims.json` directly rather
than assuming the issue text's summary still matches (it does: 45
claims, 28 formalProof / 2 authority / 3 buildEvidence / 12 open,
release status genuinely WorkloadIncomplete by design).

Decisions recorded:
- adopt-active (partial): the GNAF claim/status vocabulary was already
  adopted into `uor-r4-naf` before this repo's Lean source was ever
  vendored (#623/PR#631) -- confirmed still true (grep of
  uor-r4-graph-certify/src/server.rs finds no GNAF-vocabulary usage
  yet, so wiring it into live certificate/result records remains
  correctly unstarted, not silently done).
- defer-open, each independently scoped: the #602-#606 cost/witness
  seam, the Atlas incremental-rebuild discipline applied to a real r4
  compile seam, porting the proof-integrity Tools/*.py disciplines
  into xtask (as Rust checks, not vendored Python), CI wiring for Lean
  verification (confirmed: no lean-toolchain step exists anywhere in
  r4's own .github/workflows/ today), and the model/ registry
  cross-reference once an adapter layer exists.
- reject (for correctness transfer only): WasmGemmGnaf/Gemm/ proves
  properties of a reference GEMM module under GNAF's abstract cost
  objective, not r4's own uor-matmul or table-native kernels -- citing
  it as evidence about r4 model correctness would be exactly the
  green-washing #653's own Problem section warns against.
- retain-formal-reference: everything else (Foundation/Wasm/Universal/
  Theorems/Artifact/Conformance Lean namespaces, authority/,
  vendor/wasm-spec).

No layer is left unclassified or as unregistered dead code. This phase
does not implement any of the deferred bridges, port any tooling, or
register any CI workflow -- per #653's own non-goals, each defer-open
item is sized to become its own follow-up issue rather than one
monolithic phase 3.

Verification:
- scripts/check_claim_wording.py (passes clean -- this doc makes no
  unlinked machine-verified/exact-equivalence claims)
- cargo fmt --check
- cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
- cargo run -p xtask -- audit-limits
- cargo run -p xtask -- check-model (CM-01, 29 ids, unaffected -- this
  is a docs-only change)
- cargo metadata --no-deps: workspace package count unchanged at 15
- cargo test --workspace --offline (full suite, no failures)

Not closing #653 with this PR -- it remains a legitimate multi-phase
epic with real, evidenced remaining work (the defer-open items above),
and closing it here would misrepresent that work as done. Status
comment posted on the issue with this phase's summary and what's next.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

