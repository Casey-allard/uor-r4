# ORDER: m1-export-gap-and-toks, antigravity-74

Lab: antigravity. Item: [m1-export-gap-and-toks](../../../../roadmap/items/m1-export-gap-and-toks.md) (#2839). Base: `03930e2485a5a8dc2d51e9ff7afb088e478c6850`.

## Goal

The item's done-when reads:

> Stage 4 bar: 'the integer audits stay green, the gap stays within the noise band, and tokens per second is reported for both reading the prompt and writing the reply.' Decode and context rates are reported separately.

Complete Step 3 of `docs/roadmap/items/m1-export-gap-and-toks.md` (`How to start`: non-author `TEST FITNESS: FIT` review of `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md` under `CHARTER.md` §4.5 and D67/D68), update `docs/labs/m1-export-gap-and-toks/README.md`, and complete a D68 decision review (`CHARTER.md` §5.1) registering the Stage 4 bar/noise band and setting `status: held` on `docs/roadmap/items/m1-export-gap-and-toks.md` (`status_only_edit` per `crates/uor-r4-harness/src/loopprcheck.rs:488`) until `m1-bvg-k-stage-t-training` (`#2836`, `PLAN.md` Step 4) produces the trained checkpoint input.

## Cancel probe (run first)

1. **Umbrella-lock refusal & zero-overlap check (`run-antigravity.txt` Part 3 Step 2):**
   - `uor-harness loop claim m1-export-gap-and-toks --lab-topic m1-export-gap-fit` exited `2`:
     `uor-harness: loop claim: refused: scope docs/mechanisms/lut-serving.md overlaps docs/mechanisms/lut-serving.md of item infra-loop-harness (PR #3169 (docs/loop/tasks/infra-loop-harness/codex-1/ORDER.md))`
   - Checked `gh pr view 3169 --json files,isDraft,updatedAt`: PR `#3169` (`codex-1`, idle draft since `2026-10-25T01:07:47Z`) modifies only `PLAN.md`, `docs/integration/DECISIONS.md`, and `docs/loop/tasks/infra-loop-harness/codex-1/{ORDER,PLAN,REPORT}.md` (`0` overlapping files with `m1-export-gap-and-toks`).
2. **Readiness & prerequisite probe (`instrument` — dependency on Step 4 `m1-bvg-k-stage-t-training`):**
   - `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md` is still at `TEST FITNESS: PENDING` (line 111) and references pre-D67/D68 holds;
   - `PLAN.md` Step 6 (`m1-export-gap-and-toks`, `#2839`) specifies `Needs: the exporter (done, #2838); its measured run needs a trained model from step 4` (`m1-bvg-k-stage-t-training`, `#2836`), while `m1-a8-full-dose-rerun` (`#2831`) ended in an instrument `STOP_FOR_DIAGNOSIS` (`consecutive_rejections`) at batch `1,600` (`Relation`) / `5,248` (`FrozenZero`), and `#2836` has not yet run because `crates/uor-r4-discrete/**` is currently locked by `claude-103` (PR `#3203`).

## Files in scope

- `docs/harness/prep/m1-export-gap-and-toks*`
- `docs/labs/m1-export-gap-and-toks/**`
- `docs/loop/tasks/m1-export-gap-and-toks/**`
- `docs/roadmap/items/m1-export-gap-and-toks.md`
- `docs/roadmap/README.md` (shared registration file)

## Done when

- `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md` passes a fresh-context non-author `TEST FITNESS: FIT` review (`CHARTER.md` §4.5) and is updated to `TEST FITNESS: FIT`;
- A fresh-context D68 decision review (`CHARTER.md` §5.1) approves registering the Stage 4 bar (`B1..B4`) and two-seed export-gap noise band and setting `status: held` on `docs/roadmap/items/m1-export-gap-and-toks.md` until `m1-bvg-k-stage-t-training` (`#2836`) produces the trained checkpoint, logged on `#2839`;
- `uor-harness loop pr-check --local` and `uor-harness loop gates --full` pass at the head (`References #2839`).

## Stop and ask if

- Any file outside the declared scope is needed;
- Any check in `uor-harness loop gates --full` fails.
