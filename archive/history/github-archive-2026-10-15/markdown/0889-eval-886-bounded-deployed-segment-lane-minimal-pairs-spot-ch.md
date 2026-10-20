# #889 — eval(#886): bounded deployed segment-lane minimal-pairs spot-check

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T02:55:59Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-08-21T03:15:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/889
- **Merged:** 2026-08-21T03:15:01Z
- **Merge commit:** c4e7e239f1c3838137f2a12c16cc3d289389e26a
- **Base / Head:** main / UOR-Foundation:issue-886-segment-lane-spotcheck
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

S1 follow-up A of #836 (verdict REVISE, lane dormant; parent programme #822).
#836 lowered the #835 segment lane onto the deployed `R4Engine` and closed by
reachability arithmetic (ceiling +19.0‰ upper CI < 20‰ floor) — exercising the
lane on **synthetic** fixtures only. #886 asks the cheap question: on the **real**
#833 bundle, does the deployed lowering (fit → emit → consume) realize the
whole-prompt-content signal on its own best cases — the exact minimal pairs the
#834 §6.2 reference arm resolved — within its known ceiling?

**Outcome — predeclared read: LOWERING-FIDELITY GAP.** The deployed lane follows
**1/10** of the reference arm's favorable pairs (< the pre-registered 6/10 bar).
6 of the 20 favorable-pair positions are **unservable** under the deployed policy
on the convert-emitted graph. Evidence-backed negative; the lane stays dormant.

## Scope and non-goals

- **In scope:** a bounded, deterministic spot-check harness + its recorded result.
- **Non-goals:** no full N≈24k causal run (its outcome is already fixed by the
  #836 ceiling < floor arithmetic); no change to the lane, the scorer, the
  serving policy, `model/ids.toml`, or `CONFORMANCE.md`; no fix of the gap (that
  is a maintainer decision — see Implications in the record).

## Acceptance-criteria status (issue #886)

- [x] Fit a learned table from the real corpus (`fit_segment_table`, top-64/key)
      — 19,710 learned content keys.
- [x] Emit via `convert_with_segment_table`, load into `R4Engine` (engine
      consumes every fitted row — asserted).
- [x] Replay only the §6.2 favorable minimal pairs through the deployed lane
      (reproduced 10/10 exactly; reproduction control asserted).
- [x] Report follow rate + predeclared read (`docs/segment_lane_886_result.json`).
- [x] No `model/ids.toml` change, no `CONFORMANCE.md` change — lane dormant.
- [x] Run contract posted before executing (issue comment).
- [x] One-line update to #822 (posted on merge).

## Tests and verification

- New harness `crates/uor-r4-api/tests/segment_lane_spotcheck_886.rs`
  (`--ignored`, reference-only). Ran on the real bundle:
  `served 14 / abstained 0 / unservable 6`, **follow 1/10**, `result_cid
  blake3:bf4e7022…`. Structural guards asserted: reproduction (`favorable == 10`,
  matches `docs/psi_arm_834_result.json`) and §6.2 control degeneracy
  (`baseline_follow == 0`).
- Local gates green: `cargo fmt --check`; `cargo clippy --workspace
  --all-targets --all-features -- -D warnings`; `python3
  scripts/check_claim_wording.py`; `cargo run -p xtask -- validate`
  (R1/CM-01 + R4 + R5 + gnaf). Test-only `.rs`, so not subject to R5
  audit-limits / P-4 / no_std / wasm gates (spec-leaf precedent).

## Evidence artifacts

- `docs/segment_lane_886_result.json` — CID-bound machine result.
- `docs/segment_lane_886_spotcheck.md` — method, emission-path note, result
  table, predeclared read, scope, maintainer implications.

## Compatibility and migration

None. Additive test + docs only; no runtime, format, or API change.

## Conformance effects

None. No `model/ids.toml` serving row, `CONFORMANCE.md` unchanged; the #836
activation gate is unchanged (re-clear the 20‰ causal floor on the packed
serving path). Primary IDs mapped: RF-21, RF-22, RF-27, RF-28.

## Claim-boundary changes

None asserted beyond empirical measurement. Wording is Empirical
(measured/observed), claim-wording gate green. The record explicitly notes the
emission-path caveat: the lane's only PSTATE emitter (`convert_with_segment_table`)
is not serving-equivalent on held-out data to the released cover/score emitter
(`score::emit_scored_r4g1`, which produces `graph/score.r4g1` and carries no
PSTATE) — 6/20 favorable held-out windows were unservable via the re-emit.

## Negative / unavailable results

The headline is a negative: deployed-lane follow 1/10 < 6/10 → LOWERING-FIDELITY
GAP. Robust to the unservable positions (1/6 among fully-served pairs). This is
the predeclared `< 6/10` branch and a legitimate evidence-backed completion of
the bounded check.

## Intentionally excluded

The pre-existing untracked `obs-text/` and `tf1-pkg/` user dirs are not staged.
No fix for the gap or the emission-path non-equivalence is attempted here — the
record leaves the ordering of #887/#888 to the maintainer.

Closes #886

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01N5x99jw8EtnodspLv2ke1h


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

