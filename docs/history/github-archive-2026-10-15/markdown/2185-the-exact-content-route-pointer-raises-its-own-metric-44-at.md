# #2185 — The exact content-route pointer raises its own metric 44% at no likelihood cost, and the answers do not follow (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T00:54:50Z
- **Updated:** 2026-10-11T13:04:11Z
- **Closed:** 2026-10-11T00:58:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2185
- **Merged:** 2026-10-11T00:58:19Z
- **Merge commit:** 813e547e07d37458bbd8cd49fefed973c8ed270d
- **Base / Head:** main / UOR-Foundation:deepseek/ptr-addr-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**D22 order 3, first configuration: the identity pointer keyed on the **slot/entity address** — the trainer's own route on a **one-atom window** (`pointer_route=prime-ranked:1`, ranked by `ln gcd`) with the six-token surrounding-window key **off**. Result: the pointer's own metric moves decisively (**hit rate 0.377 → 0.54**, stably, on three genuine seeds) **at no likelihood cost** (BPB at or below the anchor on every seed), and the answers do not follow (memory 19/21/23 of 40, wrong-value 17/15/9 against the control's 20/40 and 17). The pre-registered bar is **NOT MET**, so the configuration is not kept — and the bottleneck is now located downstream of source selection.** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: pointer identity key (slot/entity address, D22 order 3) · count 0/3 · headline before reply 30/232 + memory 20/40 → after memory **19/21/23 of 40**, wrong-value **17/15/9**, BPB unchanged

## What was run

| | |
|---|---|
| base | `chat-29m-B-lr5e-4` (`d8a3c971…`), key active **from step 0** — trained in, not bolted on |
| key | `pointer_route=prime-ranked:1` — admission on a **single shared prime atom** (the entity/attribute word), ranked by the shared atom's weight; `pointer_identity=none` removes the surrounding six-token key that #2161 used |
| data / recipe | the adopted `mix-10` store, `steps=2000 batch=16 lr=2e-4 warmup=100 protocol=2 context=384 device=cuda` — D10's own recipe |
| seeds | **three genuine seeds** = three **data orders** (`data_seed` 20261009/10/11), models `71409eac…`, `70fef0ba…`, `09e9dd3d…` |
| control | **D10** (unrouted learned pointer): memory 20/40, wrong-value 17, BPB 1.17425 |

**A measurement that had to come first: for a fine-tune from a checkpoint, the data order *is* the seed.** Two arms with different `seed=` and the same `data_seed=` produced **bit-identical models** (`71409eac…`, 605 s and 572 s wall) — with `init=` and no fresh operator to initialise, `seed=` has nothing to vary. That is also why the memory arms of cycles 8–10 could use `seed=` meaningfully (their added memory drew from it) while no-memory controls could not.

## Results

| arm | data order | pointer **hit rate** | reachable | **memory** | unknowable | **wrong-value** | derangement | float BPB |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| **control D10** (learned keys) | 20261009 | 0.377 | 0.800 | **20/40** | 0/24 | **17** | 0 | **1.17425** |
| A10 | 20261009 | **0.5452** | 0.6677 | 19/40 | 0/24 | 17 | 0 | **1.17387** |
| S1 | 20261010 | **0.5419** | 0.6677 | 21/40 | 3/24 | 15 | 0 | **1.16947** |
| S2 | 20261011 | **0.5419** | 0.6677 | **23/40** | 8/24 | **9** | 2 | **1.17109** |

**Bar (pre-registered): wrong-value ≤ 12/40 and memory ≥ 25/40 on all three seeds, BPB within 0.01. NOT MET** (only S2 clears the wrong-value bar; 19/21/23 misses 25). The BPB clause is met with room to spare: **all three arms are at or below the anchor**, where the memory arms of cycles 8–10 cost **+0.026**.

## What it establishes

The hit rate is the metric #2161's key destroyed (0.003 at weight 1.0) and the one this **entity-address** key raises by **44 %**, stably. Reachability is a constant 0.6677 — the admission rule is deterministic, as the frozen-route evidence predicts — but at **0.67 against the 0.182** of the 4-token n-gram route, so a one-atom address admits roughly four times as often. And the answers still do not follow: one seed reaches the line's **best wrong-value count (9/40)** while another reads exactly the control's 17.

The brief's failure condition was *"an oracle-supplied correct slot address still fails to raise hit rate or memory"*. What was measured is one step further along — **the hit rate rises and the memory does not** — so the bottleneck is **downstream of source selection**: the gate, the register, or the decoder that must place a copied value in the answer instead of continuing the frame.

**Next:** the **forced-copy oracle instrument** the brief asks for (declared missing rather than faked — it needs a hook in the pointer op that supplies the true span), then the **copy-side levers** (`pointer_gate_supervision`, `pointer_gate_floor`), which act on exactly "found it but did not copy it", then a **combined arm** — the key is free in likelihood, so there is room to spend.

## Checklist

- [x] Checks at the exact head `9eeaf9c154700ecb65d147a1a69b73d41dc5ad85`: `python3 scripts/check_claim_wording.py` **pass** (docs-only: one record, one `current-state` entry).
- [x] Research record: `docs/labs/ptr-addr-2026-10-10/README.md`
- [x] Newest-first `docs/integration/current-state.md` entry with **Next:**
- [x] Declared deviations carried: the frozen v5 panel holds 64 rows where the brief asks ≥ 120 memory rows (owner question); the oracle ablation is named as the missing instrument
- [ ] After merge: verify on fresh `origin/main`, delete branch and worktree, `scripts/storage/uor-hygiene --apply`

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

