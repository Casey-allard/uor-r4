# Rank-margin objective (hard hinge over declared rivals)

**Milestone:** M1 #2029 (the training objective the compliant trunk runs). **Lab:** Kimi (objective layer).
Registered for rung 0 by the sheet this brief points at. Implements the objective column of the
[D23 replacement law](../labs/d23-learning-law-design-2026-10-12/README.md).

## The idea (first principles)

The objective decides what the model is asked to do, and it is on the executed path in both directions. The
incumbent objective is cross-entropy over a normalized-exponential distribution: its forward pass needs a
log-sum-exp, its backward pass needs the exponential itself, and its *score* is in nats — a unit that only
exists because of that normalization. [D23](../integration/DECISIONS.md#d23---no-softmax-anywhere-in-the-active-project)
and [D25](../integration/DECISIONS.md#d25---geometric-operator-phase-and-current-lab-direction) ban all of
that: no softmax, softplus, sigmoid or tanh anywhere in an executed path, in training as well as serving.

The replacement here is a **hard rank hinge over declared rivals**, in raw-score units:

    L = mean_k max(0, 1 − z_target + z_rival_k)

over the top-`k` non-target raw scores `z` of each row (`k` = `rivals`, declared in advance). Every term is a
comparison and a subtraction; there is no exponential, no normalization and no unit conversion in either
direction. A row whose target already leads every declared rival by 1.0 contributes exactly zero gradient.
`1` is a *fixed score scale*, not a probability: `z` are the unnormalized head outputs, so the bar is
"the target's raw score leads its strongest rivals by the declared margin", and any comparison across
configurations must use the same scale and the same rival count.

## What it replaces and why that matters

| Replaced | Why it had to go | What the hinge uses instead |
| --- | --- | --- |
| softmax cross-entropy (log-sum-exp forward, `exp` backward) | D23 bans the exponential; D25 bans dense matmul | a hinge over raw scores, no exponential in either direction |
| normalized-exponential NLL / BPB as the quality unit | the unit only exists through the banned normalization; nats and bits are different units | raw-score measures: top-1, target rank/recall@K, the hinge itself, the unviolated-target rate |

The consequence for evaluation is as important as the training change: a compliant rung cannot report NLL or
BPB at all. Its reports carry raw-score measures with their exact panel, unit and tie rule
([operator card § Whole-path admission](../labs/operator-phase-2026-10-12/README.md#whole-path-admission-and-quality-measures)).

## Current native implementation

The selected `geometric_stack_native.rs` keeps `Objective::MarginHinge { rivals }` metadata and dispatches native learning through `NativeVectorWriter::margin_rows`/`margin`: CUDA raw negative mean-L1 token scores at fixed scale1, with an independently labelled address margin, coefficient1 each. Omitted targets receive explicit score credit without insertion into the candidate set; exact ties use smallest token IDs. The training objective is token plus address margin; the rung0 quality/anti-vacuity bars read token margin alone. Candidate-selection credit remains the declared geometric surrogate, not a derivative of hard top-K.

The registered CLI is `examples/geometric-stack-native.rs`, reusing its native rung0 evaluator. Its primary reference counts response-masked **TRAIN** targets on CUDA; the masked **DEV** oracle is diagnostic only. TRAIN/mask/tokenizer content identities and policy bind lineage `/2` and artifacts; mismatched model config or optimizer settings refuse before resumed evaluation/update. NLL/BPB, host count arithmetic, sampling and historical raw-logit/CPU margin implementations are excluded from this path. Independent sheet Amendment6 FIT retains unchanged bars and paired seeds; corrected source `1f1547bfd` passes CUDA compilation and all 31 finite controls on RTX4090, pending protected delivery. Failed setup/clipping attempts remain preserved; no optimizer update, corpus evaluation, training or CPU model/reference ran. Training holds remain HELD. [Boundary record](../labs/d29-vendor-boundary-2026-10-13/README.md).

## Historical implementation locations (inactive)

- `Objective::MarginHinge { rivals }` — `crates/uor-r4-training/src/geometric_stack.rs:1374` (the enum; `Ce`
  stays the serialization default so existing artifacts reproduce), dispatched into the fused op at
  `:10933` (and its weighted twin at `:10978`); the config validation that refuses a pointer is at `:2773`.
- The row and its gradient: `margin_row` (`:18574`) and `margin_row_grad` (`:18598`), over the raw logits,
  with the top-`rivals` order taken from the CPU's stable sort so CPU and device agree on ties.
- CUDA f32 kernels `margin_rank_rows` / `margin_rank_grad`
  (`crates/uor-r4-training/src/cuda_stack_kernels.rs:738`/`:744`, one warp per row) with launchers in
  `geometric_stack/cuda_ops.rs`; Metal is refused (the established `no_device_kernel` pattern). A bf16
  margin kernel is out of scope for rung 0, which trains f32.
- Trainer flag `objective=hinge|hinge8` (`examples/geometric-stack.rs`), carried into the report and the
  resume lineage through the config.
- **A pointer head is refused with this objective today**
  (`geometric_stack.rs:2779`, "a margin objective has no pointer-mixture form yet"): the pointer mixes a copy
  distribution into the prediction, and its own objective is the next rung (0b).

## What has been tried, honestly

- The D23 census found the executed training paths' objectives were all softmax cross-entropy — every
  existing in-tree objective reading is therefore non-compliant evidence, and this objective is the first
  lawful one.
- The implementation, its CUDA f32 kernel and its focused tests landed (#2255 and the OP-family PRs:
  hand computation on a fixed row, finite-difference gradients, bit-exactness of the `Ce` path it did not
  touch, and a serving-compliance witness that a non-pointer model's `next_scores` are its raw last logits).
- **No training run has executed this objective.** Rung 0's own pre-registration is the sheet named below;
  under the D25 freeze and the owner's holding rules, no model compute runs until a `TEST FITNESS: FIT`
  verdict and an explicit hold release. The CUDA kernel has been parity-tested against the CPU reference
  (device receipt in the operator record); it has never trained a model.
- No claim is made that a hinge reaches cross-entropy-level language quality. That is exactly the question
  rung 0 measures at its own scope, with margin curves, not with incomparable NLLs.

## What success looks like

An arm works iff, on **both** seeds, it clears the rung-0 sheet's two decisive **rank** bars (Amendment 2,
2026-10-13): raw-score top-1 and recall@8 over the head's declared kept list (`top_token_ids(8)`) each beat
a **response-masked TRAIN token-frequency count predictor**, scored on the same dev targets, by **≥ 5 percentage points**. The DEV count oracle is diagnostic only (sheet Amendments5–6). The
earlier absolute floors are kept alongside as anti-vacuity guards (final dev token-margin hinge loss
< 0.5 raw-score units; unviolated-target rate ≥ max(5× the same-seed untrained constant, 1%), where the
1% floor is the operative device because the untrained constant is ≈ 0), and the hinge loss, unviolated
rate, post-warmup monotone curve and mean table L1 norm are reported diagnostics. Every bar and diagnostic
reads the objective's **token_margin** term alone, never token+address. The sheet fixes the exact numbers
before compute; the two arms are the same recipe at 1 and 8 declared rivals.

## What failure looks like

- **One seed passing, or a positive but sub-bar margin on the rank measures**, is *iterate*: the configuration (objective scale, dose,
  initialization) is diagnosed, and one revised run needs a new sheet.
- **Divergence, a non-finite loss, a kernel defect or an observed host fallback** is a **tooling blocker to
  fix**, not a mechanism negative (D22 §1–§2).
- A **mechanism stop** (abandoning the rank-margin family) is an owner decision; no run and no sheet may
  take it.

## A fair test

`docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md`: rung 0 on the compliant trunk — six recurrence
blocks, no read and no pointer, `objective=hinge`, arms at 1 and 8 rivals, two seeds each, one Vast pod,
GPU-only. The sheet names the four dependencies that must merge first (the SpiralCore/FBS native projection,
the OP3a product-key head, the recurrence squash replacements, and the lawful device-qualification gate) and
the remaining unlawful sites it must not run past. Until those land and the owner releases the training hold,
the run does not execute.

## Open design questions

- **Rival selection and ties.** Top-`k` by raw score with a stable-sort tie rule is what the implementation
  does; whether a different declared rival set (all, random, hardest-by-margin) is a fairer objective test is
  open, but each is a different pre-registration.
- **Score scale.** `1.0` is fixed for rung 0. A scale that has to be tuned per configuration would make
  cross-run comparisons meaningless, so any change of scale is a new registration, not a hyper-parameter.
- **Language quality without a likelihood.** Raw-score top-1 and margin curves measure what a hinge trains;
  whether that is *sufficient* for useful prose is the open question the milestone carries, not this brief.
- **Interface with a read and a pointer.** Rung 0b adds the rank read's CUDA kernel and a score-gradient
  stop; the pointer's own objective and the address credit are named there, not here.
- **Device coverage.** f32 only for now; a bf16 margin kernel is a separate source item.
