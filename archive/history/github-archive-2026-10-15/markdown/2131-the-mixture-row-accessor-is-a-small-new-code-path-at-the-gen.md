# #2131 — The mixture-row accessor is a small new code path at the generation site, not a visibility change; handed over unbuilt (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T02:16:21Z
- **Updated:** 2026-10-10T03:23:34Z
- **Closed:** 2026-10-10T02:17:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2131
- **Merged:** 2026-10-10T02:17:29Z
- **Merge commit:** fa9f7cba0efa3f4b3f04e424102d6620db5d541a
- **Base / Head:** main / UOR-Foundation:pcopy-accessor-handover
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What I found — it is not a one-line accessor
The mixture row is **not merely a private struct**: it is **derived from hidden states** and constructed at exactly **two sites** in `crates/uor-r4-training/src/geometric_stack.rs`.

**Site 1 — the loss path** (`pointer_loss_from_hidden`): `linear(&hidden, p.head())` → `pointer_side` → `pointer_beta` → `PointerMixture { time, dim, score, identity, select, route, ids, keys, targets, … }`.

**Site 2 — the GENERATION path** (the one that matters): `hidden_hooked(&p, ids, batch, time, &mut None)?.detach()` → the same chain → `PointerMixture`.

**Site 2 already builds the mixture at inference time from a hidden state the model computes anyway**, and `MixtureRow::evaluate` already returns every field step 1 needs (`attention`, `copy`, `gate`, `present`, `generate_share`, `copy_share`).

**So the accessor is bounded — but it is a small new code path, not a visibility change:** a public function beside Site 2 returning the row for `(ids, target)`, plus a small public struct surfacing the six fields, **placed on the generation path so it uses the same hidden state the decoder used** — otherwise it would measure a *different* mixture than the one that produced the replies, the failure mode the voided trace already taught us to avoid.

## I did not complete it, and the reason is recorded rather than dressed up
Remaining runway is not enough for a change to the model's generation path **plus** a build **plus** the run **plus** the record — and a half-built accessor measuring a different mixture than the decoder used would produce exactly the plausible-but-wrong numbers this line has spent eleven pieces learning to refuse. **The piece lands as the handover, not as the read.**

## The exact change the next attempt needs
1. A small public struct: `attention`, `copy`, `gate`, `present`, `generate_share`, `copy_share` — nothing more.
2. A public function on the generation path: `pointer_mixture_row(&self, ids: &[u32], target: u32) -> Result<Option<Row>>`, reusing the existing chain **unchanged**.
3. **A validation step:** the new path's `copy` at a step whose argmax the trace recorded must be consistent with the trace's `attention` at that source — **validated against the instrument that reproduces the sealed replies 13 of 13, not trusted on its own.**
4. Then step 1 unchanged: per-row `copy` at the value's two digits, the distractor's positions, the frame positions (`223`, `1498`, `2369`, `1156`, `2728`, `1044`, `754`, `772`, `1790`), and word rows as the control.

**Choose the exposure deliberately:** a `pub fn` on `StackModel` is the smallest thing that lets a binary run step 1, and it should be named so a future reader does not mistake it for a supported serving interface — it is an instrument read-out, and the declaration should say so.

## The bound stays published — and it is still only a bound
`p_copy(digit) ≤ attention at the argmax` on steps where the argmax is elsewhere: a ceiling beside a measurement that does not exist yet, so a reader can see whether the ceiling was tight once the numbers exist.

## The shelving rule, with BOTH reasons in sequence
1. **"the read could not be taken yet"** — `MixtureRow` is private, so the plan's own first step was not runnable as written. **That was the plan's error.**
2. **"the accessor is a small new code path, not a visibility change"** — the row comes from hidden states at two sites, and the one that matters is the generation path.

**Neither reason is "the read said nothing to train."** That reason is still **unearned**, and the plan is **not deleted** — the next person needs all three possibilities on the record.

## Ledger
Token count **REFUTED** · digit order **REFUTED** · value addressability **REFUTED** · single-source shape **REFUTED in its simple form and replaced** · "history-insensitive" **CORRECTED** · **the pointer attends the frame and never the varying slot — MEASURED on 13 rows.** This piece adds a **code-level fact**: the mixture row is reachable only through the hidden-state path.

## Delivery
Record [`docs/labs/pcopy-accessor-handover-2026-10-09/README.md`](docs/labs/pcopy-accessor-handover-2026-10-09/README.md); newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; bundle on cloud-store: `pcopy-accessor-handover-2026-10-09.tar`, 16,384 bytes, md5 `46f7404c5868ed0dc37d161932e2ea0e`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch; scratch inside the worktree.

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged; v5 not re-run. References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

