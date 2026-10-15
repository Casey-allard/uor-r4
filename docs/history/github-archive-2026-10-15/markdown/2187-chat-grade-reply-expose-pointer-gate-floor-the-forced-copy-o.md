# #2187 — chat-grade reply: expose pointer_gate_floor=, the forced-copy oracle of the pointer mechanism brief (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T01:29:38Z
- **Updated:** 2026-10-11T13:04:10Z
- **Closed:** 2026-10-11T01:33:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2187
- **Merged:** 2026-10-11T01:33:56Z
- **Merge commit:** f97ccc3adc12fc16185f16244c9ae96543b18a28
- **Base / Head:** main / UOR-Foundation:deepseek/copy-oracle-20261011
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The pointer [mechanism brief](docs/mechanisms/pointer-identity-key.md)'s first fair-test step is a serving-time oracle ablation: *"supply the true slot address. It bounds the gain at no training cost."* The cheapest faithful half of that is the **gate** oracle — force the copy branch on and see what the model does with it. `StackModel::set_pointer_gate_floor` has existed as a read-out knob all along (both serving mixtures use `max(gate, floor)`; no parameter, no gradient, no training path), but **no CLI could set it**, so the oracle was not runnable. That is a tooling gap this PR closes.**

Line: pointer identity key / forced-copy oracle (D22 order 3) · count 0/3 · headline unchanged (preparation)

## What changed

| | |
|---|---|
| `chat-grade reply` | takes **`pointer_gate_floor=F`** (a probability in `[0, 1]`, default `0.0`), applies it after the model loads and **before any reply is generated**, and records it in `replies.json` so a reading is bound to the setting that produced it |
| the default is the identity | the `0.0` control run reproduces the artifact's published reading **exactly** — v5 memory **23/40**, wrong-value **9** — which is the check that the flag changes nothing at its default |
| other callers | `grade_into` passes `0.0`, so graded replies are unchanged |

## Checks at this head

- `cargo build --release -p uor-r4-training --bin chat-grade` — clean.
- The flag's behaviour is exercised on the real artifact: `pointer_gate_floor=0.0` reproduces the S2 arm's own reading bit for bit (23/40, wrong-value 9); `pointer_gate_floor=1.0` builds and runs.
- `python3 scripts/check_claim_wording.py` — **pass**.

## What it prepares

The **forced-copy oracle** on the frozen v5 panel at floors **0.0 / 0.5 / 1.0**, scored with the frozen checks: does the copy branch produce the value when it is allowed to participate? The order-3 measurement is why this is the right question — an exact one-atom address raised the pointer's hit rate **0.377 → 0.54** at **zero likelihood cost** while the panel stayed flat (19/21/23 of 40), so either the copy is gated shut or the decoder cannot place what the pointer found.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

