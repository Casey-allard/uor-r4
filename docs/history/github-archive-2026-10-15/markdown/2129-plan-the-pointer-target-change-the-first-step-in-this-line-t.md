# #2129 — Plan: the pointer target change, the first step in this line that costs money (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T02:05:09Z
- **Updated:** 2026-10-10T03:23:40Z
- **Closed:** 2026-10-10T02:06:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2129
- **Merged:** 2026-10-10T02:06:32Z
- **Merge commit:** 3c8cb057686b4f3523580de6a635f8fd1bbb4dee
- **Base / Head:** main / UOR-Foundation:pointer-target-change-plan
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Planning only — $0, no pod, no training, CPU only, no knob
Record: [`docs/labs/pointer-target-change-plan-2026-10-09/README.md`](docs/labs/pointer-target-change-plan-2026-10-09/README.md). This is the document for the owner.

## Background: the ledger, at its own strengths
The plan rests on **five dead readings and one measured statement**, not a hunch that survived because nobody tested it:

| reading | status |
|---|---|
| Token count explains the cluster | **REFUTED** |
| Minimal pairs / digit order | **REFUTED** |
| Value addressability | **REFUTED** — **this stopped a full ladder rebuild** |
| The pointer's single-source shape is the binding constraint | **REFUTED in its simple form, replaced** (`TopK(2)`+gate 0.9 moved coverage 0 → 3 of 13, failed its bar) |
| "The pointer's behaviour is history-insensitive" | **CORRECTED** — the invariance is the shared sentence **frame's** |
| **The pointer attends the sentence frame and never the varying slot** | **MEASURED on 13 rows** |

Ten pieces, all CPU-only, all $0; the object under investigation moved from the tokenizer to the memory path to the pointer to the attention's positional target.

## 1. What the target change is — expressible in the EXISTING training path
No new signal is needed; **the labelling is the new part**:
- **`ReadSupervisionGroup { bound, competing }`** — *"`bound` holds the window positions of **the value the answer must name**; `competing` the positions of the other stated values that answer must not name"*;
- **`gate_supervised_loss`**: `total = mixture + weight · (gate_bce + pointer_nll)`, with **`gate_bce = BCE(g_t, [target held by a source 0..=t])`** and **`pointer_nll = −log p_copy(target)`**.

**So: label the value's digit run as `bound` (both positions for a two-token value), the distractor's as `competing`, and train with `gate_supervised_loss`** — the run-level marking the current objective has never required.

## 2. The smallest decisive version — CPU-only, and it can kill the plan for free
**Build the labels and evaluate `pointer_nll`/`gate_bce` on the EXISTING weights, reporting `p_copy` at the value's two digit positions before any update.** If `p_copy` is already high there, the head is not the problem and the training piece is unnecessary. Only if it is low does a **single-rung, single-value-type labelled run** follow. The last two cheap steps on this line (the probe that killed Option A; the knob that failed its own bar) both changed the plan more than an expensive run would have.

## 3. What it invalidates
A retrained artifact is a new artifact: **v5's 10 of 40 does NOT survive as a comparison**, and **a fresh sealed panel is required**. **Confirmed again: the tokenizer is untouched**, so **every result citing `d36d3e87…` stays valid** — 43/232, the cap-96 comparison, the v5 structural validation. That is why this is a **one-rung question, not a ladder question**.

## 4. Success test, pre-declared
Primary: same 13 rows, same trace, **both digits selected as a run on 13 of 13** (currently 0 of 13), `mem-040` still emitting `84`. Secondary: the fresh panel's frozen target, declared before the draw. **The panel must be built against the obvious failure mode — memorising these 13 rows:** labels disjoint from the panel by value, key and frame with the disjointness **asserted mechanically**; held-out magnitude and leading-zero forms; matched rows sharing a frame with varied values **and the converse**; and **a control arm that never sees the label**.

## 5. Falsification, in advance
The frame-attention reading is wrong if **both digits are attended and the value still is not delivered** (back to the emitter); or **`p_copy` is already high before any update**; or **a gain on the trace is lost on the fresh panel** (memorisation); or **word rows move as much as numeric rows**.

## 6. Cost, and the refusal
**CPU, no pod:** label generator + pre-training `p_copy` read (**minutes, and it can kill the plan for free**); the 13-row trace read-out (**12 s, already built**); drawing and sealing a fresh panel (minutes).

**GPU via `uor-pod`** within the caps (≤ 4 pods, ≤ $8/h, EU-RO-1 pinned if the volume is needed): **a timed calibration run comes first, and NO THROUGHPUT NUMBER IS GIVEN BECAUSE NONE IS CITABLE** — the projection stays `calibration_throughput × corpus_tokens × epochs` until it exists. **If even the calibration needs a pod, the smallest that works is one 5090-class pod on the ladder's existing volume, leased for the calibration only and released immediately. No pod is brought up by this plan**, and the calibration should not run before §2 step 1 reports.

## 7. The limit that still bounds everything
**No memory reader on this artifact.** A pointer/attention fix is a pointer/attention fix; it does **not** transfer to the addressed-memory path, where **the two-token question has never been measured**. That is a **separate piece** using `memory_read_diagnostic`'s `predicted_token` and `target_routes`.

## Delivery
Plan record; newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; bundle on cloud-store: `pointer-target-change-plan-2026-10-09.tar`, 19,968 bytes, md5 `6603cee6e431bb83b0b3fea1827c8c69`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch; scratch inside the worktree.

**Next: run §2 step 1 — CPU only, no pod, no training.** If `p_copy` is already high at the value's digit positions, the training piece is unnecessary and the emitter is the target.

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged; v5 not re-run. References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

