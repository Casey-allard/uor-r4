# Full-context integer execution bridge — implementation work card

Owner-authorized continuation of [D9](DECISIONS.md#d9--prevent-experiment-loops-and-preserve-the-context-contract),
under #973 / #820. Base `37b244cd5093a85fa23d5e93b466faa1d3cfe00f`.
This record precedes loaded model execution. No learning or admission change is
part of this milestone.

## Deliverable, cause and fixed conditions

Execute the accepted learned-code model through integer numerical operations,
including full causal attention, copy, recurrent updates and quaternion/ordinary
transport. The measured implementation gap is that existing packed codes still
execute in an F32 emulator. Replacing those numerical operations is independent
of further admission pruning.

Use the preserved `learned-rounding-20260925/fit-{quaternion,householder_pair}-rounding-3/packed-model`
parents. Keep their parameter codes/scales, tokenizer, source data and evaluator
v2. Context remains 256 for training provenance, evaluation and sessions; all
prior positions remain admitted, with 255 readable previous events at position
255. State width256 and read width64 are independent representation dimensions.
No recent64 fit, geometry sweep, new benchmark or fresh holdout is introduced.

## Numerical implementation

- Reuse the validated packed codec, exposing signed integer codes without an
  F32 parameter materialization. Preserve existing F32 loading through that same
  validation path. Bind model/scalar contracts and the offline table artifact.
- Execute low-bit affine maps by signed additions/shifts. Use checked binary
  products, long division and integer square roots for variable state arithmetic.
  RMS normalization retains fractional guard bits before the root; state and
  interface grids remain Q11/Q10/Q8/Q14/Q15 as in the parent.
- Use the exact quaternion center denominator2560 at the Q8 raw interface,
  preserving the raw=(-10,0,0,0) identity fallback. Use a declared Q32 constant
  for the ordinary arm's sqrt2 center. Retain signs and the original Householder
  factor2; quantized transport is approximately orthogonal, not exact group closure.
- Compile finite sigmoid/tanh/negative-exp tables once in offline Rust. Bind
  actual table hashes and generator contract; served accesses are integer reads.
  F64-generated entries need not equal the F32 emulator at rounding boundaries.
- Use Q48 normalized attention and output probabilities, an in-kernel positive
  uniform mixture, and assign the rounding residual to the largest entry with
  the first index winning ties. Exact probability totals do not imply exact
  agreement with the F32 reference. Q40 affine intermediates can introduce at
  most2^-41 rounding when admitted scale combinations are finer than Q40.

The numerical bridge uses dense parameter access and allocates. Shape/address
arithmetic, legacy metadata, tokenizer, hashing, reporting and the retained
floating seeded sampler are outside its declared numerical kernel. Integer
greedy source selection is exercised separately. Inspect compiled instructions
before making a machine-instruction claim; source syntax alone is insufficient.
No complete-process integer serving, D5 sparsity, energy or new language claim
follows merely from constructing this bridge.

## Necessary comparisons and decision

1. Compile the changed Rust path and run its focused arithmetic/codec/interface
   checks. Reuse the unchanged evaluator and source-answer oracle.
2. Reload both accepted parents; execute all32 existing source variants and five
   existing continuations with Read and NoRead in integer and F32 execution.
   Preserve complete text and per-row losses/gains. Do not net gains against
   losses. A retained source comparison allows at most2 first-noun losses against
   its executed parent, as before; report complete-answer regressions separately.
3. Compare identical observed tokens for the fixed first four full256 development
   windows. Record state/probability drift, NLL, total variation, top1 changes,
   exact Q48 normalization and every causal slot count. Prospective engineering
   limits: maximum absolute state and probability drift at most0.01 each.
   These are numerical bounds for this comparison, not new language criteria.
   The four windows belong to the exposed calibration prefix; they cannot pass
   the full976-window natural-language/comparison gate.
4. A construction/interface failure receives a localized implementation repair.
   If the numerical/retention criteria fail on a sound implementation, preserve
   the actual failure and identify the operator cause from these records. No
   automatic retraining, threshold revision or second parameter sweep follows.
   If the bridge retains behavior, integrate the measured numerical component
   and name the remaining host/serving boundaries precisely.

## Resources and parallel work

Reuse the clean isolated full worktree and shared target cache. The existing
cumulative balance is541,147,250/547,200,000ms; project a complete90-minute cycle,
including a conservative initial recovery charge, to546,547,250ms. No extension
is needed at this projection. The recorded deadline is20:24:18UTC. One Cargo
process; at most two model workers with nested backend threads1; aggregate RAM
8GiB; new storage1.5GiB, preserving14.5GiB physical reserve plus128MiB stop and
64MiB closeout. Initial free space16.88GiB. Runtime guards stop at the configured
limits; preserve partial roots and charge all attempts.

Independent implementation scopes: integer primitives; evaluator integration;
principal model/codec/table implementation. RDC runs DeepSeek's bounded numerical
review concurrently; its first two CLI launches failed before model work because
prompt mode excludes auto/plan flags, and the corrected launch completed. Reuse
that review and the principal adjudication; do not start another broad survey.
