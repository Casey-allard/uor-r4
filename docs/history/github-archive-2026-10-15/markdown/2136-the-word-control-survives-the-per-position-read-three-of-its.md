# #2136 — The word control survives the per-position read; three of its rows were inflated up to 14x (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T03:17:47Z
- **Updated:** 2026-10-10T03:23:20Z
- **Closed:** 2026-10-10T03:20:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2136
- **Merged:** 2026-10-10T03:20:30Z
- **Merge commit:** 33f12db4e9bbcb03dbd024b4063f333ccdd4d500
- **Base / Head:** main / UOR-Foundation:deepseek/pcopy-posread-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The word control re-read **per position**, the way the previous record's own Next named. **References #2029.** CPU only, no pod, no training, no knob, no model saved, no v5 re-run, $0.

## The smallest step

`copy_mass` is a mass over token **ids**, so a word whose subwords recur elsewhere has its share inflated. The pointer's own attention vector - the public `read_span_probe` -> `SpanProbe.pointer.attention` - gives the same distribution over **positions**, so the value can be read at the positions that hold it. Both are now reported side by side in the same example.

## The gate, extended and enforced in every run

1. `hit` equals the trace's `matches_source` - **166 of 166** traced steps (unchanged).
2. the per-position read's attention argmax equals the trace's `source` position and `source_id` - **166 of 166** (so the attention vector is the decoder's own).
3. the two public paths agree: `score_targets` copy_mass(target) equals `read_span_probe` attention summed over the positions holding that target - measured max difference **2.329e-6** against a deliberate **1e-5** tolerance (f32 accumulation; the first run at 1e-6 tripped on exactly that value, so the tolerance was set explicitly and the measured value is printed every run).

Any failure writes VOID and reports no number.

## The answer: the control survives, the magnitudes move

- **21 of 27 word rows still put the value above the frame** on the per-position read - exactly the id-based count - **against 0 of 13 numeric rows**, and **no row's value-versus-frame verdict flips** between metrics. The falsification clause ("word rows move as much as numeric rows") still does not fire.
- **"16 of 27 word rows reach 0.40" becomes 13 of 27** at the value's own positions. Five word rows were inflated: Howard **14.05x** (0.0371 pos vs 0.5219 id), Mitchell 3.96x, Iowa 1.26x, Lloyd 1.25x, Wichita 1.07x.
- Numeric: four rows were inflated (mem-021 **9.45x**, mem-022 1.90x, mem-005 1.70x, mem-029 1.24x) because a digit is shared with the distractor; the numeric value shares move **down**, so the deficit is slightly worse than published. 3 of 13 rows still reach 0.40 under both metrics.
- Cross-checks: the single-token subset is one row (Polly, pos == id 0.2330 vs frame 0.0370, value wins 7/9); the stronger check is that **22 of 27 word rows have equal id and position shares by construction**.
- **Carry forward the per-position share**; the id share stays published beside it, labelled, with its inflation factor.

## Delivery

- record `docs/labs/pcopy-posread-2026-10-09/README.md`; evidence `docs/evidence/pcopy_posread_2026-10-09.txt` (both per-row tables, the extended gate, verbatim output)
- newest-first `current-state.md` entry ending in **Next:**
- **dated correction on the `pcopy-mass-read` record** whose word-control paragraph this piece corrects
- run outputs + scripts at `icloud:UOR-R4/results/deepseek/pcopy-posread-2026-10-09.tar` (7,043,584 B, md5 `672cd5bd...`, fetched back and verified, `all.json` `7acf52da...` both sides)
- `check_claim_wording.py` passes; `rustfmt --check` clean; the example is **run**, not asserted: numeric pass 16 s, all-rows pass 115 s
- **STATUS/ROADMAP/#2028 unchanged**: no capability, served model or milestone moved; criterion 1 remains NOT MET, 43/232 unchanged

**Limitations:** one artifact/arm/panel; the single-token subset is one row; `value_pos_share` sums over every in-window occurrence of the value's text rather than one chosen occurrence; the frame baseline stays id-based (an asymmetry that favours the frame, so it cannot manufacture the numeric deficit); the 1e-5 tolerance is a judgement with the measured value printed alongside.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

