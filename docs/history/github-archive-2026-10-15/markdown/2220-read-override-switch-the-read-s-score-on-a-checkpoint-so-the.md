# #2220 — Read override: switch the read's score on a checkpoint, so the geometric read can be tested against the flat control (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T00:08:53Z
- **Updated:** 2026-10-12T00:18:45Z
- **Closed:** 2026-10-12T00:14:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2220
- **Merged:** 2026-10-12T00:14:22Z
- **Merge commit:** 0e1e18b16b55f1880b38db0f7ef54a8aa925378f
- **Base / Head:** main / UOR-Foundation:deepseek/read-override
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**`dialogue-train` builds from `init=<checkpoint>` and takes the read's score from the loaded model's configuration. The base carries `read=l2` — the flat Euclidean control of Lorentz.** So every arm this lab has trained inherited the control, and there was no way to fine-tune the geometric read in: my pre-registered `lorentz`-versus-`l2` comparison produced **six identical `l2` arms**, and the run returned nothing (reported with its cost on [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6115374102)). This is that tooling gap **fixed**, not worked around.

## What lands

| | |
|---|---|
| `StackModel::set_read_score(read)` | switches the score on a loaded model. The read's parameters are **shape-identical** between `dot`, `lorentz` and `l2`, so this is a fine-tune of the same weights under another score — exactly what testing the geometric read against the flat control requires. The configuration is revalidated, and the switch is refused once a served representation is set |
| `dialogue-train read_override=dot\|lorentz\|l2` | applied after the model loads, **recorded** in the run's settings, and the run's own configuration follows the model so the existing consistency check compares like with like and the record shows the score **actually trained** |
| without the flag | nothing changes: a checkpoint keeps its own score, bit for bit |

## Checks at this head

- `cargo build --release -p uor-r4-training --example geometric-stack` — clean.
- **Focused test** `the_read_score_can_be_switched_on_a_loaded_shape_and_the_default_is_untouched`: the checkpoint's score is untouched without the flag; the switch is explicit and revalidated; and switching back **reproduces the original loss bit for bit** on the same row. Suite: 2 passed for the filter, library tests unaffected.
- `python3 scripts/check_claim_wording.py` — **pass**.

## What it prepares

**The geometric read against its own ablation, on the record already pre-registered**: `read_override=lorentz` against `l2`, three seeds each, 8,000 steps, matched in everything else, wrong-value failures primary with the ±3-row noise floor, memory and dev NLL as secondary gates — one busy pod, deleted at the last fit, **and the first fit's `config.read` read before the rest are queued** (the discipline the failed run taught).

Line: geometric read vs flat control · count 0/3 · headline **unchanged** — the failed run is on the record as a tooling gap, and this is the fix that lets the comparison be made

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

