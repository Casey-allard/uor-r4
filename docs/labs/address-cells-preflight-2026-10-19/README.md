# Address-structured injective cells — zero-update pre-flight receipt (2026-10-19)

- **Line:** M1 learned attention (#2029). **Lab:** claude. **Count:** this changes no headline number (pre-flight only).
- **Sheet:** [`m1-attention-address-structured-cells-DRAFT-2026-10-19.md`](../../harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md). It is TEST FITNESS: FIT (#2746), with registration bindings in #2748. Its SHA-256 at the freeze is `b94d5ae02ca433de67d81d4315a03676f931bac914625133d8973afec8b41275`.
- **Authorization:**
  - Kimi PM dispatch, #2029 `6258511411`.
  - Casey's approval, recorded in Kimi's chat at 02:45Z and re-confirmed to Claude directly in chat at 03:06Z.
  - No owner-identity phrase was needed or typed: zero updates, and nothing was released.
- **Scope:** five arms at checkpoint 0 with **zero training updates**. No training hold is released.

## Run

| Item | Value |
|---|---|
| Source freeze | `93b54f48c4e1983e278140f77a3377041ddd65e4`. It includes #2743, which by source the battery never calls. |
| Pod | `vast-56660582`: one RTX 5090, Vast.ai Czechia, $0.815/h. Bootstrap parity PASS, nvcc 12.8. |
| Battery | [`evidence/asc-battery.sh`](evidence/asc-battery.sh) (`3dbca767…`), run 03:12:47Z–03:17:30Z, `exit=0`. |
| Pod time and cost | About 13 min of pod life, about **$0.18**. The pod was deleted after archiving. |
| Archive | HF store `caseyallard/uor-r4-store:vast-archive/vast-56660582/` (read-back verified, 79 files). The full result tree (92 MB, 72 files) is hashed in [`evidence/FILES.sha256`](evidence/FILES.sha256) and also verified locally. |

A first launch at 03:11:55Z stopped before any command ran. The script's exclusive `mkdir` of the result root failed because the parent `runs/claude/` did not exist on the fresh pod. That was an instrument fault in my script. It was fixed (create the parent, keep the leaf exclusive) and relaunched once.

## Instrument gates (all PASS)

- **Inputs:** all six pinned data and table SHAs verify on the pod: TRAIN `3dee858f…` / `b5e7352d…`, DEV `0919e90f…` / `a838bb1a…`, tokenizer `d36d3e87…`, PrimeZeta `83c1380a…`.
- **Table rebuild:** `address-cells-codebook` on the pod rebuilt A8 = `6600bdb1…` (table `4122cbab…`) and S8 = `a185bdca…` (table `d2fb8680…`), both equal to the pins. The counts are `755a5a81…`, with 4,096 distinct vectors and 256 occupied cells.
- **P8 exactness (D31):** **427 / 14,287 full and 127 / 3,847 P_succ**, the 10-19 values exactly. This also shows #2742's `corpus.rs` accessors change no behaviour under the new driver hash.
- **K = 16 join precondition (D31):** on P8, P16 and A16, there are 14,287 rows, all keyed and unique. Exactly 3,847 join to `is_psucc`, 0 are unjoined, and 0 fall outside `admitted_cells − address_rivals ∈ {0,1}`.
- **Formula check:** on P8's rows, the formula reproduces 427 / 127.
- **Bracket configs:** `attention-build-identity` refuses the K = 16 configs (rc 2) at the attention K = 8 pin, as the sheet expects. `init-corpus` and `eval-corpus` accept them, which is source item (c).

## Bar (sheet, fixed before measurement), applied to A8

| Gate | Result |
|---|---|
| G-Init: the checkpoint-0 table equals the pin word for word | **PASS** on P8, A8 and S8 |
| G-Init negative (a): an edited A8 table with stale hashes is refused on the host | **PASS** (rc 2) |
| G-Init negative (b): a re-hashed, re-pinned non-root word is refused on the GPU | **PASS** (rc 2, status 6) |
| G-Init: routes, gates and epoch | Identical across all five arms on three different tables: routes `71cdce57…`, gates `ceb6ae99…`, epoch 0. Only the table differs. *Limitation:* no unpinned-initializer arm was run, so this is cross-arm equality, not the sheet's literal unpinned comparison. |
| G-Occ: `occupied_cells` > 16, zero tie mass, no split alias group | **PASS**: 256 cells, max occupancy 16, alias groups all size 1, 0 non-representative rows |
| **B3_init(P_succ) ≥ 500** | **PASS: 1,351 / 3,847** (full panel 4,244 / 14,287; complement 2,893 / 10,440) |
| **Net G-Cap > max(500, top1_control(cp0) + 24)** | **PASS: 1,351 > 500.** Net is min(alias ceiling 14,287, B3 P_succ 1,351). The control's checkpoint-0 top-1 is A8's own, 133, because Relation and FrozenZero share the identical checkpoint-0 model: the 10-19 receipt shows `model_sha256 134fd663…` and top-1 38 for both. |

**A8 clears every gate.** Per the sheet's step 4, A8 is promoted to a training-sheet draft. No KEEP or ranking claim follows from a pre-flight.

## Readings (one-sided, fixed before measurement; 3,847 P_succ rows, paired by row)

| Contrast | n11 | b (A8 only) | c (other only) | n00 | b − c | F = max(24, 2·√(b+c)) | Verdict |
|---|---:|---:|---:|---:|---:|---:|---|
| A8 vs P8 (placement) | 49 | 1,302 | 78 | 2,418 | **1,224** | 74.3 | **Placement moved admission** |
| A8 vs S8 (relation vs packing) | 93 | 1,258 | 383 | 2,113 | **875** | 81.0 | **Attributed to the predecessor relation** |

The marginals reconcile. A8: 49 + 1,302 = 1,351. P8: 49 + 78 = 127. S8: 93 + 383 = 476. Each contrast totals 3,847.

**Bracket (decides nothing):**

| Arm | Full admitted | P_succ admitted | top-1 at cp0 |
|---|---:|---:|---:|
| P8 | 427 | 127 | 38 |
| P16 | 872 | 246 | 37 |
| A8 | 4,244 | 1,351 | 133 |
| A16 | 6,023 | 2,178 | 133 |
| S8 | 1,733 | 476 | 99 |

P16 is about 2× P8, as uniform placement predicts, so admission scales with K. The sheet predicted about 254 for P16 on P_succ. Placement alone at K = 8 (A8) gives about 10.6× P8 on P_succ.

**Proxy ceiling:** P8's `address_distance(q, cell(last token))` is not reported. No existing device output supplies it, and the sheet marks it as not a gate.

## Reading

- Placing each token's cell near its TRAIN predecessors' cells is what moves admission. It lifts P_succ admission from 3.3% to 35.1% at the same K = 8 and the same 256 × 16 cell space, with no change to the read.
- Frequency-matched shuffled packing (S8) recovers only part of the gain: 476 against 1,351.
- This is checkpoint-0 admission only. It is evidence that the attention read can now see its targets, not evidence of learning. Top-1 at cp0 rises from 38 to 133 with no training.

## Evidence

- [`evidence/grading.json`](evidence/grading.json): every number above, recomputed by jq from the stored outputs on the host (D24).
- [`evidence/job.log`](evidence/job.log): the full battery log.
- [`evidence/FILES.sha256`](evidence/FILES.sha256): the hashes of all 72 result files in the HF archive.
- [`evidence/asc-battery.sh`](evidence/asc-battery.sh): the exact script.
- [`evidence/neg-b-nonroot-table.json`](evidence/neg-b-nonroot-table.json): the negative-(b) input. It is the PrimeZeta table with token 0's block 0 set to `[0,2,2,0,0,0,0,0]`, a ±2 pair that is not among the 16 roots. Its word digest is `8bf27283…` and its file SHA is `070478a0…`, and it keeps 4,096 distinct vectors. It was built with jq/awk/xxd, after checking that method reproduces the original table's declared `table_sha256` `e9e03bcd…`.
- `evidence/SHA256SUMS` covers this directory (no self-entry).

AGY's consumer verifier (#2750) expects its own bundle layout (`rebuilt-table-shas.json`, `k16-join.json` and others). Kimi's dispatch gives AGY that consumer-verification step, against the HF archive above. DeepSeek verifies independently.

## Next

The A8 training-dose sheet: the parent's Relation versus FrozenZero contrast with only the table changed. That sheet needs its own FIT, PM dispatch and Casey's typed D40 release before any training.
