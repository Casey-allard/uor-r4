# D37 reserved-R fitter

The primary production invocation is defined in [SHEET.md](SHEET.md).
The [finite numerical admission](SHEET-FINITE.md) exercised the same standalone
sm_89 optimizer on 128 invented rows per arm. Wick's fixed-case outputs matched
97 Q12 scores and 717,024 Q32 words for A1 and 97 scores and 1,195,040 words for
A1s. Their last live bucket probes matched at 527 and 879. The truncated
five-class tape failed before producing a trace.

Use a private Wick build directory. The D31 Pick implementation and typed
Rust ceiling table are shared directly. The CUDA optimizer retains D31's
integer restoring quotient and deterministic Q32 recurrence. Build with the
pinned Wick toolkit and explicit `compute_89` and `sm_89` targets. Audit
the exact embedded cubins and standalone optimizer before dispatch.
`finite-dispatch.sh` and `dispatch.sh` verify the reviewed release records,
require Wick's shared GPU lock and create fresh output directories.

## Wire formats

| File | On-disk contract |
| --- | --- |
| Count input | D36/D37 `UORCBF01`, 112-byte header with row count and TRAIN, compact and UORT SHA-256, followed by 140-byte rows. The D37 production reader checks both exact length and consumed SHA-256. |
| Pick tape | `R37PK001`, u64 count, R SHA-256 and trunk SHA-256. The row carries four u32 words for predecessor index, distance, successor original ID and hit. Missing read fields use `u32::MAX`. The receipt identifies the Pick mode. |
| Optimizer input | `R37EM001`, u64 target count and u64 bucket count, followed by one u64 bucket and fourteen Q48 u64 masses per row. |
| Optimizer trace | `R37TR001`, same counts, followed by 97 entries. Each contains one u64 Q12 score and the full bucket-major Q32 state. |
| Probe output | Two u64 words per ordinal, bucket then mixed Q48 target mass. The distinct sorted probe ordinal tape uses u64 words. |
| Fitted weights | `UORS5W01` for 528 buckets and `UORS5W02` for 880. The carrier binds the exact arm manifest and reloads the exact Q32 words. |

All integer words use little endian. Each real row uses its original ordinal
among non-BOS UORT tokens, including EOS. Primary arms use the same count
tape, same Pick tape and same rows. The class determines only the bucket
address. Native replay compares original causal prefix, read words, counts,
totals, bucket and target mass. Full-vocabulary argmax and original-ID tie
parity belong to client-side replay.

## Controls

The `disabled` preparation puts every row in the closed class, equivalent
to the count-only A0 table's first 176 buckets. `prepare-c2` joins the AOT C2 selector tape for the latest
exact causal pair. `permuted` runs C3/C3s with the registered 439 seed and
fixed BOS/EOS mapping. C1 uses the `prepare-masked IDENTITY_DIR PREP_SHA
NEW_DIR` command to preserve A1's observed classes and remove only the Pick
lane. Compile its CUDA backend with `-DD37_MASK_PICK` so initialization and
regularization leave the Pick column at zero, then verify the masked planted
fixture using `finite-masked` and `verify-finite-masked` before real use.
C1's real seal verifies the disabled column at every pass.

The [measured fit receipt](RESULT.md) names the exact Wick production
overlay and the later control builds. A0, C1, C2, C3, and C3s each have
a manifest and seal in the [reserved-R control receipts](evidence/controls/real-v1).
The separate [DEV-tune result](../tune/RESULT.md) freezes the selected
identity arm and preserves the held split.
