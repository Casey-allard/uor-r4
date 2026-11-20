# D36 finite CUDA replay of the compact count component

The device consumes the actual UORKNC02 byte payload. The host stages the
compact section offsets, cardinalities and canonical discounts computed from
the payload's adjusted count-of-counts headers. The host also stages original
token IDs with the target last. Answers are absent from the GPU inputs. The
kernel constructs the ordered suffix key, binary-searches the serialized
context index and sums that context's serialized followers on device. It uses
D31's explicit integer quotient for the two Q48 divisions. A missing context
leaves the lower mass untouched.

Use the independently audited C128 all-retained, 8 MiB cap and mandatory-floor
artifacts. The audits supply 11, 18 and 11 probes respectively. The 8 MiB case
includes seven dropped-order witnesses with lower contexts present. The floor
case exercises empty sections at each of orders 4–10. Each device process emits eleven order records for
every supplied probe. Each visited order reports raw and adjusted target
count, both context totals and the Q48 mass after that order. Orders beyond
the probe history are zero-padded. Test threshold decisions, corpus fit and
language accuracy elsewhere.

The finite bar is exact agreement over all five words at every visited order
and exact zero padding for every unvisited order. The normal native compact
reader checks raw words and Q48 masses on all causal suffixes. It checks
adjusted words through its canonical visitor. The independent D36 verifier
receipt is pinned separately and supplies a second oracle for the full-probe
raw words and final Q48 result. Each arm runs in two fresh device processes.
Their output and verifier receipts must match byte for byte. Changing the
first output raw count must make the witness reject. A failure blocks device
qualification and further use of this launch path until diagnosed.

The Wick admission is the RTX 4090 and the exact SASS-only sm_89 image. The
finite artifacts total less than 24 MB and the largest single one is under
17 MB. The launcher streams host upload with an 8 MiB staging buffer. The
general ABI bounds the artifact to 4,000,000,000 bytes and each launch to
65,536 probes. At the launch limit the input and output tapes total under
25 MB, plus metadata, device status and the caller's other components. A
future fitter can retain one immutable device artifact across batches. Its
required runtime and throughput still need measurement.

Launch requires a non-author FIT, then an explicit parent GO binding the
source and staged build/input manifests and the assigned device UUID. No GPU
fit, language evaluation, R or held-input access is in scope.
