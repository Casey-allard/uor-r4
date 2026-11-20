# [codex] Independent FIT for finite direct compact CUDA readback

TEST FITNESS: FIT

**FIT PASS for the three exact C128 device packets on Wick**, conditional on a
separate GO naming the input manifest, the GPU lock and fresh outputs. I
reviewed the CUDA reader and witness as their non-author. This measures
whole-context presence, loss and native Q48 backoff through the compact integer
state, which is the next device check before a pruned count component can be
used in the P1(b) fit.

The [build and input manifest](readback-fit/build-and-input.sha256) SHA-256 is
`a96a7baf45e9bd3c3c0d3bcf91f6ccb793478e67414a3243977dc8b064c76832`.
The CUDA executable is
`9c576eba1e78807cc5fea60f55b15c5c4d1ab5c828ba882ed488b76e32b70dc3`,
and the Rust packet and witness executable is
`ccbc041709cf3b02bb748c913b1ce7d2aaaa88ab5a8ec25cdf4bbd1c0b7facc7`.
The device is Wick's RTX 4090 `GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41`,
driver 595.99.02, with real `sm_89` AOT images compiled by nvcc 13.1. Binary
inspection found two sm_89 images, no embedded PTX, and no floating,
transcendental or matrix instructions in the finite lookup disassembly.
[Reviewed sources](readback-fit/source.sha256)

| C128 case | Pinned compact SHA-256 | Bytes | Retained probes | Dropped probes |
|---|---|---:|---:|---:|
| All retained | `eea859b5d7272ef8d4be3a4ee623c92c2b11da866478b05765a7b20557b3788e` | 16,459,406 | 11 | 0 |
| 8 MiB cap | `b50e64c74be682f04ef1661b5599c6729ede643b005e062ffa71c0799b640f71` | 4,517,125 | 11 | 7 |
| Mandatory floor | `a8fbb755090b81a0329c5a817d80b74a1d6a7f451354f0299f789e77ddfdcb30` | 2,074,826 | 4 | 7 |

The launcher uploads each original pinned compact file as bytes. The 528-byte
order metadata carries section offsets, dimensions and the shared integer
discounts derived from each file's stored spectra. The probe tape carries
only the lengths and original token IDs of independent witness histories and
targets. The device performs its own canonical key assembly, binary search,
follower span read and integer Q48 interpolation over those compact bytes.
It returns raw and adjusted target words, raw and adjusted totals, and Q48
for every requested order. Its input carries no expected outputs. Each
fixture's output is checked independently in a cold process against the
ordinary count runtime and the pinned raw-word witness.

**Pass requires** exact equality of all five fields at every returned order
record from 40 probes, yielding 440 records including 254 active orders and
186 zeroed unused records. All 14 dropped top orders must have zero raw and
adjusted target counts and totals, with Q48 identical to the preceding suffix.
All 26 retained top orders must reproduce the original nonzero raw counts and
totals. Cold repetitions must produce identical bytes. A planted changed device
result must fail exact output comparison. A mismatched artifact/witness pin
must fail input preparation. The independent CPU witness also rejects the
repinned alteration of raw 128 to 129 and the separately valid floor artifact
substituted under the wrong threshold claim. [Independent corruption and omission controls](verify/README.md)

The largest finite fixture is 16,459,406 bytes, and the CUDA launcher requires
64 MiB of reserve in addition to the artifact, wire buffers and outputs. A
digest, identity, byte-length or arithmetic discrepancy is an instrument
failure. Save the first field difference and the run's outputs for repair, and
repeat this scope with refreshed pins. This FIT covers the finite CUDA native
reader comparison and the sealed corruption and backoff controls.
