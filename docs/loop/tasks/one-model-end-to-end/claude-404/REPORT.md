# REPORT: one-model-end-to-end, claude-404 (two native training steps of the whole discrete model on the D87 student stream, CUDA against the CPU oracle)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Not every route, gate and target block was compared** [claude]. The ORDER listed every route and gate, plus the vector blocks of the batch's distinct target tokens. Two earlier runs at that size did not finish:
  - c404-run1 ran 41 minutes with no output and was stopped.
  - c404-run2 ran 8 minutes inside the CUDA propose-all loop and was stopped.

  The CPU oracle scores one candidate move at a time over all 64 rows × 192 positions, about 4.8 s per parameter here. The final list, fixed before scoring and never chosen by loss, is:
  - routes and gates with index below 16;
  - the 4 blocks of each of the batch's first 8 distinct target tokens.

  That makes 48 parameters per Route step and 64 per Flock step. Of each Flock list, 16 are routes, which both sides refuse.
- **The CPU re-save still writes a stale snapshot hash** [claude]. After each committed step, the artifact's `source.snapshot_sha256` differs: CUDA writes the hash of the stepped model, and the CPU applier keeps the hash of the bytes it loaded. This is claude-372's finding, now seen at corpus scale. Every other header value and every data byte agree. Counted apart (1 per step), not called a mismatch.
- **Only one batch, one seed and two steps** [claude]. This is agreement on 64 rows from one point of the stream. It does not show agreement over a training run.
- **No Flock `zero_gates_v1` comparison** [claude]. The CPU Flock oracle has no such start (claude-372 note 3), so Flock starts from `active_leaf_v1`.

## What changed

- `crates/uor-r4-discrete/src/cpu_oracle_parity.rs`: one new `#[ignore]` CUDA test, `student_stream_two_steps_agree_at_corpus_scale`, with its helpers:
  - `student_batch` reads `UOR_STUDENT_STREAM` (a `student-ingest` output folder) and fails loudly if it is unset or the files are absent.
  - `student_parameters` builds the bounded list above.
  - `compare_proposals_over` is `compare_proposals` over a caller-fixed parameter list, with separate CUDA and CPU rows, so the planted shift can move one target on the CPU side only.
  - `device_with` builds a device for a given spec, read and seed.

  It also checks that the CUDA read gives the same vectors from the corpus windows (with document identities) and from token-only windows. The CPU oracle sees tokens only.
- Progress lines every 16 parameters.
- The two existing tests are unchanged, and both still pass.
- No kernel, serving, export, artifact-format, driver or public-API change.

## Measurements

Run on `wick`'s RTX 4090 inside Claude's container, job c406-run2, 16 Nov 2026 [claude]:
- driver 595.99.02;
- nvcc 12.8 V12.8.93;
- `UOR_NVCC_ARCH=compute_89`;
- `discrete.cu` sha256 `f53ff7e2…7591`;
- built `discrete.ptx` sha256 `2df7b47b…b7a9`.

**Stream** (claude-403's `student-ingest` over two sealed D87 API shards):
- `tokens.u16` sha256 `a16a4412a7e5deaaca07307cbf75ae0ab745fdb39b253a1aa1e329ba65743238`, 15,471,412 tokens;
- `response_mask.u8` sha256 `25b9090dd530e5b256f39c8905c3cdc8f4c3bd46e90c7452b75f10e1eba2a7c1`;
- 13,559,938 eligible targets.

The batch is `PreparedCorpus::batch` from eligible target 6,779,969: 64 rows, context 192, 12,288 window tokens.

**Model:** vocabulary 4,096, 4 blocks, context 192, 8 kept cells, `NearestAddressMarginV1`, seed 1.

| Read | Step | Parameters | Forward words | Loss words | Choices | Refusals (both sides) | Committed | Epoch after | Snapshot-sha-only | Mismatches |
|---|---:|---:|---:|---:|---:|---:|---|---:|---:|---:|
| Route (`zero_gates_v1`) | 1 | 48 | 2,048 | 768 | 48 | 0 | `Route(0)` | 1 | 1 | **0** |
| Route | 2 | 48 | 2,048 | 768 | 48 | 0 | `Route(1)` | 2 | 1 | **0** |
| Flock K = 8 (`active_leaf_v1`) | 1 | 64 | 2,048 | 768 | 48 | 16 | `VectorBlock(56)` | 1 | 1 | **0** |
| Flock K = 8 | 2 | 64 | 2,048 | 768 | 48 | 16 | `VectorBlock(59)` | 2 | 1 | **0** |

- Step 2 on each read starts from step 1's CUDA export, loaded by both sides. This is the multi-step case claude-372 left open.
- After each commit, the data section is byte-equal.
- **Planted target shift:** one target moved on the CPU side only gave loss mismatches on 48 of 48 Route parameters. The planted pass runs the proposal comparison only, and the chooser gets CUDA's loss words. So it shows the shift is caught at the losses step. "Nowhere else" holds because nothing else runs, not because it was tested.
- **CUDA diagnostic times** (not a speed claim; claude-406 measures speed):
  - carry plus forward, 43–50 ms per step;
  - the full CPU-side comparison, 235–252 s per step.
- **Not timed, against the ORDER:** CUDA propose and apply on their own. claude-406 times the proposals.
- In the same job:
  - `cuda_and_cpu_oracle_agree_word_for_word_from_identical_bytes ... ok`;
  - `cuda_parity_catches_one_planted_fault_per_step ... ok`.
- The job: `test result: ok. 3 passed; 0 failed`, 1,205.93 s.

## What the numbers say

- **At the real model's size, on the real stream, CUDA and the CPU oracle make the same updates.** Across both reads and both steps, every compared word is equal:
  - 8,192 forward words;
  - 3,072 loss words;
  - 192 choices;
  - every refusal;
  - the four committed artifacts' data sections.
- This holds for the listed parameters, this batch, seed 1 and two steps. It is not a check of the other ~16,300 blocks, of other batches, or of a longer run.
- **The count assertions are weak.** The test requires only that the forward, loss and choice counts are above zero, not that they equal what the list implies, such as 16 loss words per scored parameter. The table above is consistent with the list (2,048 = 64 rows × 32; 768 = 48 × 16), but the test does not enforce it.
- **The CPU oracle is the slow side, at about 4.8 s per parameter.** A full-list comparison at this size needs a faster oracle or many CPU cores. It is a check to sample, not to run on every step.

## Checks run at the head

- G1: the test passed on `wick`'s 4090 for both reads and two steps, with 0 mismatches (above).
- G2: the planted shift is caught at the losses step (`r.steps() == ["losses"]`, which needs at least one mismatch). Only the proposal comparison runs in that pass.
- G3: the two existing parity tests pass in the same job.
- G4: the ORDER commit is the first of this unit's commits.
- `git diff --check` is clean. `loop pr-check --local` runs before queueing.

## Skills used

- `native-space-cycle`: state schema, shared operators, the raw-word replay oracle and lossless `UORLUT01` named in the ORDER and checked here.
- `scope-guard`: one test and its helpers; no kernel or API change.

## Still open

1. **The pre-registered timed run on `wick`.** It needs a sheet and a non-author FIT under §4.5, using claude-406's rows-K scoring with a larger row cap (the next speed unit).
2. **Compare every route, gate and target block** once the CPU oracle can score many candidates in parallel (deepseek-272 named this too).
3. **Fix the CPU applier's stale `snapshot_sha256`,** or record it as the CPU's contract (claude-372, item 1).
4. **Check identity independence after each commit.** The real-against-token-only vectors check runs once, on the starting state, and only for forward. Every later comparison carries token-only windows. A kernel path that reads a document identity only once a route is set would pass here. Repeat the check at the top of each step, and propose on the real windows too.
5. **Tie the counts to the list:** assert loss words = 16 × scored parameters, and forward words = rows × 32.

## Plain summary

The GPU trainer and the slow, exact CPU reference were given the same real batch from the new corpus, at the full model size. They took two training steps each, in two model variants. They computed the same numbers, word for word, and saved identical model data after every step. One deliberately wrong input on the CPU side was caught where it should be. This covers the 48 to 64 settings checked, one batch and two steps. It is strong evidence for the GPU trainer, not yet proof for every part of the model or for a long run.
