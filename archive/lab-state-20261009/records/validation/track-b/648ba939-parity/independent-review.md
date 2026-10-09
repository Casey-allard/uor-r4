# Independent bounded Track B checkpoint-parity evidence review

Reviewer: Codex `/root/second_council_review`, delegated by `/root`; separate non-author review session within the same lab/provider. Date: 2026-09-30.

**Decision: evidence is internally consistent within the inspected scope; no required corrections found. This is a confirmed completed numerical failure, not a parity PASS.** No builds, model executions, source edits, report-root mutations or GitHub writes were performed by this reviewer.

## Bound identities

- Executed source: `648ba939020054679f20e58d048dd6ae10a87758`.
- Executable SHA256: `f1892eb7281cd15007f894d0867b1daadc1ebc22e8b53b8f4057ed23e6983be4`.
- Report root: `/Volumes/UOR-Workspace/uor-r4-models/track-b/parity-648ba939-20260930`.
- Inputs SHA256, independently computed: `21c54e6c97c9c4c5ce29e6286206fbfeb7196894a6c5f5a0278a9b32a014dbf9`.
- Result SHA256, independently computed: `72af2546ae08d965712d3fb58f7f93a0e0548e81d4fe970f6babc4b54f8f3b2f`.
- Manifest SHA256, independently computed: `a575fe03a1fa42869bb73c7a70ff95ba13382582ea019b29a571ef55a01970b3`.

The report inputs and build handoff agree on source/executable; independently hashing the current executable at the attempt's argv path gives the exact recorded hash. Source lines 224–227 obtain revision and diff identities from compile-time environment values, not from a later runtime Git checkout. The recorded source diff is the empty SHA256. The retained optimized build log shows successful compilation of the actual repository-patched Candle core and the training example. This is consistent source binding; I did not reproduce the build or independently reconstruct its full environment. The four model/tokenizer hashes in the retained input-recheck receipt exactly match inputs.json; I did not rehash the large model file.

## Coverage and arithmetic outcome

I parsed the 325 KB result and all four small JSONL metadata files, without loading the large raw-logit arrays. There are exactly **294 unique comparison rows**, each with 49,152 logits, totaling **14,450,688 scalar comparisons**. Result rows equal the corresponding JSONL records exactly. Mode counts independently match the declared expectation:

- Stock per backend: 45 singleton positions, four full-prefill final rows, four half-prefix/singleton final rows = 53.
- Shared per backend: 45 full-prefix positions, 45 batch-two row-zero positions, four batch-two row-one valid positions = 94.
- Every 45-position mode covers precisely all (window, position) pairs of lengths 1, 4, 8 and 32. The retained log separately records all 45 unique reference positions. Reference raw-file size is exactly 45 full vocabulary rows. The shared second batch row is intentionally `[1,0,...]`; its four scored position-zero outputs map to reference window zero/position zero in source. The other 41 padding rows per backend are retained but unscored as declared.
- All 294 row summaries are finite. Raw offsets equal row index times vocabulary times four bytes and lie within the named files. Reported aggregate maxima, cell counts and exceeding-cell counts equal sums/maxima over the rows. Every row's pass flag agrees with its finite status and unchanged 1e-4 maximum-absolute criterion.

| Implementation/backend | Rows | Max absolute error | Cells exceeding 1e-4 | Failed rows |
|---|---:|---:|---:|---:|
| Stock CPU | 53 | 0.0014320611953735352 | 138560 | 7 |
| Stock Metal | 53 | 0.008588790893554688 | 345550 | 14 |
| Shared CPU | 94 | 0.0070133209228515625 | 349490 | 10 |
| Shared Metal | 94 | 0.0067539215087890625 | 381652 | 12 |

The state `FAIL_NUMERICAL_GATE`, pass=false, training_steps=0 and b2_authorized_by_this_result=false are correct for these records. All four maxima occur in window two, `[1,2,...,8]`. The reference execution receipt reports 45 completed forwards with two workers and zero remaining active workers/streams. Arithmetic owner is recorded as `uor-matmul exact GEMM`; its private selected SIMD backend is explicitly unavailable and must not be inferred.

## Seal and scope limits

The actual report directory's file names exactly match the manifest inventory plus manifest.json; all listed byte sizes match. The retained execution log explicitly reports sealing and verification. I independently checked this inventory/size consistency and the small-file SHA256 values, **not** every BLAKE3 content hash or the tens of megabytes of raw arrays. Therefore this review does not claim an independent full seal re-verification or recomputation of every reported logit error. The producer's completed verification and retained hashes remain the complete-content evidence.

Build log timing is 372.42 seconds; parity-process log is 387.50 seconds. Result-driver timing is 386.8010895 seconds, including reference construction at 366.998780708 seconds. Different timer scopes are explicitly labelled and consistent. These are not exclusive optimized serving measurements or language quality evidence.

## Review of draft published evidence

Reviewed the uncommitted docs-only writeup, compact JSON and current-state delta atop measured source648. The compact per-backend summaries exactly equal result.json with only rows removed. Numeric totals, maxima, failing prefixes, source/executable identity and claim limits in the Markdown agree with the inspected evidence. No materially overbroad numerical/model claim found. The draft correctly retains the separate harmonic failure, avoids treating tiny KL as an absolute-logit PASS, and does not authorize B2 fitting.

Reviewed documentation SHA256 values:
- `docs/evidence/track-b-parity-2026-09-30.md`: `ff15feca7c45c355d242309fa4bd27e7ad3695610ecdc262a127ba88fd5b0c9d`.
- `docs/evidence/track-b-parity-2026-09-30.json`: `1541bdf4c80f5b799b9f963b1a3ef43b367b6cd2bef2e1a7258667805486f4eb`.
- `docs/integration/current-state.md`: `e7f59b7f5e6ea84733e1ae1273e967b4aad45aca12619298c920a852ddc468ee`.

## Bounded next action

Approve the proposed causal diagnosis, not a full unchanged rerun: retain these outputs and trace the first divergent layer/operator using identical inputs. Stock CPU first misses at prefix `[1,2]` (4.2629241943359375e-4); stock Metal already misses `[1]` (1.0573863983154297e-4). Shared CPU's first full-prefix miss is window two position three (`[1,2,3,4]`, 6.86168670654296875e-4); shared Metal first misses position one (`[1,2]`, 1.41143798828125e-4).

When reducing a shared full-prefix case to a shorter run, preserve or explicitly compare original tensor/batch shape: a mathematically causal prefix can still change floating-point GEMM/reduction ordering when the execution shape changes. Use the retained failed full shape as the anchor, then discriminate shape effects. Do not infer the dense failure's cause from the separate harmonic cancellation experiment. Keep tolerance and arithmetic reference unchanged; no B2 training or expanded scale until the required parity gate is met.
