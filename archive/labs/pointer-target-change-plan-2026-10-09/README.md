# Plan: the pointer target change (the first step in this line that costs money)

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek. 2026-10-09. **PLANNING ONLY: $0, no pod,
no training, CPU only, no knob.** No v5 re-run. This document is the deliverable and it goes to the
owner.

## 0. Background: the ledger, at its own strengths

The plan rests on **five dead readings and one measured statement**, not on a hunch that survived
because nobody tested it. Ten pieces, all CPU-only, all $0, and the object under investigation moved
from the tokenizer to the memory path to the pointer to the attention's positional target:

| reading | status | what killed it |
|---|---|---|
| Token count explains the cluster | **REFUTED** | numbers tokenize in 2 tokens against 3.74 for words |
| Minimal pairs / digit order | **REFUTED** | `mem-040`'s distractor `74` is a transposition of its `84` and it passes; 0 of 12 numeric failures name the distractor |
| Value addressability | **REFUTED** | the codec recovers a two-token run as one ordered value with an interval — **this stopped a full ladder rebuild** |
| The pointer's single-source shape is the binding constraint | **REFUTED in its simple form, replaced** | `TopK(2)` + gate floor 0.9 moved both-digit coverage 0 → 3 of 13 and failed its pre-declared bar of 13 of 13 |
| "The pointer's behaviour is history-insensitive" | **CORRECTED** | the invariance is the shared sentence **frame's**, not history's |
| **The pointer attends the sentence frame and never the varying slot** | **MEASURED on 13 rows** | digits selected: **0 of 13 for the second, 3 of 13 for the first, never both**; highest attention in both tables is the bare-space token; the two compared rows differ in exactly one respect, the value |

**The limit that still bounds everything (§7): this artifact has NO MEMORY READER.**

## 1. What the target change actually is — and it is expressible in the EXISTING training path

**The supervision already exists; what is missing is that the panel's rows were never labelled so that
the value's own positions are the target.** Two public items carry it, in
`crates/uor-r4-training/src/geometric_stack.rs`:

- **`ReadSupervisionGroup { batch, bound, competing }`** — documented as: *"`bound` holds the window
  positions of **the value the answer must name**; `competing` the positions of the other stated values
  that answer must not name"*, both sorted, unique and disjoint; and `ReadSupervisionTarget.query` is
  *"the input position of a batch item whose next token is a token of the group's bound value."*
- **`StackModel::gate_supervised_loss` / `GateSupervisedLoss`** — *"a pointer model's response loss with
  copy-gate supervision: `total = mixture + weight * (gate_bce + pointer_nll)`"*, where
  **`gate_bce = BCE(g_t, [target held by a source 0..=t])`** supervises *whether the copy gate should
  open*, and **`pointer_nll = -log p_copy(target)`** on rows whose target a source holds supervises
  **the pointer's attention onto that target**.

**So the concrete change is: label the value's digit run as `bound`, the planted distractor's positions
as `competing`, and train with `gate_supervised_loss`.** For a two-token value the group's `bound` holds
**both digit positions** — a run-level marking, which is exactly the supervision the measured failure
lacks: the head currently opens the gate over the frame's tokens, and nothing in the objective has ever
required it to place mass on the **slot that varies**.

**No new signal is needed; the labelling is the new part.** If the owner wants it named as a code change:
a supervision-label builder that emits `ReadSupervisionGroup`s with the value's full run in `bound`, and
the training loop calling `gate_supervised_loss` instead of the unsupervised mixture loss.

## 2. The smallest version that could still be decisive

**Do not start with a retrain of the rung.** In the order the evidence supports:

1. **Build the label generator and score the current artifact against it — CPU only, no training.**
   `pointer_nll` and `gate_bce` are **computable on the existing weights**: run the 13 rows with
   `gate_supervised_loss`'s parts evaluated on the labelled targets and report `p_copy` at the value's
   two digit positions **before any update**. If `p_copy` is already high at those positions, the head
   is not the problem and the whole training piece is unnecessary — **that is the cheap step that can
   kill this plan, and it costs nothing.**
2. **Only if step 1 shows the head cannot place `p_copy` there**, train **one rung, one value type,
   with the label** — no new architecture, no vocabulary change, same 29M configuration.
3. **Read out with the same 13-row trace**, which is already built and takes 12 seconds.

**Step 1 is the recommendation**: it is CPU-only, it is measurable today, and the last two cheap steps on
this line (the probe that killed Option A, and the knob that failed its own bar) both changed the plan
more than an expensive run would have.

## 3. What it invalidates

A retrained artifact **is a new artifact**:

- **v5's 10 of 40 does NOT survive as a comparison** — it is a reading of `chat-29m-B-lr5e-4`. **A fresh
  sealed panel is required**, drawn and frozen with the v5 discipline.
- **Every rung scored against v5's panel or the reply panel stays valid as a result about itself** and
  **not comparable** to the retrained artifact as if the instrument were unchanged.
- **Confirmed from the last plan and unchanged: the tokenizer is untouched**, so **every result citing
  `d36d3e87…` stays valid** — the reply panel's 43/232, the cap-96 comparison, the v5 structural
  validation. This is the one thing a retrain does not cost, and it is why this is a one-rung question
  rather than a ladder question.

## 4. The success test, pre-declared before any run

**Primary, on the existing trace:** the same 13 rows, same trace, **both stored digits selected as a run
on 13 of 13** (currently 0 of 13), with `mem-040` still emitting `84`.

**Secondary, on a fresh sealed panel** (which must exist before any candidate is scored): the panel's own
frozen target, declared before it is drawn, in the v5 shape — deterministic checks with per-row
provenance, controls verified before sealing, `check_panel` run, and **the binding swaps actually
embedded and asserted non-vacuous**, since v5's control was vacuous for its whole life.

**The panel must be built against the obvious failure mode — memorising these 13 rows — and that is a
design requirement, not a hope:**

- **rows the training labels never touched**: the panel's values and frames must be **disjoint from the
  training set** by value, by key and by frame, with the disjointness asserted mechanically;
- **held-out value types**: at least one magnitude and one leading-zero form that appear in no training
  label;
- **the frame held but the value varied**: matched rows sharing a frame with different values, so
  frame-attendance alone cannot pass;
- **the frame varied but the value held**: the converse, so a value-attending head is distinguishable
  from a position-attending one;
- **a control arm that never sees the label**, scored on the same panel, so any gain is attributable to
  the target change rather than to the extra optimisation.

## 5. The falsification, stated in advance

**The frame-attention reading is the wrong explanation if:**

1. **both digits are attended and the value is still not delivered** — that puts it back on the
   **emitter**, and the 2-row emitter piece becomes the main line;
2. **or `p_copy` at the value's digit positions is already high before any update** (§2 step 1) — then
   the head already places mass there and the measured failure is downstream of attention;
3. **or a retrained artifact gains on the trace and loses on the fresh panel** — memorisation, and the
   target change is not the cause;
4. **or word rows move as much as numeric rows** — the change is not value-type-specific and the
   two-token run was not the constraint.

**Each is a result and each is cheaper to learn than a completed training programme.**

## 6. Cost, and the refusal

**CPU, this laptop — no pod:**
- the label generator and the **pre-training `p_copy`/`gate_bce` read on the existing weights** (§2
  step 1): **CPU only, minutes**, and it can kill the plan for free;
- the 13-row trace read-out: **CPU only, 12 seconds**, already built;
- drawing, sealing and validating a fresh panel: **CPU only, minutes** (v5's draw is byte-reproducible
  from a seed).

**GPU, via `uor-pod` — the caps are ≤ 4 running pods and ≤ $8/h, EU-RO-1 pinned if the network volume
is needed:**
- **a timed calibration run comes first.** **I will not give a throughput number I cannot cite**, and
  none is citable: the repo records no ladder throughput at a usable level. The projection stays a
  formula — `calibration_throughput × corpus_tokens × epochs` at the pod rate — until that run exists.
- **If even the calibration needs a pod, the smallest that works is one 5090-class pod on the ladder's
  existing volume**, leased for the calibration only, and released immediately; the run is short, so its
  cost is dominated by the minimum lease, not by throughput.
- **No pod is brought up by this plan.** The calibration is a separate, owner-authorised step, and it
  should not be run until §2 step 1 has reported, because step 1 may remove the need for it entirely.

## 7. The limit that still bounds everything

**This artifact has NO MEMORY READER — only a copy pointer.** A pointer/attention fix is a
**pointer/attention fix**; it does **not** transfer to the addressed-memory path, whose `/3`–`/5` readers
belong to different artifacts, and where **the two-token question has never been measured at all**. If
the owner wants the memory path investigated, **that is a separate piece**, using
`memory_read_diagnostic`'s `predicted_token` and `target_routes` — and it would be the first measurement
of whether a memory reader selects a two-token value as one unit.

**Criterion 1 remains NOT MET on both halves and 43/232 is unchanged.** v5 was not re-run.

## Next

**Run §2 step 1 — CPU only, no pod, no training**: build the `ReadSupervisionGroup` labels for the 13
rows and evaluate `pointer_nll` and `gate_bce` on the **existing weights**, reporting `p_copy` at the
value's two digit positions before any update. **If `p_copy` is already high there, the training piece is
unnecessary and the emitter is the target.** Only if it is low does the single-rung labelled training run
become the next step — with a fresh sealed panel drawn first and a timed calibration run before any cost
estimate.

**Executed (2026-10-09) by the [copy-mass read](../pcopy-mass-read-2026-10-09/README.md), and the rule
needs its position named.** `p_copy` is read on the public scored path (`score_targets` →
`PointerRowStats::copy_mass`), with the pre-registered trace gate passing 166 of 166 steps on the 13
rows. Measured: at the value's **first** occurrence the rule's condition is met (the digit's id carries
0.32–1.00 with `hit` true on 11 of 13 rows), but at the reply step that produces the answer it is **not**
met on 10 of 13 rows (value share ≤ 0.20 against a frame share of 0.34–1.00), and at the value's *last*
occurrence the same rows collapse. So **for the 10 READER rows the read is still the constraint; for the
3 rows whose argmax already lands on a stored digit (`mem-008`, `mem-024`, `mem-040`) the emitter is.**
Neither branch is a "nothing to train" verdict on its own, and the plan is not shelved under that third
reason.
