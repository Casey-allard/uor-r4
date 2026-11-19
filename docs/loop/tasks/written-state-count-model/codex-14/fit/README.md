# Order-10 CUDA ceiling fitter

The fitter consumes the independently verified `kn-ceiling-fit-frame/1` payload
from the detached KN instrument. The case pack and verifier define corpus
identity, target order, expert lane order and fold meanings. The fitter
validates the exact target count, frame width and row domains, including
the per-bucket absent lanes and the fixed uniform mass. It hashes the same
packed bytes it parses and requires the verified manifest and frame hashes.

Run after the final non-author FIT and capacity checks, with the lab's Wick
GPU lock held and `OUT` a new directory:

```sh
/usr/local/cuda/bin/nvcc -std=c++17 -O3 -arch=sm_89 \
  -Xptxas=-v kn_ceiling_fit.cu -lcrypto -o kn-ceiling-fit
kn-reference --verify-fit-frame "$frame_dir"
./kn-ceiling-fit "$frame_dir/frame.bin" "$targets" "$out" \
  --identity "$manifest_sha256" "$frame_sha256"
```

The fitter stages one lane-major device allocation and groups rows by record
parity and bucket in stable order. The permutation preserves per-target
inputs and uses exact commutative integer reductions. Fit subsets and held
scoring consume disjoint groups. It persists separate completed mode receipts
by rename and publishes `result.json` after every mode completes. Recover by
verifying the sealed frame again and selecting a fresh output directory. The
optional certificate is a separate computation.

Each fit trace includes the initial score and every post-update score.
`endpoint` gives the selected pass and `best_pass` gives the earliest minimum
among evaluated passes. `best_checkpoint_pass` gives the earliest minimum
among evaluated checkpoints. The registered checkpoint rule selects the
primary. The controls always stop at 24, with their plateau status recorded
for diagnosis. `floor_dwell` counts the update endpoints from 1 through the
selected pass at which an enabled lane had Q32 weight 1. All 176 buckets
remain in the receipt, including buckets with no fitted rows. Per-bucket
`targets` counts the fit subset. A split fold's `scored` field describes its
held subset. `u_split.scored` is the exact device sum of both held tallies.

The GPU evaluates weights, responsibilities, all traces and final scores.
A denominator at most 2^80 needs two integer words before normalization.
The top 32 denominator bits give an upper estimate of the exact responsibility
quotient, within two integer units. The device divides by that word using an
integer restoring reciprocal reused across lanes and corrects the estimate
against the complete denominator. The second normalization is exact with the
same operation. For fewer than 2^24 targets, total Q32 responsibility per
bucket is less than 2^56. A Q12 objective sum is less than 2^42. The CUDA
fitter uses the 12-round integer score from the frozen Rust likelihood
implementation as its arithmetic contract.

## Finite oracle

The standalone Rust oracle generates a bounded synthetic frame and expected
little-endian words. It imports the frozen likelihood score and uses native
`u128` operations independently of the CUDA quotient. Build from this
directory so the source path to the frozen likelihood module stays intact.

```sh
rustc --edition 2021 -O finite_oracle.rs -o "$scratch/finite-oracle"
"$scratch/finite-oracle" "$scratch/synthetic"
n=$(cat "$scratch/synthetic/targets")
./kn-ceiling-fit "$scratch/synthetic/frame.bin" "$n" "$scratch/cold-a" --fixture
./kn-ceiling-fit "$scratch/synthetic/frame.bin" "$n" "$scratch/cold-b" --fixture
cmp "$scratch/synthetic/oracle.bin" "$scratch/cold-a/fixture.bin"
cmp "$scratch/cold-a/fixture.bin" "$scratch/cold-b/fixture.bin"
cmp "$scratch/cold-a/result.json" "$scratch/cold-b/result.json"
```

Hold the lab GPU lock on the device runs. Each fixture witness includes the
per-target pure KN Q12 scores, every initial weight, every per-pass bucket
responsibility sum and updated weight, fit and score tally words, and floor
dwell counts. The fixture includes score powers and neighboring masses, the
Q48 extremes, absent lanes, present zero-mass lanes, empty buckets,
multi-block buckets and both record parities.

Use the supplied malformed fixture payloads to check exact EOF and lane/class
domain failures. The finite corruption build adds
`-DKN_FIT_FAULT_REMAINDER` to the nvcc invocation and uses a separate binary.
It must fail with a CUDA weight invariant error and produce no aggregate
receipt. Rerun the clean binary afterward and compare the witness. The
`--fixture` argument caps input at 65,536 targets and leaves out the manifest
binding. The production-mode fixture may use the ordinary binding arguments
to exercise manifest and payload hash checks on synthetic data.
