# #2075 — v4 memory panel: 31/40 reproduced from the artifact, nine misses in two mechanisms, no bounded lever (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T19:15:43Z
- **Updated:** 2026-10-09T19:20:23Z
- **Closed:** 2026-10-09T19:18:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2075
- **Merged:** 2026-10-09T19:18:09Z
- **Merge commit:** f7ef028b051134be000bcd5104d1c27a3eb6b309
- **Base / Head:** main / UOR-Foundation:deepseek/v4-memory-panel-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes nothing; **References #2029**. Offline capability record, **not served**. Zero pod spend.

## What this is

The v4 memory-panel half of M1 acceptance criterion 1 (`≥ 34/40`). The round reproduced the recorded
**31/40** exactly from the artifact, classified the nine failures per category with the mechanism named,
and closes **NEGATIVE-ORIENTED**: no single mechanism dominates the nine, so no candidate was trained and
Phase 2 was not opened.

## Two identities in the round's brief were wrong, and both mattered

1. **`31/40` is chat-grade's `check_pass`, not its `acceptable`.** The frozen `exact` check alone gives 31;
   the tool-level `acceptable` (fluent ∧ relevant ∧ check, `qwen2.5:7b` judge) is **27/40** for that
   category. The ladder column is `check_pass` throughout, so criterion 1's `≥ 34/40` is a **three**-cell
   gap read that way and a **seven**-cell gap read as `acceptable`. The README cells that mixed the two
   metrics are corrected.
2. **The 31/40 artifact is not the Plan A base.** It is Step 8 arm Q: pretrain `e448de86…` plus the Step 7d
   fine-tune (2,000 steps, `pointer=32`, `context=384`, `policy=full_prefix`, no read supervision), model
   sha256 `9c0d9019…`, argv taken from the **recorded** `run-step8.sh` inside
   `results/claude/step8-capacity-20261007.tar`. `bases/planA-214m-2b425ccd` and
   `models/chat-214m-planA-af-b8` have **no recorded v4 memory number at all**.

## Measured

* Panel identity: `data/panels/conversational-v4*`, 64 rows (`multi_turn_memory` 40, `exact`;
  `unknowable_or_impossible` 24), all 20 files verifying against `MANIFEST.sha256`.
* Re-grading the sealed `replies.json` (`64721530…`) with the same grader digest (`845dbda0…`): the sealed
  report's numbers line for line — memory `check_pass` 31 / `acceptable` 27, unknowable `check_pass` 10,
  panel 31 `acceptable` / 41 `check_pass`. 60 of 64 rows field-identical; no headline number moves.
* Running the model end to end on the laptop CPU: **31/40 with the same nine failures, all 64 rows
  byte-identical to the sealed CUDA replies in reply strings and generated ids** (64/64 rows, 150 reply
  calls). This panel number needs no pod to reproduce.
* The nine, by the frozen check's own failure modes: **3 wrong value** (`mem-11` turquoise for pelican,
  `mem-20` jigsaw for raincoat, `mem-29` Juniper for 47 — the forbidden distractor named in the asked
  slot) and **6 no value** (`mem-23`, `mem-24`, `mem-28`, `mem-30`, `mem-35`, `mem-37`). **0** form/prefix,
  **0** wrong-key binding, **0** abstention. The pre-registered binding probe had already split these same
  nine at Step 11 as **5 swap / 4 readout / 0 unreached**.

## Why there is no Phase 2

The nine split 5/4 by the probe and 3/6 by the check; neither reading yields one mechanism covering the
three cells needed. And the majority class's only on-point lever was already measured **on this exact base
and panel**: arm P (same pretrain, same fine-tune, plus `read_binding_supervision=0.1` at layer 14) scored
**29/40** against arm Q's 31/40 and was rejected. A config delta against swaps is a repeat of a measured
negative, not a new experiment. A negative is the result.

## D11/export

Not assumed away, and not claimed as measured. The artifact is a float pointer model, and the training
tool's own recorded scope states that a pointer's selection has no D11 port, that `export` writes a
pointer keeping every source, and that `qat=true` refuses a pointer — so no QAT/4-bit artifact can be built
for it, which is the path the served BPB target is measured on. `export` was not run here.

## Stale docs fixed

`README.md` (three cells that attributed the fine-tune's v4 memory number to the base checkpoint and mixed
`check_pass` with `acceptable`), `data/panels/README.md` (v4's held-out status: models have replied and
the misses are inspected, so v4 is a development panel from 2026-10-09). STATUS/ROADMAP and the #2028
table are **unchanged** — no served model, BPB headline or milestone moved.

## Checks

* `cargo test -p uor-r4-training --bin chat-grade panel` → **3 passed** (`panel_v4_rows_checks_and_check_only_controls`, `panel_v3_rows_checks_and_check_only_controls`, `panels_v3_v4_share_no_m_world_phrasing`).
* `python3 scripts/check_claim_wording.py` → gate passed.
* `python3 scripts/check_panel_v4_novelty.py` → `"result": "pass"`, `"collisions": []`.
* `shasum -a 256 -c data/panels/MANIFEST.sha256` → 20 × OK.
* The shipped `classify_v4_memory.py` reproduces 31/40 pass / 3 wrong_value / 6 no_value on both the sealed
  and the regenerated replies.

Cost: **zero pod spend**; laptop 4 min 53 s build, 11 min 1 s re-grade, 17 min 30 s CPU regeneration.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

