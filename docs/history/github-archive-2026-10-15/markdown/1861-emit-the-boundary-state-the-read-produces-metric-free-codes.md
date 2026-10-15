# #1861 — Emit the boundary state: the read produces metric-free codes, so no readout on it transfers (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T23:25:10Z
- **Updated:** 2026-10-07T23:26:51Z
- **Closed:** 2026-10-07T23:26:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1861
- **Merged:** 2026-10-07T23:26:51Z
- **Merge commit:** 63dab2d67a34f1b6e8d634e1643f1717e708f8c6
- **Base / Head:** main / UOR-Foundation:deepseek/entry-scorer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this adds

The entry ceiling now records each row's **boundary state** (`retained_state_codes`, the 8 retained H4 codes the emission actually reads). One line of JSON, and it is what let the representation question be answered offline without a model.

## Why, and what it settled

The adversarial review of the entry work left one fork: is the entry failure a **capacity/training** problem in the emission, or a **representation** problem in the read state? A state-only probe settles it, and it needs the state, which was never recorded.

Offline, over the emitted rows (`--slurpfile` over the sealed roots, no model, no fit):

| statistic | retained panel (512 rows) | transfer panel (32 rows) |
|---|---:|---:|
| mean distance, same question form | **7.931** | **7.938** |
| mean distance, different question forms | **7.931** | **7.941** |
| distinct states | 512 / 512 | 32 / 32 |
| state length | 8 codes | 8 codes |

Same-question and different-question distances are identical to four decimals over 130,816 retained pairs and all 496 transfer pairs. On the retained panel, consecutive rows are the same facts asked two different ways: **0 of 256 pairs have identical states and the median distance between them is 8 of 8 codes**. Fitting a 1-NN on the retained states and predicting the transfer rows gives **17/32 (53 %)** against a 0.5 majority.

**The read state is metric-free.** Nearly every pair of rows is almost maximally far apart, so the space has no neighbourhood structure and "asked the same thing" is not nearby in any metric a readout can use. The query tokens do determine the answer (the query signature predicts it with zero ambiguity, 256/256 held out), so the information is in the input; it does not survive the read into the state in transferable form.

That closes the fork on the representation side, and it is consistent with everything else measured: the emission is flat (rank ~1,140, top-1 share 0.070 % against uniform 0.024 %), entry is 0 on both panels, and credit, Copy, calibration, the decision head and the export are each excluded by a sealed run.

**Limit of the probe, stated plainly.** It measures structure under the state's own code geometry — Hamming over the 8 observed codes. It rules out first-order neighbourhood structure, which is what simple readouts use. It does not prove no learned metric could extract anything; it proves nothing in the current pipeline learns one, and that the read produces codes without same-question clustering.

**Acceptance test it hands the next step:** rebuild the boundary state so same-question rows cluster and different-question rows separate **on the transfer panel**, by that same within/across statistic, before any emission or entry work resumes.

## Verification

`cargo check -p uor-r4-training --example geometric-frozen-map-fit` clean, `cargo test -p uor-r4-training --example geometric-frozen-map-fit` **13 passed / 0 failed**, touched-file `rustfmt --check` clean. The added field is diagnostic output only; no decision path reads it.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

