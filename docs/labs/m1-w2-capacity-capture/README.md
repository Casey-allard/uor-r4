# W2 read-code capacity capture: H_mixed

Item `m1-w2-capacity-capture` (#2845). Lab: claude. Order: [claude-87](../../loop/tasks/m1-w2-capacity-capture/claude-87/ORDER.md). Sheet: [`m1-read-code-capacity-2026-10-17.md`](../../harness/prep/m1-read-code-capacity-2026-10-17.md) (sha256 `1fd08b79…`, FIT at #2572 comment 6229680584).

**Result: H_mixed (sheet §4.7 row 3).** The table keyed by the Q4 model's exact read code beats the trained writer by a wide margin. But it stays below the query-token bigram, the next-word table that ignores the read. The pinned consumer verifier accepts the record in its runtime-evidence mode and recomputes the same band.

This is one checkpoint (seed-1 Q4, epoch 407), one DEV panel (14,287 rows) and the sheet's §4.5 fallback regime. It says nothing about other checkpoints, seeds or sizes.

## The numbers

| Arm (DEV top-1, n = 14,287) | Count | Note |
|---|---|---|
| T: exact read-code table | 3,414 | model-matched 1,049,856 TRAIN targets |
| G: query-token bigram | 6,300 | M = max(U, G) = 6,300 |
| U: model-matched unigram | 1,226 | equals the sealed full-TRAIN unigram top-1 (both always predict token 1; 1,226 DEV targets are token 1) |
| W: trained writer (sealed Q4) | 149 | re-verified row by row |

- **Paired discordance.** Here b counts rows the table missed and the comparator hit, and c counts the reverse.

  | Pair | b | c |
  |---|---|---|
  | U | 201 | 2,389 |
  | G | 3,729 | 843 |
  | W | 61 | 3,326 |

- **Floors.**
  - F = max(24, 2·√4,572) ≈ 135, so M + F ≈ 6,435.
  - H = max(24, 2·√3,387) ≈ 116, so W + H ≈ 265.
  - T = 3,414 lies in [W + H, M + F]. That is row 3: "the code beats the writer but not both co-primary references".
  - All comparisons are exact integers (squares, not roots).
- **Evaluability passes.**
  - Coverage is 14,175 rows with a seen code, against a bar of 12,859 (90%). The other 112 rows fall back to the unigram.
  - 41,806 codes have count ≥ 2, against a bar of 1,024.
  - Both co-primary pairs have b + c ≥ 100.
- **The admission-matched comparator (sheet §4.3 item 4).** It covers the 1,920 rows whose target cell the writer admitted. Every predictor is restricted to tokens in that row's admitted cells.

  | Predictor | Top-1 hits (of 1,920) | Top-8 hits |
  |---|---|---|
  | Code table | 1,138 | 1,782 |
  | Bigram | 1,482 | 1,895 |
  | Symmetry-pooled table | 988 | 1,734 |
  | Unigram | 754 | 1,586 |
  | Writer | 149 | 747 |

  So when the target's cell is admitted, the code table restricted to the admitted cells ranks the target first in 1,138 rows, against 149 for the trained writer.
- **Secondary, descriptive arms.** These cannot fire a band.
  - Address-only table: 1,675.
  - Interior-only table: 1,628.
  - Symmetry-pooled table: 2,135 full-panel top-1.
  - Top-8, full panel: T 7,558 (the code's own ranking), G 11,263, U 4,060, W 747.

## What it points to

Per the sheet, T did not cross M + F. So the next work is a code-utilization and capacity diagnostic, not a change to the learning law. The W4 float-credit sheet runs only on H_law, so it does not run.

The sheet asks the admission-matched comparison to localize the gap between the output map and the admission channel. Both lose hits the code could make, and on these counts admission loses more:

- **Output map.** On the 1,920 admitted rows, the code table restricted to admitted cells ranks the target first 1,138 times; the writer does so 149 times. That is at least 989 rows lost after admission (1,138 − 149; the two hit sets need not nest).
- **Admission.** On an admitted row, restricting the table to admitted tokens can only keep or gain a hit, so at most 1,138 of the unrestricted table's 3,414 hits fall on admitted rows. At least 2,276 fall on the 12,367 rows where the writer never admitted the target's cell, and the writer cannot get those right at all.

So on this checkpoint the larger measured loss is admission, and the output map loses most of what survives it. These are bounds from one artifact, not a decomposition of the mechanism.

## How it ran

1. **Importer** ([`q4_import.rs`](../../../crates/uor-r4-discrete/src/q4_import.rs)).
   - It is evaluation-only and loads the 15 Oct Q4 checkpoint (`4477eb5c…`) into today's device.
   - It authenticates the raw wrapper and the stored model (Domain A `49fa448c…`) before decoding.
   - It rebinds only the source and kernel identity. A reverse-equality check proves the model state is byte-identical.
   - The device it returns has no apply, save or export path.
2. **Gate R** ([`w2_capture.rs`](../../../crates/uor-r4-discrete/src/w2_capture.rs)).
   - Before any W2 number, the imported model's DEV evaluation must contain the sealed Q4 evaluation (`4fff1d3c…`) exactly, at every path: 224 batches, every row, the GPU aggregate and the count reference.
   - It did, with 0 mismatches over 14,287 rows. The new code adds 9 row keys the sealed file lacks; extra keys are allowed.
3. **Capture.**
   - The production `forward` ran over DEV and over exactly 16,404 TRAIN batches of 64, which are the model-matched 1,049,856 targets.
   - Each 8-int read block was decoded to a root id by exact match, with no ABSENT block.
   - Root round-trip was 14,287 / 14,287 and determinism 1,024 / 1,024.
   - The TRAIN stream is in the verifier's compact format.
4. **Record** ([`w2_capacity.rs`](../../../crates/uor-r4-discrete/src/w2_capacity.rs), [`w2_record.rs`](../../../crates/uor-r4-discrete/src/w2_record.rs), [`w2-assemble`](../../../crates/uor-r4-discrete/src/bin/w2-assemble.rs)).
   - Host counting builds T, U, G and the arms from the captured codes. This is the sheet's D24 classification: post-processing of GPU-produced read codes.
   - The DEV panel's ordered row binding reproduces the pin `e4ef44fb…` from the original binary recipe.
   - The symmetry actions keep every root a root (0 non-roots).
5. **Two runs, one RTX 5090** (Vast `vast-57603275`, 1,570 s at $1.106/h, about $0.48).
   - **Run A** used the binary built at `d186f5f54`.
   - **Run B** used the binary built at `db25268ba`. It was rebuilt only because the banned-presence gate flagged float types (wall-clock timers and test-only tuple reads) in the run-A source.
   - The two runs are byte-identical: TRAIN stream, query sidecar, `gate-r.json`, all DEV rows and the token cells.
   - **Run B is the run of record.**

## Artifacts

In the HF store `caseyallard/uor-r4-store`, under `results/claude/w2-capacity-2026-10-25/`:

| File | sha256 |
|---|---|
| `record.json` (`uor-r4.read-code-capacity/1`, 14,287 rows) | `69717971a6f4e6f4a6dede6ad5eee630c4abdbda13e4c8dd434ef126a74c1abc` |
| `runtime-manifest.json` | `238dde67564fddce605869593f8ba724fad6632b2810549a7b4e285953e7ac4c` |
| `run-b.tar` (capture, binary, job scripts, logs) | `6aa0a766bfe1dfdc3a21075b89cfb0411986d1e8f036bdcd30be4f90253f02c2` |
| `run-a.tar` | `3b8937d5644517d77f0fe4e62f4cf225d932229231811540d65e03bb4a43dcf1` |

Inside `run-b.tar`:
- `w2-out-b/train-stream.bin`: `70a587670edb2f5c26f8c2ba299c6bed58c437baf5b0ccc469ed8f316dc69c57` (4,199,424 B).
- `query-sidecar.bin`: `6d5d4f5dd8693b44aa69989a371b592b1361d82d328c700c574ab00228a2d7fe`.
- `gate-r.json`: `dc10676c1a898426be7b006d5c50016fe203ab1b3664941be2ba0bf9570f635f`.
- `capture.json`: `d22a4eb20643fb8b72871d40e44478a5f1892a5f3322fcd56cfbbbe3a2bb8eb1`.
- `w2-bin-b/w2-capacity`: `90b7cf6d7a11ce46bb45a5db62f945e2eb1a18aa7b6928a9af81e722b7a847b7`.

The record's top-level fields, minus its rows, are in [`record-summary.json`](record-summary.json).

## Check it yourself

```sh
# inputs: HF store w2-inputs/2026-10-25/ (SHA256SUMS), placed so local/driver/data -> driver-data
CARGO_TARGET_DIR=<target> ~/.cargo/bin/cargo run --locked -p uor-r4-discrete --bin w2-assemble -- \
  <run-b>/w2-out-b <new-record-dir> --ratchets-pass true
rustc --edition=2021 -O docs/labs/w2-k16-consumer-verifiers-2026-10-19/verify_w2_read_code_capacity.rs -o <tmp>/verify_w2
<tmp>/verify_w2 --record <new-record-dir>/record.json --sealed-evaluation <inputs>/q4-evaluation-16404.json \
  --runtime-manifest <new-record-dir>/runtime-manifest.json --compact-train-stream <run-b>/w2-out-b/train-stream.bin \
  --capture-mode production_forward_matched_1049856_fallback
```

The verifier (sha256 `db02488e…`) reads the DEV and TRAIN corpora from a hard-coded path under `/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data`. A symlink there to `driver-data` is enough. Its output was:

`{"verification_tier":"VERIFIED_RUNTIME_EVIDENCE","runtime_verdict_authorized":true,"precedence":3,"verdict":"H_mixed","T":3414,"U":1226,"G":6300,"M":6300,"W":149,"coverage_rows":14175,"codes_with_count_ge_2":41806}`

The GPU capture itself is `w2-capacity CHECKPOINT SEALED_EVALUATION NEW_OUT_DIR`. It needs `--features cuda` and an RTX 5090-class GPU, and is described in [`bin/w2-capacity.rs`](../../../crates/uor-r4-discrete/src/bin/w2-capacity.rs).

## Limits

- **Fallback regime.** It uses the model-matched TRAIN table only, with no full-TRAIN arm. The sheet says this biases against H_law (a smaller table is a weaker comparator). The record says so (`smaller_table_bias_disclosed: true`). T sits 3,021 rows below M + F.
- **Code table top-8.** It is padded from the unigram ranking when a code has seen fewer than 8 tokens, because the verifier requires 8 distinct tokens. The unpadded count is 7,558.
- **Capture binary and assembler head differ.** The capture binary is from `db25268ba`. The record was assembled at the later head, which adds only the host-side descriptive arms and a log-text fix; neither changes T, U, G, W or the band.
- **Restricted top-8.** The admission-matched arms pad a key's admitted ranking from the admitted unigram, so their top-8 counts are not comparable with the unpadded full-panel 7,558. A row whose admitted tokens were never seen in TRAIN gets no restricted prediction.
- **Pathological-ordering guard.** If M + F < W − H, the code returns H_rep for every T < W − H. That is the pinned verifier's reading. The sheet's text can be read as overriding only H_law. The case cannot arise here (W − M = −6,151).
- **One checkpoint.** It is one seed at one dose. The band classifies this artifact, not the mechanism.
