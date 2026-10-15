# #2112 — The v5 declared acceptance run MISSES: 10 of 40 against a declared >= 34, and the v5 binding-swap control was vacuous (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T00:19:33Z
- **Updated:** 2026-10-10T00:23:42Z
- **Closed:** 2026-10-10T00:20:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2112
- **Merged:** 2026-10-10T00:20:55Z
- **Merge commit:** cd80fe51f305b9fcae5d2958e8f09c107900ebe7
- **Base / Head:** main / UOR-Foundation:v5-declared-acceptance
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## RESULT: the run EXECUTED, and it MISSES — 10 of 40 against a declared ≥ 34

`check_pass` **10 of 40** in `multi_turn_memory`. The target, frozen before the panel was drawn, was **≥ 34**. **It misses by 24 rows.** Unknowable 1 of 24; whole panel 11 of 64 `check_pass`, 4 of 64 `acceptable`, unparsed 0. No adjustment, no re-draw, no second sample. CPU only on the laptop, **no pod, $0** (generation 2m04s, grading 9m46s).

**Identities all matching the frozen values, so the run is not void:** binary `62aa67b76764cd48…`, **grader digest `845dbda0ea48ed749caafd9e6037047aa19acfcfd82e704d7ca97d631a0b697e` (the frozen judge)**, artifact `d8a3c971…`, tokenizer `d36d3e87…`, protocol 2, cap 64, context 384, `check_panel` **pass**, **64 of 64 rows checked**, worst case `conv-v5-mem-031` **358 of 384**.

## THE CONTROL FINDING, which outranks the headline

**The v5 binding-swap control was VACUOUS from the freeze until now.** `EMBEDDED_SWAPS` held only the v3 and v4 swaps files; `conversational-v5-swaps.tsv` is committed on main (`8917b3b6…`) but was **never added**, so `swap_replies()` had no `conv-v5-*` id and the control reported **`checked_rows: 0`** while the v5 record claimed `binding_swap 0/40` — **a false statement of verification sitting on main.**

Fixed here, all three parts:
1. `conversational-v5-swaps.tsv` is now the third entry in `EMBEDDED_SWAPS`;
2. a **v5 test** asserts 40 checked rows and the first swap's text — the assertion that would have caught it (the v4 assertion at 40 is exactly why no test did);
3. **the v5 record is corrected in place**, with the vacuous row count (`checked_rows: 0`), the date, and the reason — not quietly made true.

| | before | after |
|---|---|---|
| `EMBEDDED_SWAPS` | 2 (v3, v4) | **3 (v3, v4, v5)** |
| v5 `binding_swap` | **`checked_rows: 0`** — vacuous | **0 pass / 40 checked** ✔ |
| binary | `62aa67b7…` (the run) | **`a08c38e0…`** |
| bin tests | 18 | **19 passed, 0 failed** |

**The acceptance run stands and was not re-run** — the swap control is check-only and does not touch `check_pass`; re-running the structural `check` with the fixed binary reproduces `check_panel` pass, 64 of 64, worst case 358. Every other control was as frozen: `expected_value` 40/40, `copy_first/last` 20/40, `echo_last/history` 0/40, memory constants 0/0/0, unknowable constants 24/0/0, adversarial abstentions 0, derangement 0.

## The miss, broken down so someone else can decide what it means

A 24-row shortfall is either the model or the target; **this run does not settle which.** Failures **cluster by mechanism**:

| mechanism | rows | `check_pass` | rate |
|---|---:|---:|---:|
| `exact`, distractor keys, **word** value | 23 | **9** | 39.1% |
| `exact`, distractor keys, **number** value | 11 | **1** | 9.1% |
| `exact`, no distractor keys | 6 | **0** | 0% |

All six no-key rows fail; numeric values are four times harder than word values. By failure mode over the 30 failures: **22 name neither the expected value nor the planted distractor**, **8 name the planted distractor** — the mode the key column exists to catch, so that column is doing real work — 0 empty, 0 other. Examples: `mem-002` answers *"Your kitten is Leon."* (Leon is the planted distractor; Barbara is expected); `mem-006` answers *"I'm a helpful assistant that runs on your computer."*

**No row is unpassable: the `expected_value` control passes 40/40**, so the check is not the obstacle and 10 of 40 is a model reading.

## The declared command could not run as written

`grade-replies` defaults `grader=` to `qwen2.5:1.5b`, **which is not installed**, so the declared invocation 404s in **0.063 s with no report root** — a crash, not a sample. Corrected in the v5 record with the working form verbatim (`… grader=qwen2.5:7b`). Same class as `requests=` not `panel=`: the second documented command tonight that silently selects the wrong thing. **A crash was kept apart from a miss**, which is why re-running was legitimate — had the first attempt produced a report with a bad number, that run would have been the sample and re-running would have been exactly what the pre-registration forbids.

## Criterion 1, both halves

**NOT MET on either half, each for a stated and measured reason.** The memory half is measured and **missed**: 10 of 40 against ≥ 34, on the one instrument in this line that cannot move between identical runs. The reply half cannot carry the criterion as instrumented: 43/232, failures diffuse, the cap not the constraint, 62.1% of rows with no judge-free answer, a deterministic sub-reading over 88 rows that scores 2. **The unknowable category is won by a constant** (24/24 and 14 acceptable against the model's 1 and 0, McNemar p = 2.4e-07), so that half is not a model measurement.

**Next:** do not re-run this panel for a better sample — it is development evidence now, exactly as v4 became. The measured target is explicit and unmet: **10 of 40**, clustered on numeric values (1 of 11) and the six no-key rows (0 of 6).

## Delivery

New record [`docs/labs/v5-declared-acceptance-2026-10-09/README.md`](docs/labs/v5-declared-acceptance-2026-10-09/README.md); corrections to [`panel-freeze-2026-10-09`](docs/labs/panel-freeze-2026-10-09/README.md); newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; the `EMBEDDED_SWAPS` fix and its test. Bundle on `cloud-store` **before** anything local is deleted: `v5-declared-acceptance-2026-10-09.tar`, 346,624 bytes, md5 `27518f606964dc316631e6afb0d52239`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch.

**References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

