# #860 — docs/governance: publish R4 Intelligence Completion Plan + evidence-first issue forms (#829)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-19T23:07:02Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-08-19T23:27:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/860
- **Merged:** 2026-08-19T23:27:59Z
- **Merge commit:** d1d1f7357c44b7bcd30765a38236f7f56a40ef04
- **Base / Head:** main / UOR-Foundation:issue-829-completion-plan
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

The repo has strong evidence discipline but no authoritative post-v0.1 completion plan or issue forms. `AGENTS.md` still named the original graph-compiler plan as project direction; the old `Phase 0`–`Phase 11` milestones remained open with no recorded disposition; and several live summaries still risked being read against superseded interpretations of #784 / #811 / #605-#804.

## Implemented outcome

One canonical plan (`docs/r4_intelligence_completion_plan.md`) is adopted as authoritative for **post-v0.1 intelligence sequencing** — the readable mirror of GitHub programme root **#820** — and every public entry point now links to it. Two evidence-first issue forms encode the repository run/conformance contracts. Live summaries are reconciled and the historical milestones get a recorded disposition, all without rewriting any measurement history.

## Scope (issue #829 checklist)

- [x] `docs/r4_intelligence_completion_plan.md`: #820 hierarchy (F0 + S0–S7 native children), dependency order, promotion gates, kill/redesign + global falsifiers, programme invariants, definition of completion.
- [x] #859 documented as the cross-cutting **F0** lane + the two-level proof strategy (reference math #859 → production refinement #856).
- [x] Proof-asset boundaries distinguished: R4 prime-router Lean package vs `research/riemann-lean` vs vendored GNAF (`proofs/wasm-gemm-gnaf`) vs `uor-r4-proof-model` Rust obligations + Kani vs the Hologram/R4 monograph vs target-binary (deployed-serving) evidence.
- [x] Mechanism boundaries distinguished: W(3,3) visualization/router vs the `f64` geometric router vs R4G1 graph scoring vs dormant route-attention vs the normative production path.
- [x] Entry points updated: `AGENTS.md`, `ROADMAP.md`, `docs/RESEARCH.md`, README roadmap link.
- [x] Status banner added to `docs/r4_graph_compiler_implementation_plan.md` (retained engineering plan; superseded only for sequencing).
- [x] `.github/DOCUMENTATION_OVERHAUL_PLAN.md` marked **superseded** with live-assertion corrections.
- [x] Current-summary prose after #784 / #811 / #605-#804 corrected (append-only for records): #784 = continuation-distribution convergence (not full-depth code collision); #811 did not establish semantic abstention; #605/#804 S1 verdict FAIL, operator dormant.
- [x] W(3,3) / R4G1 scoring / route-attention reference work / normative path distinguished.
- [x] `.github/ISSUE_TEMPLATE/research-experiment.yml` (hypothesis, pinned identities, reachability, nulls, cheap instrument, thresholds, outcome branches, cost, evidence record, claim status).
- [x] `.github/ISSUE_TEMPLATE/implementation.yml` (execution scope, dependencies, acceptance, compatibility, conformance mapping, verification, documentation, claim status).
- [x] 12 historical milestones audited; disposition recorded (completion-plan milestone-disposition table) without repurposing titles. Live milestone descriptions annotated separately (governance action, evidenced on the issue).

## Non-goals (respected)

- No runtime behavior or empirical-threshold changes.
- No deletion/rewrite of historical measurement records (`docs/<topic>_<issue>.md` untouched — append-only).
- No speculative roadmap work assigned to maintainers.

## Acceptance criteria

- [x] One canonical plan owns stage order; all public entry points link to it without duplicating a drifting phase table.
- [x] Current docs state #784 found continuation-distribution convergence rather than full-depth code collision, and that #811 did not establish semantic abstention.
- [x] Both issue forms validate (schema-checked; see evidence).
- [x] Every old milestone has an explicit recorded disposition and stays distinguishable from `R4 Intelligence / S0–S7`.

## Tests and evidence

- `python3 scripts/check_claim_wording.py` → **passed** (no unlinked machine-verified/exact-equivalence wording).
- Issue-form schema validation (types, unique ids, required attributes, dropdown options) → **PASS** for both forms.
- Relative-link sanity: every path referenced by the new plan resolves.
- `cargo fmt --check` → clean (no Rust/toml changed).
- Full required ladder (tests, clippy, no_std, κ, wasm, fuzz, Gate C trend) runs in the merge queue; this diff changes only Markdown + issue-form YAML, so those verdicts are unchanged from `main`.

## Compatibility / conformance

Documentation & GitHub governance only. Existing issue URLs, old milestone names, artifact formats, and CLI/runtime behavior are stable. No new capability ID; RF-01…RF-29 mappings and claim boundaries preserved. `CONFORMANCE.md` (generated) is **not** touched.

## Claim-boundary changes

Reference/offline/certifier/dormant/normative/deployed-serving evidence scopes are explicitly separated in the new plan and the overhaul-plan corrections; no capability is promoted. This closes #829 only as a documentation/governance milestone.

## Negative / unavailable results

None. The one item delivered outside the git diff (annotating the 12 live GitHub milestone descriptions) is a governance action recorded on the issue.

## Files intentionally excluded

Untracked `obs-text/` and `tf1-pkg/` (unrelated in-progress/local artifacts) were deliberately not staged.

Closes #829


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

