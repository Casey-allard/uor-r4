# #2124 — Plan: a reader-side fix for the two-token run, CPU-only first (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T01:37:45Z
- **Updated:** 2026-10-10T04:05:12Z
- **Closed:** 2026-10-10T01:42:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2124
- **Merged:** 2026-10-10T01:42:03Z
- **Merge commit:** 1a9438ec24d585314c859e35cf80bc9f9b1217bf
- **Base / Head:** main / UOR-Foundation:reader-side-fix-plan
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Planning only — $0, no pod, no training, CPU only, no v5 re-run
Record: [`docs/labs/reader-side-fix-plan-2026-10-09/README.md`](docs/labs/reader-side-fix-plan-2026-10-09/README.md).

## 1. Why the pointer never selects both tokens — the code path
**It is a single-source pointer by construction**, documented in `geometric_stack.rs`: the attention is a softmax over the sources `PointerConfig::select` keeps, "**`TopK(1)` being the single-source pointer**"; `PointerSelection.source` is "**the lowest source index whose attention is the largest**" — **one index per step**, and `CopyTraceStep` records exactly that one per step; and `p_copy(v|t) = Σ_j a_tj [x_j = v]` mixes over **single attended positions**.

So a two-token value needs **two consecutive steps landing on consecutive positions**, and **nothing rejects the second — the head is shaped to put its mass on one source per step.**

**The same expression explains the EMITTER rows**: `p(v|t) = (1−g_t)·softmax(z_t)[v] + g_t·p_copy(v|t)`, with **`p_copy` exactly 0 when no kept source holds the target and no probability floor** — so a selected digit need not win if the gate is low, which is how `19` (`1`) is emitted while the pointer sits on `24` (`6`).

## 2. The smallest change — and it is NOT a training change
Selection and gate are **serving-time knobs on unchanged weights**: *"Training may stay soft (`select: None`); `StackModel::set_pointer_select` applies a selection to the same weights afterwards."* So:
- **(a)** widen the kept source set — `PointerSelect::TopK(k)`, `k ≥ 2` — so a run is reachable within one step's mixture;
- **(b)** raise the copy gate so `p_copy` can win at the second digit instead of being outvoted by `softmax(z_t)`.

**CPU-only, testable tonight.** Only if that fails: **(c)** a training-path change making the head's target a run — **GPU**, and it needs the owner.

## 3. The success test, pre-declared
Same trace on the same 13 rows: **all 13 show BOTH stored digits selected** (currently 0 of 13); **`mem-040` still emits `84`**; **`mem-008`/`mem-024` no longer emit `19`**; instrument conditions hold again (byte-for-byte reproduction re-established, `digit_ids` exactly two tokens or VOID).

**Falsification in advance:** if both digits are selected and the reply **still** lacks the value, **the fix is in the wrong place and the emitter is the real problem after all** — go to §4, do not widen the same change.

## 4. The 2 EMITTER rows are a separate piece
They **selected one stored digit and emitted `19`** — a **selected-then-dropped** failure, mechanically distinct from the 10 READER rows where nothing was selected. The reader fix would touch them only incidentally, so they get **their own pass condition** and are reported separately even if they move: their question is **gate/mixture**, not selection coverage.

## 5. Cost, with the split
Inference-path change + test: **CPU only, tonight**. Training-path: **GPU via `uor-pod`** under the caps — and **no throughput number is given because none is citable**; a timed calibration run comes first, the same refusal the last plan made. A retrain produces a new artifact, so **the v5 comparison does not survive as a comparison** (v5's 10 of 40 reads `chat-29m-B-lr5e-4`; a retrained artifact needs a fresh sealed panel). **The tokenizer is untouched either way**, so every result citing `d36d3e87…` stays valid.

## 6. The limit that bounds everything
**This artifact has NO MEMORY READER.** A **pointer fix is a pointer fix**; it says nothing about the addressed-memory path, whose `/3`–`/5` readers belong to different artifacts. What would have to be measured separately — **and never has been** — is whether a memory-reader artifact selects a two-token value as one unit, using `memory_read_diagnostic`'s `predicted_token` and `target_routes` rather than the pointer trace.

## Ledger
Token count REFUTED · minimal pairs/digit order REFUTED · value addressability REFUTED · **the learned read/emit path SPLIT (READER 10, EMITTER 2, PARTIAL 1)**, reader side targeted first.

## Delivery
Plan record; newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; bundle on cloud-store: `reader-side-fix-plan-2026-10-09.tar`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch.

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged; v5 not re-run. References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

