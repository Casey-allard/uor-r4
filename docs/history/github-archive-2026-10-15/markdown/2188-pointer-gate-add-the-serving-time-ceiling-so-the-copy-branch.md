# #2188 — pointer gate: add the serving-time ceiling, so the copy branch can be switched off at read-out (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T03:11:48Z
- **Updated:** 2026-10-11T03:52:41Z
- **Closed:** 2026-10-11T03:16:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2188
- **Merged:** 2026-10-11T03:16:06Z
- **Merge commit:** 30e83ef2f301137861631271f32b59023b8528a8
- **Base / Head:** main / UOR-Foundation:deepseek/answer-path-attribution
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The forced-copy oracle measured the copy branch at **~6 % of the output mixture** (`dev_pointer_mean_gate` 0.056–0.064 across the order-3 address arms) and showed that forcing it to 0.5 or 1.0 costs 14–16 of 40 memory rows. The mirror experiment — **switch the copy branch off entirely** and see whether the answers change — needs a ceiling, because `set_pointer_gate_floor(0.0)` is the identity.**

Line: pointer identity key / answer-path attribution (D22 order 3) · count 0/3 · headline unchanged (preparation)

## What changed

| | |
|---|---|
| `StackModel::set_pointer_gate_ceiling` | the floor's mirror on the read-out path: both serving mixtures use `min(max(gate, floor), ceiling)` in place of the head's own gate, so `0.0` switches the copy branch off and `1.0`, the default, leaves every reply bit for bit as it was. No parameter, no gradient, no training path, not saved; refused outside `0..=1` and refused below a floor the caller has set |
| `chat-grade reply` | takes **`pointer_gate_ceiling=F`** and records it in `replies.json` beside the floor |
| other callers | `grade_into` passes `1.0`, so graded replies are unchanged |

## Checks at this head

- `cargo build --release -p uor-r4-training --lib` and `--bin chat-grade` — clean.
- The default is the identity by construction (`1.0`), and the `0.0` sweep is running on the real artifact as this PR is opened.
- `python3 scripts/check_claim_wording.py` — **pass**.

## What it prepares

**The answer-path attribution** on the frozen v5 panel, one artifact, no training: **(B)** copy branch off — if the panel is unchanged, the copy is confirmed irrelevant to the answers; **(C)** the fact turns removed from the prompt — if the answers collapse, they come from the context through the **read**, and if they do not, the model is answering from pretraining and the grounding story is wrong at the root. Both readings are needed before any mechanism is built on top of either path.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

