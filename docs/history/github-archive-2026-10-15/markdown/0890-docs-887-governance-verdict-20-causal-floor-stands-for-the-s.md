# #890 — docs(#887): governance verdict — 20‰ causal floor stands for the segment re-ranking arm class

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T03:37:13Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-08-21T03:58:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/890
- **Merged:** 2026-08-21T03:58:21Z
- **Merge commit:** 7bb8b4ecbd441ee7018185a0d2e73a1216a6927f
- **Base / Head:** main / UOR-Foundation:issue-887-floor-governance
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

#887 asks, as a **governance / pre-registration** question, whether the frozen
20‰ `CAUSAL_FLOOR_PERMILLE` is the correct promotion bar for the bounded
**segment re-ranking** lane (#836, verdict REVISE), now that the #886 bounded
spot-check is in hand.

**Outcome (the pre-declared default): the bar stands; the lane stays dormant.**
This PR commits the governance decision record
`docs/segment_lane_887_governance.md`, which concludes the 20‰ floor **STANDS**
for the bounded state/residual (segment re-ranking) arm class and **retires** the
segment lane from the promotion track (kept dormant / ledgered as an
evidence-backed negative).

## Scope and non-goals

**In scope:** one append-only governance record under `docs/`; the #822 tracker
update (posted as an issue comment); the closing-off of #888.

**Non-goals:** no floor change; no new bar minted; no code; no `model/ids.toml`;
no `CONFORMANCE.md` edit; no serving-path change; no S1 stage verdict (that is
#822's, against the full child set, with #834 still open); no #834 action.

## Acceptance-criteria status (#887 DoD)

- [x] Committed decision record stating whether 20‰ stands for this arm class → **stands** (§5).
- [x] The non-default "new pre-registered criterion" path is addressed and **declined** with rationale (§4).
- [x] #822 update recording the governance verdict → posted as a tracker comment.
- [x] The candidate-support widening follow-up (#888) is **closed off** (§6).

## Verification

- `python3 scripts/check_claim_wording.py` → **passed**.
- `cargo fmt --check` → **clean** (confirms no Rust touched).
- Diff is **docs-only** (`docs/segment_lane_887_governance.md`, 145 insertions),
  so the PR takes the repo's docs-only fast path (claim-wording only on the PR
  side); the merge queue re-verifies the full ladder on the merged content. No
  code compiles differently, so the cargo test/clippy/no_std/κ ladder is
  unaffected by this change.

## Evidence artifacts (all pre-existing, CID-bound, on main)

- Pre-registration: #822 promotion gate; `CAUSAL_FLOOR_PERMILLE = 20.0‰`.
- Reachability: `docs/segment_lane_836_verdict.md` + `segment_lane_836_result.json`
  (reference ceiling +17.5‰, CI [15.9, 19.0]; upper 19.0 < 20.0, unconditional;
  deployed −1.6‰, CI [−2.4, −0.8]).
- Deployed fidelity: `docs/segment_lane_886_spotcheck.md` +
  `segment_lane_886_result.json` (follow 1/10 < 6/10 faithful bar; robust 1/6;
  emission-path non-equivalence).
- Reference arm: `docs/psi_arm_834_result.json` (suffix 246.6‰ → Ψ 264.1‰; Δ
  +17.5‰; minimal pairs 10/4722).

## Compatibility and migration

None. Docs-only; no format, artifact, or API change. Absent-section identity and
all serving bytes unchanged (the lane was and stays dormant).

## Conformance effects

None. No `model/ids.toml` row, no `CONFORMANCE.md` regen. Governance/documentation
execution scope. Primary IDs: RF-01, RF-27, RF-28.

## Claim-boundary changes

Records a **negative / retire** verdict for the segment re-ranking arm: it fails
the 20‰ causal floor and is retired from the promotion track (dormant). No new
capability claim; no existing claim strengthened.

## Negative / unavailable results

This PR **is** the evidence-backed negative branch (the segment lane does not
clear the bar). No hours-long run launched — the #836 reachability arithmetic
(ceiling < floor, unconditional) already fixes the outcome.

## Intentionally excluded

- #834 (research B) — remains open and maintainer-parked; not touched.
- #888 — closed off by this verdict as a downstream consequence; no code change here.
- Pre-existing untracked `obs-text/` and `tf1-pkg/` — never staged.

Closes #887

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

