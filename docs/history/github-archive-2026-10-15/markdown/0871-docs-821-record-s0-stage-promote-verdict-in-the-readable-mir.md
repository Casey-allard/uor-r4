# #871 — docs(#821): record S0 stage PROMOTE verdict in the readable mirror

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T15:26:52Z
- **Updated:** 2026-08-20T15:51:04Z
- **Closed:** 2026-08-20T15:47:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/871
- **Merged:** 2026-08-20T15:47:53Z
- **Merge commit:** 43b7b9e9dbe00aae496b8242b35a2c0c7a562ea1
- **Base / Head:** main / UOR-Foundation:issue-821-s0-promotion
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

The S0 stage tracker **#821** (programme #820) has all five native children closed with
recorded verdicts (#829–#833), and every condition of its promotion gate is met. Its only
remaining action is the stage promotion/closure verdict. This PR records that verdict —
**S0 PROMOTE** — in the readable mirror of the tracker tree and reconciles the product
roadmap's "next up" framing so S1 (#822) reads as the entry stage. GitHub #821 stays the
source of record; it is closed separately with the full PROMOTE verdict comment and this
merge commit reference.

## Scope and non-goals

- **In scope:** doc reconciliation only — `docs/r4_intelligence_completion_plan.md` (the
  explicit readable mirror of programme #820) gains an `S0 stage verdict — PROMOTE` record;
  `ROADMAP.md` next-up prose and review date are updated.
- **Non-goals:** no code, format, conformance, `model/ids.toml`, or `CONFORMANCE.md` change;
  no new capability; no re-measurement or re-compilation; no change to any promoted numeric
  claim. This does not begin S1/S2 implementation.

## Acceptance-criteria status (tracker #821 promotion gate)

- [x] **All five children have final verdicts** — #829 (PR #860 `d1d1f735`), #830 (PR #861
  `bc5a2a92`), #831 (PR #862 `1c377df7`), #832 (PR #863 `70586aa0`), #833 (PR #867
  `1b3e46f2`; chain #864 `9c1f2c10`, #866 `19465441`), all CLOSED/COMPLETED.
- [x] **One normative scoring path named and differentially tested** — ADR-0001 designates the
  deployed R4G1 runtime scoring path; differentials + planted-negative in
  `crates/uor-r4-graph-certify/tests/normative_scorer_831.rs` (#831).
- [x] **Benchmark identities and attribution frozen** — CID-bound capability suites +
  per-token resolution attribution in `crates/uor-r4-api/src/capability_suite.rs` (#832).
- [x] **Fully attested post-#755 broad bundle passes actual production admission** —
  `smollm2-360m-broad-clean`, byte-reproducible, R4Engine admission + offline canary PASS (#833).
- [x] **M.V.G. thresholds retained/revised/retired with dated evidence** — Gate C RATIFY
  (RETAIN), 2026-08-20, headline rows within the predeclared `<~0.5 pp` bound (#833).

Kill/redesign criterion **not triggered**: claimed signals reproduce from pinned source
inputs through the deployed path.

## Tests and verification

- `python3 scripts/check_claim_wording.py` → **passed** (no unlinked machine-verified /
  exact-equivalence wording).
- Whole diff is `*.md` (neither file is `include_str!`d into Rust), so this takes the
  repository docs-only fast path; the merge queue re-verifies the merged content in full.
- No `*.rs`/manifest changed → fmt/clippy/tests/no_std/κ are unaffected by this diff.

## Evidence artifacts

`docs/r4_intelligence_completion_plan.md` (new S0 verdict record with exact evidence links),
plus the pre-existing child records it cites: `docs/adr/0001-normative-r4g1-scorer.md`,
`docs/normative_scorer_831.md`, `docs/capability_suites_832.md`,
`docs/conformance_execution_scope_830.md`, `docs/attested_broad_baseline_833.md`.

## Compatibility and migration

Documentation-only; no format, API, artifact, or on-disk migration. No downstream consumer
impact.

## Conformance effects

None. `CONFORMANCE.md` remains generated and unchanged; no RF capability added or altered;
no `model/ledger.toml` change. The S0 children already bound RF-09/15/21/22/23/29.

## Claim-boundary changes

Records the **S0 = PROMOTE** stage verdict (Definition/governance scope). No empirical claim
is created or strengthened. The teacher-floor and teacher-forced table-native rows remain
**RETAINED by invariance**, explicitly **not** re-asserted as a fresh PASS.

## Negative / unavailable results (carried forward, non-blocking)

- Full-population teacher-floor re-measurement is **UNAVAILABLE** at scale under the current
  exact-GEMM teacher (~24 h for the held-out split); RETAINED by invariance — recommended
  follow-up (faster teacher forward path or persisted floor cache), not a promotion blocker.

## Intentionally excluded

Untracked `obs-text/` and `tf1-pkg/` (pre-existing user dirs) are not staged. No stage-closure
"record doc" was invented — the repo's house style records stage verdicts on the tracker and
in this plan mirror, which is what this PR does.

References #821

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

