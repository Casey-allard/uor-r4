# Evidence survey: UOR-R4, 2026-09-20 to 2026-10-05 (main at `71b54adf`, #1705)

Sources are issue #820 comments (282 since 09-20), #1552 comments since 09-28, and merged PR bodies. Every number below is quoted from those sources. I re-ran nothing. "Measured" means reported by a lab with its stated scope. Unless a row says otherwise, it has **one seed**.

## 1. Base ladder: TinyStories V2 validation NLL

Common to all rungs: quaternion transport (`r`), context 384. NLL is over 512 TS-val windows.

| Rung | Params | Shape / pattern | Read | Tokens | Data mix (TS/TD/chat-v0/chat-v1) | LR | Final TS NLL | Source |
|---|---|---|---|---|---|---|---|---|
| 8M geo-8m-a s1 | ~7.16M | 288w, `rrarra` | Lorentz | 98.3M | TS+TD+chat-v0 | – | 1.5974 (TD 1.8942, chat-v0 2.2110) | #820 2026-10-03T15:29 |
| 8M s2 (Metal) | ~7.16M | same | Lorentz | 98.3M | same | – | 1.5951 | same |
| 8M flat-L2 s1 | ~7.16M | same | L2 | 98.3M | same | – | **1.5898** (TD 1.8878, chat-v0 2.2025) | same |
| 20M s1 / s2 | – | 512/8/8 `rrarrarr` | L2 | 300M | .6/.15/.25/0 | 0.002 | 1.3882 / 1.3858 | #820 10-03T22:52 |
| 29M | – | 576/8/10 `rrarrarrar` | L2 | 434M | .6/.15/.25/0 | 0.001 | 1.2989 | same |
| 29M | – | same | L2 | 434M | same | **0.0005** | **1.2857** | same |
| ~96M geo-100m | 95,957,184 | 1024/16/14 | L2 | 1.5B | .35/.05/.10/.50 | 0.0004, bs32, `data_parallel=2` | **1.1729** | #1552 10-04T14:14 |

**Caveat.** The ~96M rung trained on only 35% TinyStories, against 60% for 20M/29M, so its TS NLL does not compare like for like across rungs. At step 5k its dev NLL was 1.694, against 1.800 for 29M lr5e-4 at the same step.

**20M learning-rate check.** 660 s runs, dev NLL at steps 5k / 7.5k:

| LR | Step 5k | Step 7.5k |
|---|---|---|
| 0.003 | 1.925 | 1.847 |
| 0.002 | 1.882 | 1.805 |
| 0.001 | 1.838 | 1.759 |
| 0.0005 | 1.817 | 1.729 |

**Matched controls** (#1639 `38377c87`, run 37084862057). Width 288, `rrarra`, 6.1M tokens, 7,155,396 params, 2 seeds:

| Arm | Mean NLL | Change vs quaternion+Lorentz |
|---|---|---|
| quaternion + Lorentz | 2.2138 | – |
| U(1) lanes + Lorentz | 2.2372 | **+0.0234** (both seeds) |
| quaternion + flat L2 | 2.1793 | **−0.0345** |

Rulings at the time: quaternion is kept, and L2 replaces the Lorentz read (#820 10-03T02:18).

The 8M transformer control (7,155,360 params, TS 1.6079, chat-v0 2.2528) is recorded at its scope only. The owner withdrew transformer controls on 10-03 (#820 10-03T17:31).

**Throughput.** CUDA parity 18/18 and later 19/19 (#1649, #1675). `data_parallel=2` gives 1.49× at 29M (58.1k → 86.8k tok/s) and 1.47× at 77.5M (#1660). TF32 gives 1.17× on the 96M fine-tune: 254.2 → 217.1 s per 1k steps, train NLL within 0.001 (#1693).

## 2. Chat fine-tunes and the 232-request open panel

**Setup** (#820 10-03T22:52):
- Recipe B: M-world plus chat-v0 responses (49,437), pointer=32, protocol 2, 4,000 steps, lr 3e-4.
- Arm C (96M): M-world ×3 plus chat-v0.
- Judge: qwen2.5:7b. Panel: everyday-32 + heldout-200-a/b, paired against a derangement control.

| Model | Acceptable | Fluent | Relevant | Control acc. | Source |
|---|---|---|---|---|---|
| chat-8m-a | 21 | 53 | 21 | 8 | #820 10-03T22:52 |
| 20M recipe B | 28 | 49 | 40 | 3 | same |
| 29M lr1e-3 B | 37 | 61 | 55 | 2 | same |
| **29M lr5e-4 B** | **43** | **70** | 53 | 4 | same |
| 96M-B | 45 | 71 | 64 | – | #1552 10-04T14:14 |
| **96M-C** | **46** | **80** | 53 | 4 | same |

**Paired McNemar tests on acceptable:**
- 29M lr5e-4 vs 20M B: p=0.024 (fluent p=0.007, relevant p=0.073).
- 29M lr5e-4 vs chat-8m-a: p=0.0001.
- 96M-B vs 29M: 19 vs 17, p=0.87 (relevant 30 vs 19, p=0.15).
- 96M-C vs 29M: 22 vs 19, **p=0.76**.

**Longer fine-tunes.** #820 10-04T19:36 says the panel "stayed flat at 43–46 acceptable from 29M to 96M, across 4,000 / 12,000 / 40,000-step fine-tunes, with and without chat-v1", with failures "mostly incoherent content on open-domain questions". The per-arm numbers for the 12k and 40k runs were not in the sources I read (**UNAVAILABLE**).

**Scope:** these are authored development panels. They are not held-out general chat.

## 3. D19 grounded session

**Setup:** 300 conversations, 1,075 scored turns, seed 9101. Compiler `compiler-save-op-v25-rawtable` with trunk `op-model-v25`, `op_policy=unless_query`. The pod reproduced the laptop's score exactly per category (960).

| Model | Sieve ON total | Sieve OFF total | MQAR on → off | Instr. | Relation | Copy |
|---|---|---|---|---|---|---|
| chat-8m-a | 959 | – | 107 → – | 67 | 75 | 31 |
| 20M B | 960 | 857 | 104 → 1 | – | – | – |
| 29M lr1e-3 | 929 | 823 | 107 → 1 | – | – | – |
| 29M lr5e-4 | 972 | 865 | 108 → 1 | 56 | open 42/52, closed 17/17, abstain 8/8 | – |
| 96M-B | 984 | 883 | 106 → – | 68/98 | 73/83 | 29/33 |
| **96M-C** | **1008** | 908 | 105 → – | **78/98** | 75/83 | 31/33 |

- **Network-alone check** (re-checked from sealed L12 reports `session-off/sieve-chat-100m-L12`): **MQAR 3/109 with the sieve off** (the easiest cell, D16×N2, is 1/13; every D200 cell is 0), against 105/109 with the sieve on (#820 10-04T20:10).
- **Caveat:** M-world is about 2% of the fine-tune mix, or about 0.8% of tokens per #820 10-05T00:36.
- **Only MQAR changes with the store.** Every non-MQAR category is identical with the sieve on and off (#820 10-03T22:52).
- **Store effect depends on the model** (#820 10-03T19:54, world dev split, 23 MQAR):
  - `wide-bind-1` (no pointer, 9.2M tokens): 0 → 2.
  - `chat-8m-a` (pointer=32): 1 → 22.
  - Pointer and scale are confounded here.
- **Earlier milestones:** emit-3 session MQAR 42 → 79/109 with log recall (#1600).
- **Binding pre-test, 8M** (A1 dev cell, `recall=off`): MQAR at d16 is 1/37 for both chat-8m-a and emit-6r, against 13/37 for the recency rule. The kill criterion fired (#820 ~10-03T03:29).

## 4. Integer (D11) serving vs float

| Model / arm | Measure | Integer | Float | Test | Source |
|---|---|---|---|---|---|
| geo-20m, round-to-nearest, D10 | TS NLL, 64 windows | 1.4163 | 1.3872 | +0.0291 nats, top-1 agreement 0.927 | #1667 |
| geo-20m D11 vs D10 | logits, 3,072 targets | bit-identical (max diff 0) | – | – | #1667 |
| 29M lr5e-4, round-to-nearest | acc / fluent / rel | 31 / 54 / 43 | 43 / 70 / 53 | acc p=0.017, fluent p=0.009 | #820 10-04T06:59 |
| 29M lr5e-4, GPTQ (64 windows, 16 s) | acc / fluent / rel | 33 / 70 / 46 | 43 / 70 / 53 | acc p=0.053, fluent p=1.0, rel p=0.21 | same; `model.lut` 53e3964c…, 16.9 MB |
| 96M-C, GPTQ (36 s) | acc / fluent / rel | 43 / 71 / 55 | 46 / 80 / 53 | acc p=0.71, fluent p=0.18, rel p=0.86 | #820 10-04T14:53; 53.6 MB |
| 29M B, D11 greedy agreement | everyday-32 | 13/32 requests and 20/40 turns id-identical | – | – | #1676 |

**D11 speed on the M1** (#1691), in generated ids/s:

| Model | 1 thread, before | 1 thread, after | 4 threads, after |
|---|---|---|---|
| 29M | 26.7 | 55.0 | 97.0 |
| 96M | 8.7 | 16.3 | 32.5 |

Before #1691 the engine was single-threaded, so the 4-thread setting gave no gain (27.0 and 8.67).

- **Tokens:** 32/32 rows are identical at every thread count.
- **Multiplier-free audit:** FULL PASS, 0 forbidden instructions over 44 symbol ranges and 83 call-graph functions. Rayon and crossbeam scheduler code is excluded from the walk, which is a stated policy decision (#1691).
- **Pointer head:** served in D10 and D11 with schema `/2` (#1669).
- **Key shift:** refused by every export and served path; it has no served form yet (#1704).

## 5. MQAR bench: network-alone in-context recall

**Common settings** (#1698, #1701, #1704):
- Model: width 128, 4 heads, MLP 384, 6 layers, about 1.36–1.37M params.
- Training: context 512, 1,800 steps × batch 8, lr 1e-3, CPU at 4 threads.
- Scoring: 512 fresh-pairing queries per bucket. Chance is 1/224 = 0.0045. d1000 is **UNAVAILABLE** at context 512.

**Binaries and sealed roots:**
- `eba6a15f`: `grid3-ctx512-eba6a15f`
- `30502cd7`: `fixes-ctx512-30502cd7`
- `787c350d`: `key-shift-production-787c350d`

| Arm | Seed | d16 | d64 | d200 | d400 | All | Held-out class | Positions scored per query | First ≥0.9 |
|---|---|---|---|---|---|---|---|---|---|
| A `aaaaaa` l2 | 1 | 0.807 | 0.176 | 0.010 | 0.029 | 0.255 | 0/1024 | 8,564 | never |
| A | 2 | 0.799 | 0.188 | 0.023 | 0.039 | 0.262 | 1/1024 | 8,523 | never |
| A-dot | 1 | 0.777 | 0.123 | 0.014 | 0.012 | 0.231 | 0/1024 | 8,564 | – |
| A, age bias zeroed | 1 | 0.115 | 0.027 | 0.010 | 0.010 | 0.041 | – | – | – |
| F1, age-slope spread | 1 | 0.398 | 0.041 | 0.014 | 0.014 | 0.117 | 0/1024 | – | never |
| **F2, key `k_t + j·k_{t−1}`** | 1 | **1.000** | **1.000** | **1.000** | **1.000** | 1.000 | **1024/1024** | 8,564 | step 400 |
| F1+F2 | 1 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1024/1024 | – | step 600 |
| `rrarra`, no shift | 1 | 0.014 | 0.008 | 0.002 | 0.006 | 0.007 | 0/1024 | 2,855 | never |
| `rrarra`, no shift | **2** | 1.000 | 1.000 | 0.998 | 1.000 | 1.000 | 1024/1024 | 2,841 | 400 |
| `rrarra` + shift | 1 / 2 | 1.000 all | 1.000 all | 1.000 all | 1.000 all | 1.000 | 1024/1024 | – | 700 / 600 |
| `rararr`, no shift | 1 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1022/1024 | 2,855 | 500 |
| `rararr` + shift | 1 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1024/1024 | 2,855 | **200** |
| `rrrrrr` (recurrence only) | 1 | 0.020 | 0.010 | 0.000 | 0.006 | 0.009 | – | 0 | – |
| Recency baseline | – | 0.445 | 0.051 | 0 | 0 | – | – | – | – |

**Rotation (F3).** `rotation=true` touches only the recurrence. On an all-read stack, rotation on and off give bit-identical logits.

**Probe: best head's weight on the true key / true value position** (#1701):
- Arm A at the end of training has about uniform weight on the key at every distance, and 0.045 on the value at d200. It reads a "bag of values".
- F2 at step 1800 puts 1.000 / 1.000 / 1.000 / 0.994 on the value position at d16 / d64 / d200 / d400.
- The phase transition under F2 is sharp: in-class accuracy is 0.15 at step 300 and 0.98 at step 400.

**What the result does and does not show.**
- The fix supplies predecessor identity in the read's keys. It is the standard previous-token / token-shift route, realised as an exact quaternion quarter-turn: a signed permutation plus an add, with zero parameters.
- It is a necessary mechanism, but not by itself evidence of a geometric advantage. The PR author says this explicitly (#820 10-04T23:03).
- #1704 corrects #1698: the claim that recurrence crowds out the reads was specific to seed 1.
- **Not measured yet:** a chat model trained with the shift, and the D19 MQAR cell with the sieve off. The pod A/B (key shift off vs `key_shift=add`) was scheduled after the base finished.

## 6. DeepSeek compiler line (#1552; PRs #1659, #1663, #1673, #1678, #1690)

**Disjoint panel.** 100 rows, 10 relations. Requests `3e5d481b…`, expected `4fae646a…`. The Rust port is byte-identical (#1659). Run settings: seed 9101, `max_new_tokens` 32.

**Calibration** (#1663; pre-#1678 binary `ea173f13…` for 96M):

| Model | Recall | ok/100 | asked_stored | store_read | log_no_st | other |
|---|---|---|---|---|---|---|
| chat-8m-a | off / sieve | 9 / 24 | 50 | 8 | 0 / 16 | 1 / 0 |
| 20M B | off / sieve | 12 / 24 | 50 | 7 | 0 / 15 | 5 / 2 |
| 29M lr5e-4 | off / sieve | 10 / 23 | 50 | 7 | 0 / 13 | 3 / 3 |
| 96M-B | off / sieve | 10 / 25 | 50 | 8 | 0 / 17 | 2 / 0 |
| 96M-C | off / sieve | 13 / **29** | 50 | 8 | 0 / 18 | 5 / 3 |

- **Paired tests vs 29M (sieve):** 96M-B +2, p=0.79; 96M-C +6, p=0.18.
- **The pre-registration is not confirmed.** `log_no_st` moved; `store_read` did not.

**Diagnosis split** (#1673). Identical in every arm because the compiler is shared and fixed:

| Bucket | Pre-#1678 | After #1678 (wa/1) |
|---|---|---|
| fact_unresolved | 8 | 8 |
| distractor_only | 42 | 37 |
| question_not_a_query | 32 | 36 |
| query_but_no_read | 9 | 10 |
| read_wrong_value | 1 | 1 |
| stored_and_read | 8 | 8 |

**#1678, word alignment** (`9a88bd0a`):
- `asked_stored` rose 50 → 55, and truncations fell 7 → 2 at 8M.
- **Per-row correctness changed in 0 rows** on 29M, 96M-B and 96M-C (#1552 10-04T14:58).

**Root cause of the 36 non-query rows** (#1552 10-04T15:40 and 16:58):
- Zero of the 36 come from `parse_op`.
- 21 rows: "the heads name no relation" (`relation_compiler.rs:2036`).
- 15 rows: the question is classified as `assert` or `correct`.
- **Relation identity is a closed 11-label set:** `user_name`, `pet_name`, `friend_name`, `hometown`, `lucky_number`, `code_word`, `job`, `home`, `favorite_food`, `favorite_color`, `none`.
- The M-world generator has 5 relations. Only 1 of the panel's 10 relations is in the compiler identity.
- The v26 query-form card was superseded. Adding the panel's relations was rejected as teaching to the instrument.

**Fix B: deterministic relation phrase from the turn's own words** (relation-split dev set, 200 rows):

| Step | Result |
|---|---|
| Extractor | 140 extracted, 0 wrong, 60 declined; equals the frozen ceiling 140/200 |
| First wiring | 84 → 36 |
| Guard moved to the `UnlessQuery` arm | → 69 |
| Sieve fallback on a resolved miss | 69 → 69 |
| `?` → `QueryCurrent` rule | **→ 137/200**: store_read 137, stored_not_recalled 60 (all anaphoric "What is it?" turns), misrendered 3, 636 tests pass |

- The world gate is exact: MQAR 22/23, open relation 11/12, closed relation 3/3.
- **Labelled a development result** (#1552 10-04T23:54).
- **Open blocker:** `open_relations()` is a process-global `OnceLock`.
- **Unmeasured defect:** the op model writes prose as facts ("The weather is nice today."); its false-write rate has not been measured.

**Other levers on the 8M dose line (all negative or neutral):**

| Lever | Result |
|---|---|
| Distance curriculum | 14/120 → 3/120, Fisher p=0.0099; the budget match halved practice, so the cause is not isolated |
| Per-channel decay | neutral (0.22% worse) |
| Delta-rule memory | dominated by the exact store (1.0000 vs 0.0703 prototype) |
| Surface-form breadth | statement prefixes 37.8% → 94.6%; questions 58.1% → 63.5%, p=0.54 |

The 14/120 figure depends on the scorer: 31/120 under lenient subsequence scoring, 9/120 under the strict "only one value" rule.

## 7. Codex native geometric attention (CPU-only fits; #1695, #1700, #1703, #1705)

| PR | Change | Measured | Adopted? |
|---|---|---|---|
| #1695 | 64-update native route-credit fits | Joint64 CE 0.739, 20/64; Role64 CE 0.810, 15/64; parent CE 0.492, 31/64; fresh 0/32 | no; regressions |
| #1700 | causal multi-source bank fit | bank CE 2.163 → 1.736; all single-source rows regress; fresh 0/32 | parent retained |
| #1703 | readout-only on frozen context | full CE 1.328 → 1.146; single complete 31 → 34/64; bank 0 → 1/64; fresh 0/32 | no promotion |
| #1705 | learned directed signed-H4 cue term | dev CE 1.146 → 1.040 (cue-unary 1.124); banks 2/64 vs 1/64; fresh 0/32 vs 1/32 | no promotion |

The #1705 PR states that its result shows "learned native cue influence, not reliable geometric attention".

## 8. Measured facts: what scales and what does not

**Scales with parameters and tokens:**
1. **Base LM NLL.** 1.59 at 8M, then 1.386–1.388 at 20M, 1.286 at 29M and 1.173 at ~96M. The mixes differ across rungs, and the 96M rung saw less TinyStories.
2. **Open-panel acceptable, 8M → 29M.** It rose 21 → 28 → 43, significant against 20M (p=0.024) and against 8M (p=0.0001). Fluency also rose, 53 → 70.
3. **Grounded session total.** 959 → 960 → 972 → 1008. From 29M to 96M the gain was mostly instruction following in Arm C, 56 → 78/98.
4. **Integer-serving fidelity.** The GPTQ acceptable gap was −10 at 29M (p=0.053) and −3 at 96M (p=0.71).
5. **D11 speed.** About 3.6× faster at 4 threads with identical tokens. This is an engineering gain, not a scale effect.

**Does not scale:**
1. **Open-panel acceptable, 29M → 96M.** 43 → 45/46 (p=0.76–0.87). It stayed at 43–46 across 4k/12k/40k-step fine-tunes and with or without chat-v1 (summary claim; per-arm figures UNAVAILABLE).
2. **Network-alone session MQAR.** 0–5/109 at every scale; 3/109 at ~96M. With the sieve the session gets 103–108/109. The fine-tune contained only about 0.8–2% MQAR-format data, so this does not separate "cannot" from "never trained to".
3. **Disjoint-panel `store_read`.** 8 / 7 / 7 / 8 across 8M / 20M / 29M / 96M. The bottleneck is the closed 11-label relation identity in the compiler, which no emitter scale affects. Only the log-sieve path moved (13 → 17/18, not significant).
4. **#1678 storage fix.** `asked_stored` rose 50 → 55 with zero correctness change.
5. **8M learned binding.** MQAR at d16 was 1/37, below the recency rule's 13/37.

**Changed by mechanism, not scale (bench scope only: about 1.4M params, synthetic MQAR, 1–2 seeds):**
1. **Predecessor identity in read keys.** `k_t + j·k_{t−1}`, a zero-parameter quaternion quarter-turn, gives 1.000 recall at d16–d400 and 1024/1024 on the held-out pairings, on every pattern and seed tried.
2. **Without it, recall depends on the pattern.** All-read stacks fail on both seeds. `rrarra` succeeds only on seed 2, because recurrence before a read can supply the predecessor through its width-4 causal conv. `rararr` succeeds.
3. **Where it helps and where it is untested.** The shift makes recall robust and speeds learning (step 200 vs 500 on `rararr`). Its effect on the chat models, the D19 sieve-off cell and D11 serving is **not measured**.
4. **On the compiler side, Fix B** (relation address from the turn's own words, plus the `?` rule) raised dev panel memory from 84 to 137/200. This is a development result. The remaining 60 rows need anaphora or previous-turn state.

**Geometry vs matched controls** (8M or smaller, 2 seeds):
- Quaternion beats U(1) lanes by 0.0234 nats, about 2–3× the seed spread.
- The flat L2 read beats Lorentz by 0.0345 nats.
- No measured semantic or recall advantage is established for prime, zeta or H4 mechanisms. Every Codex native-attention fit gives 0–1/32 on its fresh transfer set.