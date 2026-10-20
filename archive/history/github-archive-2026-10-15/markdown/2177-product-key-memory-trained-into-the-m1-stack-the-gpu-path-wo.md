# #2177 — Product-key memory trained into the M1 stack: the GPU path works (0.19 s/step) and the configuration reads 19/40 against the brief's 27/40 (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T21:03:55Z
- **Updated:** 2026-10-10T21:49:23Z
- **Closed:** 2026-10-10T21:05:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2177
- **Merged:** 2026-10-10T21:05:58Z
- **Merge commit:** 401bc71452c966a61fed4862e183fc980c5bdb63
- **Base / Head:** main / UOR-Foundation:deepseek/addressed-memory-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The device fix works and is measured; the configuration at the anchor's own steps does not.** The product-key memory's **CUDA forward is its exact CPU forward on host copies** ([#2175](https://github.com/UOR-Foundation/uor-r4/pull/2175)), so the same two-seed fit that needed **7.4 s/step on CPU** now runs at **0.19 s/step** — 2,000 steps in **387 s** per arm, both seeds in parallel, CPU path untouched. With the mechanism present from **step 0** and D10's recipe, the pre-registered bar (**memory ≥ 27/40 on both seeds, BPB within 0.01**) is **NOT MET on both halves**: **19/40** and **19/40**, and **1.20184 BPB against the anchor's 1.17425 (+0.0276)**. **REJECT for this configuration; the mechanism stays open (D22 §1).** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: exact addressed memory (D22 order 2) · count 0/3 · headline before reply 37/232 + memory 20/40 → after reply *pending* + memory **19/40** (both seeds)

## The run and its readings

| arm | steps | device | v5 memory `check_pass` | unknowable | wrong-value | derangement | float BPB (chat held-out) |
|---|---:|---|---:|---:|---:|---:|---:|
| D10 (adopted base, **no memory**) | 2,000 | GPU | **20/40** | 0/24 | 17 | 0 | **1.17425** (nll 2.30985 / 6,169,728 targets) |
| **G1** (memory, seed 20261010) | 2,000 | GPU | **19/40** | 0/24 | 17 | 1 | **1.20184** |
| **G2** (memory, seed 20261011) | 2,000 | GPU | **19/40** | 0/24 | 15 | 2 | pending |
| *cpuM1 (memory, seed 20261010, **half the steps**)* | 1,000 | CPU | *22/40* | *2/24* | ***11*** | *2* | pending |

Memory `memory_layers=4` of the 10-layer `rrarrarrar` stack, `memory_sub_keys=64`, `memory_top_k=16`, `memory_heads=4`, `memory_key_dim=64`, `memory_score=dot`, on the adopted `mix-10` store with `steps=2000 batch=16 lr=2e-4 warmup=100`, i.e. **D10's own recipe**, so the contrast is the memory operator. Pre-registered with a [`TEST FITNESS: FIT` review](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6101635064) and two declared deviations (the frozen panel has 64 rows where the brief asks 80; the tagger/read split belongs to the exact-key arm).

## What this does and does not close

**Closes:** one configuration — memory at layer 4, 64 sub-keys, 16 top-k, replacing that layer's MLP, 2,000 steps from the chat base, two seeds. It does not beat the anchor on the panel and costs 0.0276 BPB on the chat held-out stream.

**Does not close:** the mechanism. The device fix is a capability gain with a measured number (**39× faster**), and the brief's **exact-key arm** (prime/semiprime addressed store) is a different path this run did not test.

**Names the next step:** the same memory at **half the steps on CPU** reads **22/40** (above the adopted base) with **11 wrong-value failures**, the best on this line. It differs in step count *and* device, so the next cycle opens with the **step-matched control** — the identical recipe with no memory at 1,000 steps, two seeds — now cheap at ~6.5 min per pair. Then configuration variation at the anchor's steps (memory at a **read** layer; larger sub-key sets), then the exact-key arm.

## Checklist

- [x] Checks at this exact head: `python3 scripts/check_claim_wording.py` (docs-only change: the record, a `current-state` entry, `STATUS.md`, the README's two mechanism sections). The source change landed separately in #2175, whose CUDA build and parity test ran on the pod.
- [x] Research record: `docs/labs/addressed-memory-2026-10-10/README.md`
- [x] Newest-first `docs/integration/current-state.md` entry with **Next:**
- [x] `STATUS.md` row; README **How it works / Results so far** updated where the mechanism result changed (D22 §6)
- [x] Evidence: arm reports, models, BPB reports and job logs on the pod volume `/workspace/uor-r4/deepseek/mem-in-20261010/`, pulled to the laptop and bundled to cloud-store
- [ ] After merge: verify on fresh `origin/main`, delete branch and worktree, `scripts/storage/uor-hygiene --apply`, post the reply-panel and remaining BPB readings on #2029

**Declared pending:** G1's open reply panel (232 rows) is grading in the shared judge queue, and G2's/cpuM1's BPB reports are still running; all posted on #2029 when they land. The verdict is decided by the memory half and the BPB, both already measured.

References #2029


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6102155996 — 2026-10-10T21:04:01Z — Casey-allard

**Validation at the exact head `9509e002bdff6a2f7dc9de07773129f2018e7520`** (docs-only: the record for this configuration, a `current-state` entry, a `STATUS.md` row, the README's two mechanism sections; the source change is the separate prep PR #2175).

| check | command | result |
|---|---|---|
| claim wording | `python3 scripts/check_claim_wording.py` | **pass** |
| the device fix is real and measured | the pod's two-seed fit | 2,000 steps in **377 s / 379 s** wall (both arms in parallel) against **7.4 s/step** measured on CPU — the same job that had been a 2.5-hour CPU fit per arm |
| v5 acceptance | `chat-grade grade-replies`, frozen `conversational-v5-checks.tsv` | **G1 19/40**, **G2 19/40**, unknowable 0/24 both, derangement 1 and 2; anchor (D10) 20/40 |
| float BPB bar | `geometric-stack evaluate`, same lens, whole held-out stream | **anchor 1.17425**, **G1 1.20184** → **+0.0276**, outside the bar's 0.01 |
| bar | pre-registered on #2029 before compute | **NOT MET on both halves** → REJECT for this configuration, mechanism open |



---
_Comment/review ids in this file: 6102155996_

