# #2192 — Head ablation: silencing the supervised read layer removes the answers, not just their accuracy (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T06:27:48Z
- **Updated:** 2026-10-11T06:36:10Z
- **Closed:** 2026-10-11T06:30:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2192
- **Merged:** 2026-10-11T06:30:46Z
- **Merge commit:** 8d6b94f63fc94b454ba14210e3f9943fe8641a77
- **Base / Head:** main / UOR-Foundation:deepseek/head-ablation-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The read-binding brief's last unmet requirement — *"a head-ablation check that the bound head drives the answer"* — answered with a new serving-time instrument: silence one read layer and watch the answers.** On the read-bound artifact **S2** (24/40 unablated): **silencing the supervised (last) read layer → 0/40 with zero wrong-value failures** (37 rows produce no value at all), the first read → **2/40**, the middle read → **7/40**. The layer the supervision targets is the layer the answers come from. References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: read binding / head ablation (D22 order 4) · count 0/3 · headline before memory 24/40 → after **0/40** (supervised read ablation)

## The instrument

`StackModel::set_read_silence(Some(layer))` makes that layer's read contribute nothing to the residual (`x = x + 0`) with the rest of the forward unchanged; `chat-grade reply read_silence_layer=L` sets it and records it in `replies.json`. Read-out only: no parameter, no gradient, no training path, not saved, and a non-read layer is refused ([#2191](https://github.com/UOR-Foundation/uor-r4/pull/2191)).

## Results — one artifact, one read layer at a time, frozen v5 panel

| condition | memory `check_pass` | wrong-value | distractor-key | **no value** | mean reply words |
|---|---:|---:|---:|---:|---:|
| **unablated** | **24/40** | 5 | 4 | 7 | 4.8 |
| silence the **first** read (layer 2) | 2/40 | 9 | 0 | 29 | 18.2 |
| silence the **middle** read (layer 5) | 7/40 | 12 | 0 | 21 | 5.5 |
| **silence the supervised read (layer 8)** | **0/40** | **0** | 3 | **37** | 6.3 |

**Three things this establishes.** (1) **The bound head drives the answer:** the supervised layer's removal leaves no correct answer in any row and — unlike the other two ablations — **zero wrong values**; the model stops producing values rather than producing the wrong ones. (2) **The read layers are not interchangeable** (0/40, 2/40, 7/40), which gives the brief's open question *"supervise every head, or route the output through the binding head?"* a measured answer about where that routing matters most. (3) **Three measurements now tell one story:** the [attribution](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6105227220) says the answers come from the read; the [powered read binding](https://github.com/UOR-Foundation/uor-r4/pull/2190) says the read can be steered and the headline moves; this says the head being steered is the one that carries the answers.

## Also completed: the powered cycle's pending BPB pair

**Ctl2 reads 1.17003 against S2's 1.17067** — a difference of **0.0006**, inside the bar's 0.01, with both below the anchor's 1.17425. Ctl3's evaluation is the one remaining number and is named pending.

## Checks

- [x] `python3 scripts/check_claim_wording.py` **pass**; docs-only (one record, one `current-state` entry, one `STATUS.md` row). The instrument is the separate prep PR #2191.
- [x] Laptop CPU only; no pod held for the measurement (the lease used to re-fetch the artifact was released after the pull).

**Declared limitations:** one artifact, one seed, 40 rows; silencing a **whole layer** is coarser than silencing the supervised head (which runs per head inside that layer); no judge-side numbers (not needed for a causal check); the unablated column is the previous cycle's scoring rather than a re-run.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

