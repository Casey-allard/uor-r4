# The v5 declared acceptance run: EXECUTED, and it MISSES — 10 of 40 against a declared ≥ 34

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek. 2026-10-09. **CPU only on the laptop: no
pod, no GPU, no training. $0.** Generation 2m04s, grading 9m46s.

## The declared run, and what it is

The v5 memory panel was frozen, drawn not chosen, byte-reproducible, provenance-recorded and
structurally validated before any model replied to it, with the acceptance run declared in advance so
it could not be improvised later
([panel-freeze-2026-10-09](../panel-freeze-2026-10-09/README.md)). The declared reading:

> Target: **≥ 34 of the 40 `multi_turn_memory` rows at `check_pass`**. `acceptable` and judge-changed
> rows are secondary and non-gating, because the judge is not deterministic.

**The run happened once and this is it.** The panel becomes development evidence from here, exactly as
v4 did.

## RESULT

| | memory (`multi_turn_memory`, 40 rows) | unknowable (`unknowable_or_impossible`, 24) | whole panel (64) |
|---|---:|---:|---:|
| **`check_pass`** | **10 / 40** | **1 / 24** | **11 / 64** |
| `acceptable` | 4 | 0 | 4 |
| fluent | 5 | 2 | 7 |
| relevant | 5 | 0 | 5 |
| unparsed answers | 0 | 0 | 0 |

**THE DECLARED TARGET WAS ≥ 34 OF 40. THE READING IS 10 OF 40. THE RUN MISSES, BY 24 ROWS.** No
adjustment, no re-draw, no second sample, no tuning: a miss is a result, and this is it.

## Identities and conditions — every one matching the frozen values

| | value |
|---|---|
| `executable_sha256` (the binary run) | **`62aa67b76764cd48170d0e90ff168a176b471e5d3f4a84c0748e399659e83984`** |
| source revision | main `cb81cde84`; the binary's `chat-grade.rs` verified byte-identical to the built tree's |
| **grader digest** | **`845dbda0ea48ed749caafd9e6037047aa19acfcfd82e704d7ca97d631a0b697e`** — **the frozen digest**, so the run is not void |
| grader | ollama `qwen2.5:7b` local, temperature 0, seed 1, `num_predict` 4 |
| artifact | `model.safetensors` sha256 `d8a3c971ef07a82ca0db0704b34821e24782c7734baaafb504fd754f82988068` (the sealed 29M `chat-29m-B-lr5e-4`) |
| tokenizer | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` (the #1017 ladder tokenizer) |
| protocol | `protocol=2`, `max_new_tokens=64`, `context=384` |
| `check_panel` | **pass**, 64 of 64 rows checked, `checks_without_loaded_request: []` |
| checks sha256 | `737c4dfd5a65fe49c207afe40e88bf354c7cf9dd45fa173c54e639bb0aa0dbc8` |
| **worst-case context position actually used** | **`conv-v5-mem-031`, 358 of 384** |
| wall | generation 124.1 s (2,154 ids); grading **586.2 s** |

### Two argument facts that cost a step each, recorded so nobody repeats them

1. **The declared command as written cannot run.** `grade-replies` defaults `grader=` to
   `qwen2.5:1.5b`; that model is not installed (`ollama list` shows only `qwen2.5:7b 845dbda0ea48`),
   so `/api/chat` returns 404 and curl exits **22**. The first attempt died in **0.063 s** with no
   report root — **a crash, not a sample**. The working invocation is the declared one plus the frozen
   grader:
   ```text
   chat-grade grade-replies out=<NEW_REPORT_ROOT> \
     replies=<replies.json> checks=data/panels/conversational-v5-checks.tsv grader=qwen2.5:7b
   ```
   A crash was not treated as a sacred first attempt: it produced no report, so nothing was
   contaminated and the run was executed once, properly.
2. **`reply` takes no `context=`** (the context comes from the model config, 384 here), and `model=`
   must point at the directory holding `config.json` and `model.safetensors`, not its parent.

## Controls — all as frozen, and one that is NOT

Every control the freeze declared, at this run:

| control | frozen | this run | |
|---|---|---|---|
| `expected_value` (memory) | 40/40 | **40/40** | as frozen |
| `copy_first_stated` / `copy_last_stated` | 20/40 each | **20/40 each** | as frozen, neither copy control vacuous |
| `echo_last` / `echo_history` (memory) | 0/40 | **0/40** | as frozen |
| memory constants | 0/0/0 of 40 | **0/0/0** | as frozen |
| unknowable constants | 24/0/0 of 24 | **24/0/0** | as frozen — the abstention constant passes all 24 |
| adversarial abstentions | 0 | **0** | as frozen |
| derangement | relevance must fall | **`acceptable` 0, `check_pass` 0 of 64** | as frozen |
| unparsed answers | — | **0** | — |
| **`binding_swap` (v5)** | the record claims **0/40** | **`checked_rows: 0`** | **VACUOUS — see below** |

### THE CONTROL FINDING: the v5 binding-swap control has never run

`EMBEDDED_SWAPS` in `crates/uor-r4-training/src/bin/chat-grade.rs` is a **two-element** array holding
only `conversational-v3-swaps.tsv` and `conversational-v4-swaps.tsv`. **`conversational-v5-swaps.tsv`
is committed on main (`8917b3b6c296ba3616143fd31e132a1e9f9fbcec`) but was never embedded** —
`git log -S "conversational-v5-swaps" -- crates/uor-r4-training/src/bin/chat-grade.rs` returns nothing.
So `swap_replies()` holds no `conv-v5-*` ids and the control reports **`checked_rows: 0`** in every
binary built from main, including at the freeze.

**The v5 record's `binding_swap 0/40` is therefore not reproducible, and that control has been vacuous
for v5 since the panel was sealed.** The v4 test asserts 40 checked rows, which is why no test caught
it — nothing asserts the v5 row.

**Does it invalidate this reading? No, and that is stated rather than assumed.** `check_pass` — the
gating reading — does not use binding swaps, and every control that bears on it (`expected_value`,
the copy controls, the constants, the adversarial abstentions) is non-vacuous and behaves exactly as
frozen. The binding-swap control tests whether the *judge* responds to relevance, which is a property
of the secondary `acceptable` reading; its absence makes that reading weaker, not the check reading
invalid. **Fixed in this same delivery** (the Lead's correction, and rightly): `conversational-v5-swaps.tsv` is
now the third entry in `EMBEDDED_SWAPS`, a v5 test asserts 40 checked rows and the first swap's text
(the assertion that would have caught it — the v4 assertion at 40 is why nothing did), and the v5
record's false `0/40` is **corrected in place** rather than quietly made true.

| | before the fix | after |
|---|---|---|
| `EMBEDDED_SWAPS` | 2 entries (v3, v4) | **3 entries (v3, v4, v5)** |
| v5 `binding_swap` | **`checked_rows: 0`** — vacuous, from the freeze until now | **0 pass / 40 checked** ✔ every swap fails, as the freeze intended |
| binary sha256 | `62aa67b76764cd48…` (the acceptance run) | **`a08c38e0a5f5e2cd…`** |
| bin tests | 18 | **19 passed, 0 failed** |

**The acceptance run stands and is not re-run:** it used `62aa67b7…`, and the swap control is
check-only — it does not touch `check_pass` or the judge. Re-running the structural `check` with the
fixed binary reproduces `check_panel` **pass**, 64 of 64 rows checked and worst case **358** of 384,
with `binding_swap` now non-vacuous. The model's replies are unchanged by this fix.

## What the run actually shows

1. **The memory half does not meet criterion 1's declared target.** 10 of 40 against ≥ 34. This is the
   first reading in this line taken on an instrument that cannot move between identical runs — frozen
   checks, values with per-value provenance, controls verified before the panel was sealed — and on
   that instrument the model scores a quarter of the bar.
2. **The unknowable half is won by a constant.** The abstention constant `"I'm not sure. Can you tell
   me more about what you mean?"` passes **24 of 24** `check_pass` and scores **14 acceptable**; the
   model scores **1 of 24** and **0 acceptable** (McNemar exact **p = 2.4 × 10⁻⁷**, 23 constant-only
   against 0 model-only). Per the report's own rule — *a category the model does not beat its
   constants on measures nothing about the model* — **the unknowable category is not a model
   measurement at all.**
3. **On the memory rows the model does beat its constants on the deterministic check** (10 against 0,
   McNemar exact **p = 0.00195**, 10 model-only against 0 constant-only), so the 10 of 40 is a
   genuine, if poor, model reading. On the secondary `acceptable` reading it does not reach
   significance (4 against 0, p = 0.125), and the report flags the overall `acceptable` reading
   `discriminates: false`.
4. **A constant outscores the model on the panel as a whole**: 24 `check_pass` and 14 `acceptable`
   against the model's 11 and 4.

## Criterion 1, both halves, stated plainly

**Criterion 1 is NOT met, and now both of its halves have been measured rather than argued.**

- **The memory half MISSES its declared target: 10 of 40 against ≥ 34**, on the one instrument in this
  line that cannot move between identical runs. It is not a near miss and it is not a noise question:
  the reading is deterministic.
- **The reply half has been measured to its limit**: 43/232 acceptable, failures diffuse across two
  classifications, the 64-token cap demonstrably not the constraint, **62.1 % of rows with no
  judge-free correct answer at all**, and a deterministic sub-reading over 88 rows that scores **2**.
  The reply panel's reading is a judge whose line the text does not show, with a quarter of its
  acceptances being one memorised greeting.

So the honest sentence is: **criterion 1's memory half is measured and missed; its reply half cannot
carry the criterion as instrumented and has been restated as a deterministic sub-reading.** There is no
version of this in which criterion 1 is met, and 43/232 is unchanged.


## The miss, broken down so the next reader can decide what it means

**A 24-row shortfall has two very different explanations and this run does not settle which: the model
cannot do this task, or the target was set where no artifact of this size could reach it.** What
follows is the breakdown that lets someone else decide; no interpretation is offered here.

Per-row `check_pass` was recomputed independently with the conformance port and agrees with the
sealed report: **10 of 40**.

### By mechanism

| mechanism (row shape) | rows | `check_pass` | rate |
|---|---:|---:|---:|
| `exact` with distractor keys, word value | 23 | **9** | 39.1 % |
| `exact` with distractor keys, number value | 11 | **1** | 9.1 % |
| `exact` with no distractor keys, word value | 4 | **0** | 0 % |
| `exact` with no distractor keys, number value | 2 | **0** | 0 % |

**The failures cluster.** Every one of the 10 passes is on a distractor-key row; **all six rows without
distractor keys fail**, and **numeric values are four times harder than word values** (1 of 11 against
9 of 23). The 10 passing rows: `mem-001, -003, -009, -015, -019, -023, -031, -033, -039, -040`.

### By failure mode, over the 30 failing rows

| what the reply does | rows |
|---|---:|
| **names neither the expected value nor the planted distractor** | **22** |
| **names the planted distractor** (the mode the key column exists to catch) | **8** |
| empty reply | 0 |
| other | 0 |

Examples, verbatim: `mem-002` answers "Your kitten is Leon." where **Leon is the planted distractor**
and Barbara is expected; `mem-006` answers "I'm a helpful assistant that runs on your computer.";
`mem-008` answers "Your sister's locker number is 1."; `mem-010` answers "The garage code is the code
word."

So the distractor-key column is doing real work — **8 of 30 failures are the wrong-value-named mode it
was built to catch** — and the remaining 22 are wrong values that are not even the planted distractor.

### Is any row passed by nothing at all?

**No.** The `expected_value` control passes **40 of 40**: every memory row is passed by its own bare
expected spelling, so no row is unpassable by construction and the check is not the obstacle. The
model's 10 of 40 is a model reading, not an artefact of the panel.

### The declared command cannot run as written

`grade-replies` defaults `grader=` to `qwen2.5:1.5b`, which is not installed, so the declared
invocation 404s in 0.063 s with no report root — a crash, not a sample. The v5 record's declared
command has been corrected, and it is the same class of trap as `requests=` not `panel=`.

## Vindication worth one line: a crash was not a miss

The first attempt produced no report root, so it was a crash and re-running was legitimate; had it
produced a report with a bad number, that run would have been the sample and re-running would have been
exactly what the pre-registration forbids. The two were kept apart, and the pre-registration was not
bent to produce a second draw.

## Cost

Laptop CPU only. Generation 2m04s, grading 9m46s, one structural `check` (seconds). **$0.00, no pod.**
Two mechanical retries: the grader-default crash (0.063 s) and one wrong `model=` path probe.

## Next

1. **Fix the v5 binding swap**: add `conversational-v5-swaps.tsv` to `EMBEDDED_SWAPS` and assert
   `checked_rows == 40` in a v5 control test, so the control the freeze claimed is actually running.
   This is a small, bounded correction to the instrument, not a re-run of the acceptance.
2. **The measured target is now explicit and unmet: 10 of 40 memory rows at `check_pass`**, with the
   failing rows named in the sealed report — a deterministic, diagnosable target for the first time in
   this line. Any candidate that trains against criterion 1 should be scored on this run's terms, at
   the same declared cap and with the same controls, and the 24-row gap to the bar is what it has to
   close.
3. **Do not re-run this panel for a better sample.** It is development evidence now, exactly as v4
   became; a fresh acceptance needs a fresh sealed panel.

Evidence: the report root `v5-declared-run/` (sealed, `attempt.json` + `manifest.json` + `report.json`),
the generation root `v5-acceptance-29m/`, and the bundle
`icloud:UOR-R4/results/deepseek/v5-declared-acceptance-2026-10-09.tar`.
