# REPORT: m1-w2-capacity-capture, claude-87

Lab: claude. Order: [ORDER.md](ORDER.md).
Closes #2845

## What did not work, first

- **The first pod job never reached the GPU.** `uor-pod run … -- bash -lc '<script with ; and $S>'` lost its quoting through ssh. Only `S=…` ran inside `bash -c`; the `cd $S` ran in the outer shell with `S` unset, so cargo found no `Cargo.toml` (exit 101 at the same second). The job now ships as a script file over stdin and runs as `bash /workspace/w2-job.sh`.
- **The run-A source failed the banned-presence gate, so the capture was run again.**
  - The gate counted 10 D32 sites at `d186f5f54`:
    - six `f64` wall-clock fields in `w2_capture.rs`;
    - four `decide_band(..).1` tuple reads in `w2_capacity.rs` tests, which its lexer reads as float literals.
  - None was in model computation or counting.
  - `db25268ba` makes the wall times integer milliseconds and destructures the tuples, and the gate now passes.
  - Run B used a binary built at `db25268ba` and is the run of record. Runs A and B are byte-identical: TRAIN stream, query sidecar, `gate-r.json`, all DEV rows and the token cells.
- **The first row-binding attempt used the wrong recipe.** The harness reference scorer's nine-field text lines gave `1144080a…`, not the pin `e4ef44fb…`. The pin was made by the 14 Oct `relation-subpanel-audit` (`ee047cb88`) with a binary format: u64 ordinal, u64 position, u32 query, u32 target, a copy-successor byte and a bucket byte. With that format the hash matches.
- **Three builder rounds handed back early on their context budget.** One round delivered only the symmetry check, one only the row binding, and one only the core library. Each later round started from what the previous one had learned.
- **Run B's Gate R log line printed "7695s" for 7.7 s.** It used the new millisecond field with the old "s" suffix. This is log text only and is fixed in `aa76619d9`.
- **The first write-up overclaimed where the loss sits.** It said the output map was "the immediate bottleneck, more than the admission channel". The fresh review showed the record's own counts point the other way. At least 2,276 of the table's 3,414 hits are on rows the writer never admitted, against at least 989 lost after admission. The lab page, this REPORT and the notebook now state both losses as bounds.
- **The review found code nits, fixed before merge.**
  - A Gate R mismatch count was read but not required to be 0.
  - The writer's top-8 was copied from the capture unchecked.
  - The McNemar helper's bound allowed n up to 127, where its binomial can overflow.
  - An arm zipped rows with slots without a length check.
  - The importer's real-file test read `UOR_W2_Q4` as a file, while the W2 tests read it as a directory. With the directory form set, the full suite failed that one test.
  - The 64 actions' closure under composition was untested.
- **The pathological-ordering guard follows the verifier, not the narrowest reading of the sheet.** If M + F < W − H, the code returns H_rep for every T < W − H, as the pinned verifier does. The sheet's text can be read as overriding only H_law. The case cannot arise on this data (W − M = −6,151).
- **`uor-pod down` could not archive the pod** (an internal `uor-r4/pods: Is a directory` error, then no SSH). Both runs were already in the HF store, so it ran with `--confirm-archived`.

## What changed

All in `crates/uor-r4-discrete`, with no driver source touched (`driver_source_sha256` unchanged):

- **`q4_import.rs`.**
  - An evaluation-only importer for the pinned Q4 checkpoint.
  - It authenticates the raw wrapper and the stored model (Domain A `49fa448c…`) and rebinds only the source and kernel identity.
  - A reverse-equality check refuses any other drift, including an explicitly stored default.
  - The returned device exposes forward, evaluate and four read-only accessors. A source-text test pins that surface.
- **`w2_capture.rs` and `bin/w2-capacity.rs`.**
  - **Gate R:** a sealed-subset comparison of `batches`, `aggregate`, `count_reference` and `model_epoch`, with the sealed file pinned by length and sha.
  - **Capture:** the DEV and TRAIN production-forward capture, writing `dev-capture.json` before TRAIN starts.
  - **Faults:** `fault.json` on every post-Gate-R error. A determinism or round-trip mismatch stops the run.
- **`w2_capacity.rs`.** The host side:
  - root decode by exact match;
  - the T, U and G tables with the sheet's tie and fallback rules;
  - the address-only, interior-only, symmetry-pooled and admission-matched arms;
  - paired discordance, exact McNemar, the evaluability rule and the §4.7 band, all in integers;
  - the DEV panel and its ordered row binding.
- **`w2_record.rs` and `bin/w2-assemble.rs`.**
  - They build the `uor-r4.read-code-capacity/1` record and its runtime manifest.
  - They refuse on any pin, join or conservation mismatch.
- **Docs.**
  - [`docs/labs/m1-w2-capacity-capture/README.md`](../../../../labs/m1-w2-capacity-capture/README.md) and its `record-summary.json`.
  - One row and one plain-terms bullet in [`m1-discrete-attention.md`](../../../../research/notebook/m1-discrete-attention.md).

## Measurements

- **Gate R.** The imported Q4 on today's device code contains the sealed Q4 DEV evaluation (`4fff1d3c…`) exactly: 14,287 rows, 0 mismatches, in 7.7 s.
- **Capture checks.**
  - Root round-trip 14,287 / 14,287.
  - Determinism 1,024 / 1,024.
  - No ABSENT block.
  - TRAIN: 16,404 batches, 1,049,856 rows, final `next_target` 1,049,856, 480 s.
- **Result: H_mixed** (precedence 3).
  - **Top-1 counts:** T 3,414, G 6,300, U 1,226, W 149.
  - **Paired discordance (b, c):** U (201, 2,389), G (3,729, 843), W (61, 3,326).
  - **Floors:** M + F ≈ 6,435 and W + H ≈ 265.
  - **Evaluability:** coverage 14,175 (bar 12,859) and 41,806 codes with count ≥ 2 (bar 1,024).
- **Admission-matched (1,920 admitted rows), top-1:** code table 1,138, bigram 1,482, symmetry-pooled 988, unigram 754, writer 149.
- **Consumer verifier** (`db02488e…`, runtime mode): `VERIFIED_RUNTIME_EVIDENCE`, `runtime_verdict_authorized: true`, H_mixed. Record `69717971…`, manifest `238dde67…`. The record is in the HF store at `results/claude/w2-capacity-2026-10-25/`.
- **Cost.** One RTX 5090 on Vast `vast-57603275`, 1,570 s at $1.106/h, about $0.48 for both runs.

## Checks run at the head

- **`cargo test --locked -p uor-r4-discrete --lib`:** 0 failed. The W2 tests (34) pass with and without `UOR_W2_Q4`. The real-file importer test runs with no skip.
- **The stub-nvcc `cargo check --offline --features cuda -p uor-r4-discrete --all-targets`** is clean. The real nvcc 12.8 build on the 5090 succeeded at `d186f5f54` and `db25268ba`.
- **`uor-harness loop pr-check --local`:** banned-presence PASS. The only FAIL before this commit was `report`.
- **D67 pre-run review:** three fresh reviewers, each PASS-WITH-NITS, logged on #2845 (comment 6392900474). The fixes landed in `f0b69fda8`.

## Skills used

None.

## Still open

- **Code-utilization diagnostic.** This is the sheet's next step on H_mixed. It is not registered yet; it would go on a new item.
- **Verifier input path.** The verifier hard-codes its corpus path under the retired `uor-r4-local` folder. I created a symlink there only to verify, and it should be replaced by a flag on the verifier's next revision.

## Plain summary

The old Q4 model was loaded into today's code and proved to behave exactly as it did on 15 October. It was then asked one question: does the model's internal read code already know the next word better than the model's own output says?

It does. A simple table keyed by that code gets 3,414 right, against the model's 149. But it does not beat a plain table keyed by the previous word (6,300). So the read is useful but not exceptional. The model loses what it carries in two places. The shortlist (admission) excludes the right word in most rows the table gets right: at least 2,276 of its 3,414. And when the right word is on the shortlist, the output map still misses it in at least 989 of the 1,138 rows the restricted table gets. The registered next step is a code-utilization diagnostic, not a new learning rule.
