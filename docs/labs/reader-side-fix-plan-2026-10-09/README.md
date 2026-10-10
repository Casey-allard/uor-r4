# Plan: a reader-side fix for the two-token run (reader 10, emitter 2)

References #2029. Lab: DeepSeek. 2026-10-09. **PLANNING ONLY: $0, no pod, no training, CPU only.** No
v5 re-run. This document is the deliverable.

## What it plans against

From [#2123](../numeric-pointer-split-2026-10-09/README.md), on an instrument that reproduces the
sealed replies **13 of 13 byte for byte** with its two-token assertion holding:

- **READER 10, EMITTER 2, PARTIAL 1** on the 13 numeric rows.
- **0 of 13 rows had BOTH stored digits selected — including `mem-040`, which nevertheless emits the
  value correctly.** The value is a two-token run and the pointer never selects the whole run.
- The 2 EMITTER rows (`mem-008` stored `68`, `mem-024` stored `54`) each selected **one** stored digit
  and emitted `19` (`1`) with `matches_source: false`.

**The limit that bounds everything below: this artifact has NO MEMORY READER — only a copy pointer.**
A pointer fix is a pointer fix. Nothing here transfers to an addressed-memory path without its own
measurement (see §6).

## 1. Why the pointer never selects both tokens — the code path

**The pointer is a single-source pointer by construction**, and this is documented rather than
inferred (`crates/uor-r4-training/src/geometric_stack.rs`):

- the attention `a_t` is the softmax over the sources the pointer's own `PointerConfig::select` keeps —
  "**`TopK(1)` being the single-source pointer**";
- `PointerSelection.source` is "**the lowest source index whose attention is the largest**" — **one
  index per step**; `CopyTraceStep` records exactly that one `source` per step;
- the copy distribution is `p_copy(v | t) = sum_j a_tj [x_j = v]` — a mixture over **single attended
  positions**.

So a two-token value needs **two consecutive steps whose selections land on consecutive window
positions**, and **nothing rejects the second selection — the head is simply shaped to place its mass
on one source per step.** The observed `0 of 13` is that shape showing up on a two-token value.

**And the same expression explains the EMITTER rows**: the emitted distribution is
`p(v | t) = (1 - g_t) softmax(z_t)[v] + g_t p_copy(v | t)`, with `p_copy` **exactly 0 when no kept
source holds the target and no probability floor**. A selected digit therefore does not have to win:
if the gate `g_t` is low, the ordinary distribution decides, which is how `19` (`1`) is emitted while
the pointer sits on `24` (`6`).

## 2. The smallest change that makes a run selectable as a unit

**Two candidate changes, and the code says which is cheap.**

**(a) INFERENCE-PATH — CPU-only, testable today, no training.** The pointer's selection and gate are
**serving-time knobs on unchanged weights**: "Training may stay soft (`select: None`);
`StackModel::set_pointer_select` applies a selection to the same weights afterwards." So:
- **widen the kept source set** (`PointerSelect::TopK(k)`, `k >= 2`) so a two-token run is *reachable*
  within one step's mixture rather than requiring two lucky steps;
- **raise the copy gate** so `p_copy` can win at the second digit instead of being outvoted by
  `softmax(z_t)` — the mechanism that produced `19` on the EMITTER rows.

This is the smallest change and it is **not a training change**. It is measurable with the existing
trace in the same 12-second run.

**(b) TRAINING-PATH — GPU, only if (a) fails.** Make the head's target a **run** rather than a
position: train with attention targets that reward consecutive sources across a multi-token value.
That changes weights, needs a retrain, and is justified only if (a) cannot move the `0 of 13`.

**Name the code path for (a):** `set_pointer_select` / `PointerSelect::TopK`, `set_pointer_identity`,
and the gate `g_t = sigmoid(w_g . h_t + b_g)` in the mixture — all applied after loading, with the
weights untouched.

## 3. The success test, pre-declared

**The same trace on the same 13 rows, requiring both stored digits selected as a run.**

**Pre-declared pass condition, stated before the change is built:**
1. **All 13 rows show BOTH stored digits selected** (currently 0 of 13).
2. **`mem-040` still emits the stored value `84`** — the change must not break the row that works.
3. **`mem-008` and `mem-024` no longer emit `19`** (currently both do).
4. The instrument's own two conditions hold again: **byte-for-byte reproduction of the sealed replies
   is re-established for whatever the change produces**, and **`digit_ids` are exactly two tokens** or
   the run is VOID.

**Falsification, stated in advance:** **if both digits are selected as a run and the reply STILL lacks
the value, the fix is in the wrong place and the emitter is the real problem after all.** That outcome
sends the work to §4 rather than to a bigger version of the same change, and it must be reported as
that, not retried with more gate.

## 4. The two EMITTER rows get their own line

`mem-008` and `mem-024` **selected one stored digit and emitted `19` (`1`) with `matches_source:
false`** — a **selected-then-dropped** failure, mechanically distinct from the 10 READER rows where
nothing was selected at all.

**The reader fix would touch them only incidentally** (a gate change alters the mixture everywhere), so
**they are not folded in**: their own condition is that the gate must let `p_copy` win when the
pointer *is* on the right position, which is a **gate/mixture** question rather than a
**selection-coverage** question. **They get their own piece and their own pass condition**, and the
reader piece must report them separately even if they move.

## 5. Cost, with the split

| step | cost |
|---|---|
| (a) inference-path selection/gate change | **CPU only, tonight**: edit, rebuild (~2 min warm), one 12-second trace run |
| the same trace as the success test | **CPU only, seconds** |
| (b) training-path change, if (a) fails | **GPU via `uor-pod`** under the caps (≤ 4 pods, ≤ $8/h) |
| a timed calibration run before any estimate | **required, GPU** — **I will not give a throughput number I cannot cite.** Nothing in the repo records the ladder's training throughput at a citable level, so the estimate stays a formula until a short timed run exists |

**What a retrain would invalidate, if (b) is needed:** it produces a new artifact, so **the v5
comparison does not survive as a comparison** — v5's 10 of 40 is a reading of `chat-29m-B-lr5e-4`, and
a retrained artifact must be scored on a **fresh sealed panel**, exactly as v5 replaced v4. The
**tokenizer is untouched** by either path, so the ladder's other artifacts and every result citing
`d36d3e87…` stay valid and comparable.

**The instrument cost is zero either way**: the trace, the assertion, and the 13-row test are already
built and take seconds to re-run.

## 6. The limit that must survive: no memory reader

**`chat-29m-B-lr5e-4` carries no memory operator — only a copy pointer.** Therefore:
- **"10 of 40 on the v5 memory category" is 10 of 40 for a copy-pointer dialogue stack on requests
  named memory rows.** The qualifier stays on the v5 record and on current-state.
- **A pointer fix is a pointer fix.** It says nothing about the project's addressed-memory path, whose
  readers (`/3`, `/4`, `/5` schemas) are exercised by **different artifacts** and diagnosed by
  `memory_read_diagnostic`.
- **What would have to be measured separately:** whether a memory-reader artifact selects a two-token
  value as one unit — the same question, on the memory path, with `memory_read_diagnostic`'s
  `predicted_token` and `target_routes` instead of the pointer trace. **That measurement has never been
  taken**, and this plan does not assume its answer either way.

## 7. The ledger, kept as structure

| reading | status |
|---|---|
| Token count explains the cluster | **REFUTED** |
| Minimal pairs / digit order | **REFUTED** |
| Value addressability | **REFUTED** |
| The learned read/emit path | **SPLIT: READER 10, EMITTER 2, PARTIAL 1** — and this plan targets the reader side first |

Every step of this line so far has cost nothing, and the split is believable because the instrument
that produced it was made to fail loudly first: two void splits reported as void, a turn-grouping bug
found by reading the panel, and a tab-parsing bug caught from the output shape.

**Criterion 1 remains NOT MET on both halves and 43/232 is unchanged.** v5 was not re-run.

## Next

**Take the inference-path change in §2(a)**: widen the kept source set and raise the copy gate on
unchanged weights, rebuild, and re-run the same trace on the same 13 rows against the §3 pass
condition. **CPU only, $0, tonight.** If both digits are selected and the reply still lacks the value,
stop and go to §4 — the emitter — rather than widening the same change. **No training, no pod, and no
v5 re-run is authorised by this plan**; a training-path change goes to the owner with a timed
calibration run first.
