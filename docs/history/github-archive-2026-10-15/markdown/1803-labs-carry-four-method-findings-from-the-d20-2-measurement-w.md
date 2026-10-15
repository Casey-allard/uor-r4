# #1803 — labs: carry four method findings from the D20 §2 measurement work

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T22:00:00Z
- **Updated:** 2026-10-09T13:22:04Z
- **Closed:** 2026-10-06T22:01:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1803
- **Merged:** 2026-10-06T22:01:21Z
- **Merge commit:** cc2284251d47868ca08da754d546e56244e098ff
- **Base / Head:** main / UOR-Foundation:deepseek/schedule-control-note
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Adds four method findings to §10 of `docs/labs/deepseek-session-2026-10-06.md`. **Documentation only — no code, no behaviour change.**

## Why these four

Every one cost rounds this session, and each is cheap to apply and was cheap to miss.

**1. Hold the training schedule fixed when comparing two mechanisms.** Six interpretive claims moved the same direction once the schedule was controlled — the attribution of the effect, the working band's width, the lr=1e-4 behaviour, the read's "reliability", the recurrence layers' contribution, and the hybrid's bimodality. The schedule-controlled head-to-head (same `pattern=rrarra`, lr=3e-4, `min_lr=1.0` constant, three sealed seeds each) puts geometric at **0.9862** against the attention-read hybrid's **0.9149** — **+0.071 capability and 22.6× stability**, both about an order of magnitude below what the annealed comparison implied. Six for six in one direction is a bias; the remedy is procedural.

**2. An instrument's absence from `main` does not mean the experiment cannot run.** The D20 §2 parameter-matched control is not on `main` (`mlp=matched`, `parameter_match` → 0 hits), and "the 2e-4 control test is still unrun" was concluded from that — while a prebuilt binary at `~/.cache/uor-r4-d20control/release/examples/mqar-bench` could run it throughout. The same failure reported a present instrument as missing (`read_firing`; on `main` it is `best_head_mean_weight_on_key`/`_on_value`) and labelled all 23 `deepseek/*` branches unmerged, because squash-merging rewrites the commit so a merged tip is never an ancestor. **It fails in both directions.**

**3. A cross-rate comparison is a silent error of unpredictable size.** The verdict paired geometric@3e-4 with control@1e-3; at the same rate the margin moves +0.7458 → +0.7455, **0.04%**. The same mistake on the lr=1e-4 claim moved a gap −0.078 → −0.056, **40%**. Nothing distinguished them in advance.

**4. Gate a destructive step on its own check**, and the runner suite's two pre-existing flakes — both already in §10 and unchanged.

## Checks

- Markdown only; no build or test surface touched.
- Claims cite sealed roots already filed on #820; the schedule-controlled pair is at
  `~/uor-r4-local/mqar-bench/{geoconst-20261006-2125,hybconst-20261006-2055}/*-s{1,2,3}`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

