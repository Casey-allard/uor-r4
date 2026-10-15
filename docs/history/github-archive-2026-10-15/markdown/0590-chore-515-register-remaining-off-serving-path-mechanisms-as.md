# #590 — chore(#515): register remaining off-serving-path mechanisms as open ledger claims

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T20:04:44Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-08-10T20:24:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/590
- **Merged:** 2026-08-10T20:24:15Z
- **Merge commit:** 9a00460ab570991b92368212ec45ae109a26854b
- **Base / Head:** main / UOR-Foundation:r5/ledger-register-dormant
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #515 PR-1 — register remaining off-serving-path mechanisms as `open` ledger claims

Part 1 of #515 (preserve-and-gate). Audit finding: the named negative-results
(#64 Σ-over-cloud `legacy_sum`, #456 reconstruction-null, #457 IPF probe) are a
**live comparison baseline**, a **measurement tool**, and a **"don't-build" probe**
respectively — already accounted for, not unregistered mechanisms. The real gap was
the many research / certificate / measurement / reference surfaces and never-wired
mechanisms retained **off the serving path** but absent from the register.

Per the maintainer direction (**preserve by registration, not deletion**), this
registers each as an `open` claim (measured and reported, never asserted) with an
activation gate, following the existing 6-claim pattern. **Registration only — no
module is changed.**

Newly registered (16 claims):
- **core:** `region-store`, `endomorphism`
- **graph-certify:** `holographic-encoding`, `predictive-sufficiency`,
  `shortlist-evaluator`, `anti-degeneracy` (+`perturbation`), `fairness-provenance`,
  `long-context`, `compiler-scaling`
- **graph-compiler:** `reference-compiler-ir`, `rate-distortion-compression`,
  `monograph-validator`, `behavioral-probes`, `future-state-planner`, `stage-dag`
- **cross-layer:** `patch-overlay` (`patch_induction` + `patch_chain` + `patch_lifecycle`)

Left unregistered (genuinely wired, not dormant) and **flagged for your confirmation**:
`fmm` (referenced by graph-format + `score_runtime`) and `vp_tree` (an engine
ROUT-signature nearest-neighbour optimisation with a linear fallback).

`CONFORMANCE.md` regenerated (`xtask check-model --write`): 29 ids. `audit-deferral`,
`audit-limits`, `check-model` all clean.

Next in #515: PR-2 (default-off cargo features + `--all-features` CI build) and PR-3
(a runnable `xtask gate <claim-id>` activation-gate registry).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

