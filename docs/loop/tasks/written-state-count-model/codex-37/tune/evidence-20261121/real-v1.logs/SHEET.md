# [codex] D37 DEV-tune selection on frozen reserve-trained S5

**Decision.** The all-target 95% paired band for A1s minus A1 must have
a strictly negative upper endpoint in exact Q12 sums to select A1s.
Otherwise select A1. Report the novel-target band separately. Apply the
claude-442 cost fallback independently if the separately approved budget
cannot accommodate the split. A failed instrument or replay produces no
selection receipt.

**Inputs.** Pin the claude-420 DEV-tune census, UORT, novelty classes,
response mask, segment tags, and ID byte lengths. Require 2,670 complete
BOS/EOS records and 10,807,417 targets. Check the complete per-class target
and byte census, including the response denominator. The D36 deployed
compact has the same original C source and 3,912,334,481 physical bytes as
the R fits. The capture tape binds those bytes and the original DEV-tune
UORT, and is sealed by an independent count capture. C2 uses the AOT
exact-pair tape of the same DEV-tune UORT with one predecessor for each
original ordinal. The score GO pins both tapes and the R count tape.

**Frozen arms.** A1 and A1s share the same R Pick tape and the same
DEV-tune causal CUDA reads. A0 is pure count, C1 masks Pick with A1's
classes and zero Pick weights, C2 is latest exact pair, and C3 and C3s use
the shared seed-439 code permutation with BOS/EOS fixed. Every arm loads
an independently sealed, manifest-bound R-only weight file. The read
uses D11 original token IDs, original fixed-point state and 192 causal
prefix IDs at most. Counts use order at most ten and never cross BOS.
There is no optimizer invocation, mixture update, calibration step or
alternative source split.

**Replay.** Use the common Rust feature encoding and the fitter's CUDA
forward Q48 kernel. Check both record ends in the first 64 records, their
first eight prefix positions, both sides of score, count and Pick batch
seams, an interior modular sample and the final two targets. Native
readback compares raw count/total words and KN mass, predecessor,
distance, tie, successor, bucket and mixed Q48. C2 uses the native exact
pair oracle on the same probe ordinals. A finite negative moves distance
2 to distance 3 and another corrupts the tie count. Both must fail the
same exact row validator. No numeric tolerance applies. Require every
original ordinal and complete consumption of all seven scoring streams.

**Band and output.** Sum bits in Q12 by original record for all targets
and for novel targets. Resample 2,670 complete records with replacement,
1,000 times, using splitmix64 seed 4390 and the existing paired-band
indices. Use the fixed original byte denominator per cohort when mapping
to micro-BPB. Include the pure-count, masked, exact and permuted contrasts,
per-class targets and hits, all/novel/response sums, the seven sealed
weights' identities, the per-record sufficient statistics, and the
selected digest and carrier version. Keep DEV-test admission separate.

**Cost and admission.** Use Wick's registered sm_89 RTX 4090 and shared lock,
or one verified Vast RTX 4090 under the codex/d37-tune-21 uor-pod lease.
Pin the actual host name and device UUID in the Vast GO. Verify a 64-bit
Ubuntu 24.04 container, driver compatibility with the staged sm_89 CUDA
images, and the exact executable and transferred input bytes before use.
The single-pod plan is two lease hours, with an active watcher and prompt
offload and teardown when scoring finishes. Require no other active GPU compute, 8 GiB free device memory,
10 GiB available host memory and 2 GiB free output capacity. CUDA Pick
runs two complete reads, then each arm runs the frozen forward scorer in
8,192-row batches. Estimate under 60 minutes after staging. Record
actual elapsed time and RSS. A native divergence holds the affected arm
at its first differing ordinal. Use one private fresh output with the
reviewed source, finite result, separate nonauthor FIT and explicit
digest-pinned parent GO before launch.
