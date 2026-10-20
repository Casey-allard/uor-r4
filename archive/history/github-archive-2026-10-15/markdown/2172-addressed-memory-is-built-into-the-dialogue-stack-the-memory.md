# #2172 — Addressed memory is built into the dialogue stack; the memory op's missing device support is the measured limit (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T18:41:35Z
- **Updated:** 2026-10-10T19:22:19Z
- **Closed:** 2026-10-10T19:14:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2172
- **Merged:** 2026-10-10T19:14:33Z
- **Merge commit:** 69905327e168264d5c3d47c31b3895855084d9b6
- **Base / Head:** main / UOR-Foundation:deepseek/addressed-memory-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**D22 order 2 says the addressed-memory path is this lab's own source work, not an owner block, and order 1 adopts the 10 % recall dose as the M1 base. Both are delivered here — with the mechanism **trained in** from the first step of a fine-tune rather than bolted onto a finished model — and with one measured limit that shapes what the test can be: the product-key memory is a **CPU custom op with no CUDA path**, so a memory-equipped 29M stack fits at 3.7 s/step best case instead of minutes on the GPU.** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: exact addressed memory (D22 order 2) · count 0/3 · headline before reply 37/232 + memory 20/40 → after: base adopted, memory arms **fitting**

## Order 1, recorded: the 10 % recall dose is the M1 base

D10 (`runs2/dose-D10`: mixed **10.21 %** recall share, 2,000 steps, `data_seed=20261009`) is the artifact every M1 arm is now measured against — v5 memory **20/40** `check_pass`, open reply panel **37/232** `fluent_and_relevant` (43 → 37, paired p = 0.42, inside noise), unknowable 0/24, derangement 0. It was rejected by one row under D21; under D22 it is kept and iterated on, and `STATUS.md` and `current-state.md` say so.

## Order 2, landed as source (both PRs merged)

| PR | merge | what it does |
|---|---|---|
| [#2168](https://github.com/UOR-Foundation/uor-r4/pull/2168) | `a5e140669` | `memory_config_from_args()` shared with `stack_config`; the seven memory options accepted beside `init=` (adding on a model saved without memory, requiring exact agreement on one that has it); `StackModel::add_memory_layers` drops the named layers' `mlp.gate/up/down` and initialises the memory's own **exactly as a fresh construction with that configuration and seed would**; `dialogue_train` installs it **after `init=` and before the optimizer**; `DialogueSettings` carries the parsed memory; the option names join the allowlist |
| [#2170](https://github.com/UOR-Foundation/uor-r4/pull/2170) | `14a909885` | the run's `seed=` reaches an added memory, as `pointer=DIM` takes it — found by running it: two arms with different seeds were **bit-identical at step 0**, so the pre-registered two seeds would have been one run twice. Re-lands #2169, which I opened against the preparation branch instead of `main` and which therefore merged off main |

Focused checks at those heads: a library test asserting the added memory replaces layer 1's MLP and is **equal element-wise to a fresh `StackModel::new`** with that configuration and seed, idempotent for the same memory and refused for a different one; a CLI test covering add/agree/refuse, the companion-without-`memory_layers=` refusal, the codebook defaults and the seed reaching the configuration; the example suite (13 tests) and the claim-wording gate.

## The measured limit, named because it is the result of this cycle

`Error: Tensor(no cuda implementation for geometric-stack-product-key-memory)` — the product-key memory is a `CustomOp3` with only `cpu_fwd`, so `device=cuda` refuses a memory-equipped stack **outright**. Measured on this pod: **3.7 s/step** for one arm at 24 threads, **~9–10 s/step** with two arms on its cores. At the pre-registered 2,000 steps that is a **five-hour fit**, so the two seeds are fitting at **1,000 steps** each (stated in the record's limitations, not hidden) on the adopted `mix-10` store with the memory present from step 0: `memory_layers=4` of the `rrarrarrar` stack, 64 sub-keys, 16 top-k, 4 heads, key width 64, seeds `20261010` and `20261011`.

**Their step-0 dev NLLs differ (2.230381 vs 2.229698)** — the pre-registered check that the two seeds are two mechanisms — and the first interval reads 1.2279. **No KEEP/REJECT is claimed for the memory configuration:** the arms are unscored, which under D22 §1 is not a negative about addressed memory either way. The readings land on #2029 when the fits complete, and the next piece is the fix that makes them affordable: **implement the product-key memory for the served device**, validated by the parity test the added-memory construction already passes.

## Also in this delivery (D22 §6)

The README's **How it works** and **Results so far** sections now carry the product-key memory's status and its CPU-only limit, and the M1 line's three headline readings — v5 memory (20/40 at the adopted dose), the open reply panel (37/232 at that dose, 39/232 with read-binding supervision), and criterion 2's served **1.20138 BPB** bit-exact against a ≤ 0.90 target. None of the M1 results since 9 October had reached the README.

## Checklist

- [x] Checks at this exact head: `python3 scripts/check_claim_wording.py` (docs-only change: one record, one `current-state` entry, `STATUS.md`, the README's two mechanism sections). The source changes landed separately in #2168/#2170 with their own exact-head checks.
- [x] Research record: `docs/labs/addressed-memory-2026-10-10/README.md`
- [x] Newest-first `docs/integration/current-state.md` entry with **Next:**
- [x] `STATUS.md` row and M1 cell (base adoption + the memory source); README mechanism sections
- [x] Evidence: the fit logs and (when they land) the arm reports on the pod volume `/workspace/uor-r4/deepseek/mem-in-20261010/`, with the cloud-store object named in the record
- [ ] After merge: verify on fresh `origin/main`, delete branch and worktree, `scripts/storage/uor-hygiene --apply`, post the fit's readings on #2029

References #2029


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6100910976 — 2026-10-10T18:41:42Z — Casey-allard

**Validation at the exact head `559e4f1f78fe3e81e83b8088dcf68d8b7d08a705`** (docs-only: one record, one `current-state` entry, one `STATUS.md` row and cell, two README mechanism sections; no Rust path is touched here — the source changes landed in #2168 and #2170 with their own exact-head checks).

| check | command | result |
|---|---|---|
| claim wording | `python3 scripts/check_claim_wording.py` | **pass** |
| README mechanism sections (D22 §6) | `git diff origin/main -- README.md` | **How it works** gains the product-key memory's status and its measured CPU-only limit; **Results so far** gains the M1 line's three readings (v5 memory 20/40 at the adopted dose; reply 43 → 37/232 at that dose and 39/232 with read-binding supervision; criterion 2's 1.20138 BPB served bit-exact against ≤ 0.90) |
| the base is recorded where state changes | `STATUS.md`, `current-state.md` | D10 (the 10 % recall dose, #2151) named as the M1 base under D22 order 1 |
| the tooling limit is measured, not asserted | the pod's job logs | `no cuda implementation for geometric-stack-product-key-memory`; 3.7 s/step at 24 threads, ~9–10 s/step with two arms |
| the two seeds are two mechanisms | both arms' step-0 lines | dev NLL **2.230381** vs **2.229698** |



---
_Comment/review ids in this file: 6100910976_

