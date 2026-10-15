# #886 — eval(S1): bounded minimal-pairs spot-check of the deployed segment lane (follow-up A of #836)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-21T01:33:20Z
- **Updated:** 2026-08-21T03:16:45Z
- **Closed:** 2026-08-21T03:15:02Z
- **Labels:** enhancement, conformance
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/886
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent programme: #822 · Follows: #836 (4c verdict: REVISE)

## Objective

Empirically confirm that the **deployed** learned-table segment lane (the P-4,
integer-`ScoreQ`, top-8 serving path lowered in #836 4c-i/4c-ii) actually
realizes the whole-prompt-content signal the #834 §6.2 reference arm measured
off the serving path — within its known ceiling — using a **bounded** check, not
the full hours-long causal run.

## Why this is first

#836's 4c-iii verdict is REVISE by reachability arithmetic: the reference-arm
ceiling (+17.5‰, CI [15.9, 19.0]) is below the 20‰ causal floor, and the
deployed lane is strictly weaker (quantization + top-8 bound). Before investing
in any mechanism change, we need cheap evidence that the deployed lowering
tracks the reference within that ceiling — i.e. that the fit→emit→consume chain
is faithful on the *real* bundle, not only on the synthetic fixtures the #836
tests use.

## Scope (bounded, seconds not hours)

- Fit a learned table from the real #833/#834 corpus with
  `uor_r4_graph_compiler::segment_fit::fit_segment_table` (top-64/key, as §6.2),
  emit a bundle via `convert_with_segment_table`, and load it into `R4Engine`.
- Replay **only** the 10/4,722 minimal pairs the §6.2 reference arm resolved (or
  a similarly small, pre-registered favorable subset) through the deployed lane.
- Report: of those known-favorable pairs, how many the deployed quantized
  top-8 lane follows. Predeclare the read: `< 6/10` ⇒ the deployed path does not
  even track its own ceiling on its best cases (a lowering-fidelity problem to
  fix before anything else); `>= 6/10` ⇒ the lowering is faithful and the lane
  sits at its arithmetic ceiling as expected.

## Out of scope / discipline

- **No** full N≈24k causal run. This issue exists precisely to avoid it; the
  reachability arithmetic already fixes the full-run outcome.
- Post the repository run contract for the bounded check before executing.

## Execution scope & conformance

Compiler-side fit + a bounded serving replay. Preserve P-4/determinism/allocation
/no_std/error/witness contracts on the serving path (the lane is unchanged; this
only measures it). No `model/ids.toml` change, no `CONFORMANCE.md` change — the
lane stays dormant regardless of outcome. Primary IDs: RF-21, RF-22, RF-27,
RF-28.

## Definition of done

A committed bounded-check result doc (mirroring `docs/segment_lane_836_result.json`)
with the follow rate and its predeclared read, and a one-line update to #822 on
whether the deployed lowering is faithful.


## Comments (2)

### comment 5364504207 — 2026-08-21T02:36:18Z — Casey-allard

### Run contract — bounded deployed-lane minimal-pairs spot-check (#886)

Posted before executing, per the long-run discipline (`AGENTS.md`) and this issue's
"Post the repository run contract for the bounded check before executing".

```
metric to move:       deployed-lane follow count on the §6.2 favorable minimal pairs
                      (how many of the reference arm's followed pairs the DEPLOYED
                      quantized top-8 lane also follows). Current value: UNMEASURED on
                      the real bundle — #836 exercised the lane on synthetic fixtures +
                      reachability arithmetic only; the lane has never run on the real
                      #833 bundle.
reachability ceiling: the §6.2 reference arm followed 10 / 4,722 minimal pairs
                      (psi_follow = 10; docs/psi_arm_834_result.json). The deployed lane
                      is STRICTLY WEAKER — integer-ScoreQ quantization, the top-8
                      candidate bound, and the 8-slot content ring — so its follow count
                      is bounded above by 10. This is a lowering-FIDELITY check within
                      that ceiling, not a floor-clearing run: the whole-prompt effect
                      ceiling (+19.0 permille upper CI) is already below the 20 permille
                      floor unconditionally (#836), so NO full causal run is launched.
instrument + verdict: crates/uor-r4-api/tests/segment_lane_spotcheck_886.rs (--ignored).
                      Two guards must pass or the measurement is void:
                      (a) reproduction: the mined favorable set == 10 (matches §6.2);
                      (b) base fidelity: the re-emitted bundle's base decision equals the
                          deployed graph/score.r4g1 on every evaluated position.
exit rule:            deployed lane follows < 6/10 => LOWERING-FIDELITY GAP;
                      >= 6/10 => FAITHFUL (lowering tracks the §6.2 ceiling).
if faithful (>=6/10): record docs/segment_lane_886_result.json; the fit->emit->consume
                      lowering is confirmed faithful; the pivotal question moves to #887
                      (is 20 permille the right bar for a re-ranking lane). Lane dormant.
if gap (<6/10):       record docs/segment_lane_886_result.json; the lowering loses signal
                      on its own best cases — a fidelity bug in the quantize/top-8/ring
                      path to fix BEFORE #887/#888. (Distinct next action.)
cost estimate:        seconds-to-minutes, one machine, offline, no teacher forward
                      (one table fit + one convert over the real store + ~20 deployed-lane
                      predictions). Blocks nothing; no ids.toml / CONFORMANCE change.
```


### comment 5364766338 — 2026-08-21T03:16:45Z — Casey-allard

## #886 complete — deployed lowering **NOT faithful** on the real bundle (read: LOWERING-FIDELITY GAP)

Merged: PR #889, merge commit `c4e7e239f1c3838137f2a12c16cc3d289389e26a` (single clean queue pass; full ladder green). Artifacts on `main`:
- `crates/uor-r4-api/tests/segment_lane_spotcheck_886.rs` (`--ignored` harness)
- `docs/segment_lane_886_result.json` (CID-bound result, `result_cid blake3:bf4e7022…`)
- `docs/segment_lane_886_spotcheck.md` (method + record)

### Outcome against the run contract

| run-contract field | outcome |
|---|---|
| metric (deployed-lane follow on §6.2 favorable pairs) | **1 / 10** |
| instrument guard (reproduce favorable == 10) | **PASS** (10/10; §6.2 `baseline_follow == 0` also asserted) |
| exit rule (`< 6/10` ⇒ gap; `>= 6/10` ⇒ faithful) | **`< 6/10` → LOWERING-FIDELITY GAP** |
| servability | served 14 / abstained 0 / **unservable 6** of 20 positions |
| `if gap` next action (pre-declared) | a lowering-fidelity issue to weigh **before** #887/#888 (left to the maintainer) |

Robust to the 6 unservable positions: among the 6 fully-served pairs, 1 followed (1/6). The fitted table (19,710 learned content keys) reached the engine intact (`segment_learned_rows` asserted), so the gap is in the quantized top-8 / 8-slot-ring re-rank, not in the fit or emit.

### Emission-path finding (measured)

The lane's only PSTATE emitter is `convert_with_segment_table`; the **released** serving graph `graph/score.r4g1` comes from a different emitter (`score::emit_scored_r4g1`) and carries no PSTATE. The two are not serving-equivalent on held-out data: under the deployed policy the re-emitted graph left **6/20** favorable held-out positions **unservable** (a widen re-probe the released cover graph supports is unsatisfiable on the re-emit), whereas `causal_prompt_run_834` served 20k held-out windows on the released graph.

### Scope

Reference-only / off the serving default; measurement, not deployment. Lane stays **dormant** — no `model/ids.toml` serving row, `CONFORMANCE.md` unchanged, #836 activation gate unchanged. RF-21/22/27/28. No hours-long run launched. Evidence-backed negative recorded as a legitimate completion.



---
_Comment/review ids in this file: 5364504207, 5364766338_

