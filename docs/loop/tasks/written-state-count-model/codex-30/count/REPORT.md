# [codex] C128 count storage and lookup readback

No measured full-C1 compact artifact, peak memory trace, lookup throughput, or matched-byte held-out quality is available from this experiment. The expanded probe reloads both representations for the exact differential. Its allocation counts cannot characterize the peak of a compact-only process. A1's 59,180-byte reservation was unfilled. Mixture fitting, ordinary inference, DEV-tune and sealed DEV-test were outside the registered run.

## Measurement

Two fresh `compact-count` processes on the same leased RTX 5090 counted the first 128 complete C1 TRAIN records (111,671 tokens, 223,406 input bytes). Each counted the complete raw order-0–10 tables and consumed the production adjusted sweep. The counting path used the CUDA kernels with a bounded host derivation of the order-zero continuation from the GPU-counted order-one support. Each process wrote and reloaded both codecs under their digests. Both processes produced identical raw and adjusted triples and contexts digests for all 11 orders, identical artifact bytes, and identical lookup receipts. Each fresh comparison passed 12,678 probes (8,582 strided training targets and 4,096 seeded synthetic histories), comparing raw count words, raw totals and integer KN Q48 results. The tape SHA-256 is `7dcd116dae4ebef8d7c96f5c17916cec55deb75d9d09cd9c276b52e078febd9e`. The finite local test `compact_lookup_preserves_native_words_and_refuses_invalid_rows` passed.

| Bytes | Expanded UORKNC01 | Compact UORKNC02 |
| --- | ---: | ---: |
| Serialized count file | 85,494,240 | 16,459,406 |
| Resident count representation payload | 110,995,296 | 16,459,406 |
| Serialized count file plus original seed-one D11 | 172,030,372 | 102,995,538 |
| Same sum plus unfilled A1 layout reservation | 172,089,552 | 103,054,718 |

The compact count file uses 80.75% fewer bytes. Expanded residency counts the typed row payload. Compact residency counts the borrowed file byte payload. Both exclude allocator overhead, lookup metadata, temporaries and other process allocations. The original trunk accounts for 86,536,132 serialized bytes in both sums. The two serialized-file sums describe the component sizes and were computed from independently loaded files.

The registered C1 projection uses the 10‰ census `pick-p10.json`. Its 20,754,301 training tokens match the D29 C1 manifest. The 1000‰ census `pick-p1000.json` describes the 2,075,608,357-token full C dose. Both censuses have matching raw and adjusted cardinalities at every order. The projection adds the 44-byte global header, eleven 72-byte order headers, `ceil(12k/8) + 4` bytes per context at order `k`, and 10 bytes per gram below order ten or 6 bytes at order ten. [D29 C1 manifest](../../codex-29/count/manifest.json) · [10‰ census](../../claude-439/run2/pick-p10.json) · [1000‰ census](../../claude-439/run2/pick-p1000.json)

The compact projection is **2,229,740,835 count-file bytes for C1** (about 2.08 GiB), or 2,316,336,147 bytes with the original trunk and unfilled A1 layout reservation. The earlier expanded estimate is 11,210,050,512 count bytes, a difference of 8,980,309,677 bytes. The same formula gives 148,096,362,864 count-file bytes for full C, 145,866,622,029 bytes more than C1. These are format arithmetic on historical cardinalities. The two C128 runs checked exact support and readback at their registered dose. [D29 expanded estimate](../../codex-29/count/README.md)

## Artifact identity

The [receipt](RECEIPT.json) records input and trunk hashes, per-order digests, counts, artifact hashes and lookup tape hash. The [sheet](SHEET.md) binds source tarballs and the cut. The on-pod build used Rust 1.97.1, CUDA nvcc 12.8.93 targeting `compute_120`, and driver 580.178.04 on GPU UUID `GPU-ae327616-54ab-beda-1a69-8c6d67439ef8`. The ELF listing identified `sm_120` and the PTX listing was empty. The linked-library check passed.

| Build object | SHA-256 |
| --- | --- |
| `compact-count` | `b4eb3d32a65d23b0a860198484310196b6b3e21a7d64e710a16d145a8bdcc787` |
| `ngram-count.cubin` | `9661ad0e8615062fd6b07a860b552a1931fe92d0b34c636569791a0b671e192f` |

GPU job `20261120T025206Z-count30-c128-gpu0` completed with exit 0 at 02:52:10 UTC. The count files and build objects were copied to private local custody under `local/codex/context-swap-30/count/custody-v1/` and to private Wick custody under `/home/callard/uor-codex/20261120-context-swap-30/count/custody-v1/`. Both off-pod copies passed hash verification for both artifact pairs, both receipts, the build manifest, the authorization manifest and the staged input manifest. Both count files and receipts also compared byte-for-byte on each host. The lease was released and live status confirmed removal of `vast-61560708`.

The measured overlay is preserved at commit `807bca6f655a291671857aadc2e8f6fa4946d22b`. The delivery change destructures the integer discount pair by name. The source scanner attributed a floating literal to the original Rust tuple field access. The original GPU receipt binds the pre-edit source. The compact decoder regression tests were run again on the delivery source.
