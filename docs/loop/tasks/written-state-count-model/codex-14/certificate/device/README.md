# P1(c) CUDA tangent certificate

This separate diagnostic implements the Q32 tangent reduction and row-max witness
in [CERTIFICATE.md](../../CERTIFICATE.md). It reads the already sealed frame and
the independently verified fit. It emits raw device sums for the independent
certificate verifier. Each evaluated anchor is scored again on the device and
must exactly match the fit's all-target Q12 score for that anchor.

## Build and use on Wick

```sh
/usr/local/cuda/bin/nvcc -std=c++17 -O3 -arch=sm_89 -Xptxas=-v \
  kn_ceiling_certificate.cu -o kn-ceiling-certificate-device
cargo build --locked --release
/usr/local/cuda/bin/cuobjdump --dump-sass kn-ceiling-certificate-device > device.sass
```

Bind the source, both executables, and the native SM89 disassembly in the exact
code review. Integer restoring division avoids the floating-point reciprocal
that nvcc can emit for ordinary variable integer division. Use the established
Wick GPU lock and pinned device UUID. A corpus run needs a separate exact-code
FIT and pre-run review. Reserve enough capacity for a bucket-grouped frame in
both processes and one copy on the device, plus the CUDA context and reduction
buffers.

```sh
target/release/kn-ceiling-certificate "$FRAME_DIR" "$VERIFIED_FIT_JSON" \
  "$FIT_DIR" "$PWD/kn-ceiling-certificate-device" "$OUT_DIR"
```

`FRAME_DIR` contains the canonical frame, manifest and seal. `VERIFIED_FIT_JSON`
is the existing independent fit verification produced with `--query` and
`--fit`. The host checks the seal, identity, ordered bucket census and all mass
domains as it stages the exact bytes it hashes. `FIT_DIR/result.json` supplies
the ordered endpoint or initial weights. `OUT_DIR` must be new. On success it
contains `device-certificate.json`, which binds the two executables, manifest,
frame, fit receipt and independent verification by SHA-256.

## Raw receipt

The named anchors are pure KN and the earliest best pass in each U_count
start. The starting weights are available at pass 0. The final weights are
available at the fit endpoint. An earlier interior best pass appears with
`weights_unavailable`. For every other anchor the receipt contains all 176
buckets, their exact target counts and ordered masks, the max-sum lane (ties
use the lowest lane ID), and 13 directional sums. Present lanes include those
with zero row mass and those at the EM coefficient floor. Their sums are
decimal strings of widened integers. Absent lane positions are JSON null.
The score is in Q12 units. `rowmax_q12` is the independent per-target maximum
over present masses, scored using the same Q12 function.

For each row and anchor, CUDA computes the full-width mixed Q48 mass and
its exact Q12 score, and reduces ceil(2^32 × (p+1)/s) in two-word unsigned
arithmetic. Each term is at most 2^32 × (2^48+1). With fewer than 2^24 rows,
each reduced sum is below 2^104. The independent verifier applies the
registered log allowance and cutoffs to the raw receipt.

## Finite controls

Under the Wick lock, point `KN_CERT_TEST_WORKER` at the clean CUDA binary:

```sh
KN_CERT_TEST_WORKER="$PWD/kn-ceiling-certificate-device" \
  cargo test --locked --release cuda_tangent_matches_widened_finite_oracle -- --ignored
```

Compile a second binary with `-DKN_CERT_FAULT_FLOOR` and rerun the same test
with its path. The Rust widened oracle requires this fault to fail. Re-run
with the clean binary and check that it passes again. For the captured
ordinary-CLI synthetic frame, evaluate each binary into a distinct directory,
verify the exact rescan with the independent certificate verifier, and compare
the clean and restored receipts byte for byte.
