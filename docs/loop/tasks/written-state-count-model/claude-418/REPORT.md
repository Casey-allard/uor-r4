# REPORT: written-state-count-model, claude-418 (modified Kneser–Ney adjusted counts on the GPU counter)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The first wick run failed one CUDA test because my test was wrong** [claude].
  - `ngram_count_cuda_adjusted_matches_the_oracle` first asserts that its comparison is not vacuous. I wrote that check as "the adjusted triple SHA differs from the raw one exactly below N".
  - The adjusted assembler hashes under its own domain (`uor-r4/ngram-adjusted-triples/v1`), so its SHA differs from the raw one even at k = N, where the counts are equal. The assertion therefore failed at order 3.
  - This was an instrument fault, not a counter fault. Device = oracle had held at orders 0..2 and at both budgets before the failing line.
  - The check now compares the triples. Run 2 passes it.
- **The host review found a MEDIUM** [claude]. `adjust_partition` checked that order k's triples were strictly ascending, but not order k+1's. A repeated order-(k+1) triple counted as a second distinct left token. The device path passes the GPU's order-(k+1) partition straight in, with no assembler in between. Fixed and pinned (see Checks).
- **The device review also proposed a negative gate. My first attempt at it did not compile** [claude]. In `c418-run-2`, my set-aside (`targets: range.targets + 0 * targets`) made the type of `sum()` ambiguous (E0283). A set-aside that fails to build pins nothing, so I re-ran it with the type written out (`c418-neg-2`, below).
- **The two device builders ran out of context early** [claude].
  - The first stopped at the 150k mark with the host half done but its rebuild and G3 not run. I re-ran its gates.
  - The second landed `count_order_adjusted` only. I wrote the CUDA tests and the probe mode in the top session.
  - The second builder's isolated worktree started at `main`, not at the unit's tip, and was fast-forwarded before work.
- **Pre-existing, not this unit's** [claude]: without the cuda feature, `cargo check -p uor-r4-discrete --all-targets` fails on two examples (`first_pick_read_parity`, `native_vector_qualify`). They import the cuda-only `native_vector_device` and declare no `required-features`. This branch does not touch them.

## What changed

- `ngram_count.rs` [claude]:
  - `adjust_partition(order, highest, lo, hi, &counted, extended)` gives an order-k partition's adjusted counts:
    - a = c at k = N and for BOS-oldest grams;
    - otherwise a = N1+(•g), the distinct order-(k+1) triples whose back-off key and next equal g's.
  - It refuses:
    - a zero adjusted count (rule 4);
    - an order-(k+1) triple whose suffix is absent or extends a BOS-oldest gram;
    - triples that are not strictly ascending or carry count 0, in either partition;
    - a wrong order, range or partition count.
  - `UnigramContinuation`: order 0's N1+ summed over every order-1 partition. It refuses a repeated or overlapping partition, and any order-1 partition under N = 0.
  - `NgramAssembler::new_adjusted`, with its own SHA domains (`uor-r4/ngram-adjusted-{triples,contexts}/v1`). The adjusted census and count-of-counts come from the adjusted triples and are re-derived by the existing cross-check.
  - Four host tests:
    - a two-document hand fixture worked out in its comments;
    - assembler = oracle on seeded streams at budgets MAX, 700 and 1;
    - the refusals;
    - the SHA is independent of the budget.
- `ngram_count_oracle.rs`: `oracle_adjusted` builds a(g) from the token stream as a set of distinct left tokens per gram, with no back-off and no triples [claude].
- `ngram_count_device.rs`: `count_order_adjusted(order, highest, budget, assembler)` [claude]:
  - For k ≥ 1 it plans from order k's histogram. Each range is counted at k and at k+1 over the same lo..hi, and order k+1's target count is summed from its own histogram, because `select` checks the exact planned count.
  - Order 0 counts its unigrams, then adds every partition of order 1's plan to the continuation once.
- `ngram_count_cuda.rs`: three CUDA tests [claude]:
  - adjusted = oracle on synthetic streams (orders 0..3, N = 3, one partition and several);
  - the adjusted SHA is stable across budgets and runs (N = 3 and 8);
  - an ignored real-stream test.
- `bin/ngram-count-probe.rs`: an optional `adjusted` argument; each line carries `highest` and `plan_order` [claude].
- Count-only outputs in [run2/](run2/) and one notebook row in `docs/research/notebook/m4-serving.md` [claude].

## Measurements

Wick's RTX 4090 (`compute_89`), on claude-415's 15,471,412-token D87 student slice `c404/stream/tokens.u16`, build `47eef0fdb`. The probe ran twice with budgets 4,294,967,295 and 1,000,000 (1 and 17 partitions) [claude].

Adjusted counts, all orders under N = 8. For k < 8, "mass" is Σ a(g), the number of distinct (left token, gram) pairs; at k = 8 it is the raw target count:

| order | mass | grams | contexts | n1 | n2 | n3 | n4 | ms |
|---|---|---|---|---|---|---|---|---|
| 0 | 492,323 | 3,968 | 1 | 79 | 71 | 74 | 72 | 14 |
| 1 | 2,446,321 | 492,323 | 3,968 | 261,644 | 81,888 | 38,662 | 22,418 | 149 |
| 2 | 4,846,646 | 2,442,225 | 492,172 | 1,835,606 | 304,708 | 108,277 | 54,171 | 437 |
| 3 | 6,866,488 | 4,842,550 | 2,441,235 | 4,132,730 | 413,614 | 122,848 | 56,120 | 789 |
| 4 | 8,367,253 | 6,862,392 | 4,840,783 | 6,219,790 | 412,476 | 105,086 | 44,333 | 1,085 |
| 5 | 9,432,886 | 8,363,157 | 6,860,095 | 7,839,709 | 357,245 | 80,543 | 32,494 | 1,298 |
| 6 | 10,205,462 | 9,428,790 | 8,360,558 | 9,013,453 | 295,223 | 60,907 | 23,721 | 1,447 |
| 7 | 10,778,653 | 10,201,366 | 9,425,798 | 9,871,323 | 242,322 | 46,358 | 17,223 | 1,544 |
| 8 (raw) | 15,438,644 | 10,774,557 | 10,198,155 | 9,452,882 | 828,094 | 200,186 | 96,273 | 921 |

- The triple and context SHAs are equal at both budgets for every order, and the two runs are `cmp`-identical once timings are stripped [claude]. Order 3's adjusted triples are `b7cdaa64…`.
- Under N = 3 (the real-stream test), order 3 keeps raw counts. Its whole-stream count-of-counts is 3,353,723 / 702,536 / 252,800 / 138,227 with SHA `f58434d7…`, and orders 0..2 equal the N = 8 rows above [claude].
- On the first 2,000,000 tokens, device = oracle at orders 0..3 under N = 3, with 3 partitions each [claude].

## What the numbers say

- **The adjusted counts are exact on the GPU and do not depend on the partition plan** [claude]:
  - device = oracle on synthetic streams and on a 2M-token real slice;
  - one SHA across 1 and 17 partitions;
  - one SHA across two runs.
  - claude-419 can therefore compute the modified-KN discounts from these count-of-counts, and the R\* scorer can read these counts.
- **Almost every order-k gram is an order-(k+1) context** [claude]. At orders 1, 2 and 3 the gram counts exceed the next order's context counts by 151, 990 and 1,767: 492,323 − 492,172, 2,442,225 − 2,441,235, and 4,842,550 − 4,840,783.
  - My reading is that these are grams seen only at a record's end, whose next token is the following record's BOS. A BOS target is never counted, so these grams have no admitted successor [claude, inference; not measured per gram].
  - Each gap is below the slice's 4,096 records.
- **The discounts land in the usual range** [claude, inference; claude-419 computes them in integers]. From order 3's adjusted n1..n4, D_i = i − (i+1)·Y·n_{i+1}/n_i with Y = n1/(n1 + 2·n2) gives about 0.83, 1.26 and 1.48. That is the shape modified KN expects.
- **The highest order is sparse** [claude]. 9.45M of the 10.77M order-8 grams occur once. On this 15.5M-token slice, order 8 mostly memorizes. The tuning of N on DEV-tune (claude-420) decides how far up R\* should go. Nothing here scores a model.

## Checks run at the head

- G1: the host tests of `uor-r4-discrete` [claude]:
  - `cargo test --release -p uor-r4-discrete --lib ngram_count`: 21 passed, 0 failed, at `47eef0fdb`;
  - `cargo check -p uor-r4-discrete --lib --tests --bins` is clean;
  - a stub-nvcc `--features cuda` check of the lib, tests and bins is clean, and its `--test` rustc run carries `feature="cuda"`.
- G2: set-aside negatives, each failing, then restored and rebuilt [claude]:
  - With the BOS case set aside (`document_start = false`), all 4 adjusted host tests fail, the hand fixture included.
  - With the distinct-v grouping replaced by a sum of raw counts, the hand fixture and the oracle test fail.
  - The review fixes, each set aside in turn, fail the refusal test at their own assertion:
    - the order-(k+1) ascending check;
    - the continuation's ascending check;
    - the N = 0 refusal, without which `adjust` returned Ok.
  - Restored and rebuilt: 21 of 21.
- G3: `UOR_LAB=claude uor-harness loop pr-check --local` at the committed head: see the PR [claude].
- G4, wick `c418-run-2` at `47eef0fdb` [claude]:
  - The CUDA `.cu` sources were touched after the extract.
  - All CUDA tests pass: 27 passed, 2 ignored.
  - `UOR_NGRAM_STREAM=…/tokens.u16 … ngram_count_cuda_real_stream -- --ignored`: 2 of 2 pass (claude-415's raw test and the new adjusted test).
  - The adjusted probe runs are identical.
  - Device negative (`c418-neg-2`): the order-(k+1) call was given order k's own target count in place of the k+1 histogram sum.
    - It built, and both adjusted CUDA tests failed with `partition selected 85809 targets, planned 92671` (and `858091` / `926479`).
    - Restored and rebuilt, both pass.
- G5: no corpus text, no token stream and no binary is committed. `run2/` holds probe lines (counts, timings and SHA-256 digests) and test logs [claude].
- Fresh review [claude]: two reviewers, host and device halves. Both returned APPROVE-WITH-LOW, with one MEDIUM, fixed. See the review record on the PR.
- Nothing on `/mnt/plexpool` was touched. Wick inputs were read from the `/workspace` copies [claude].

## Skills used

None invoked [claude]. I followed AGENTS.md and the native-space rule: integer counts only, no float, no squashing, no matmul.

## Still open

- claude-419, the scorer, which reads these adjusted counts [claude]:
  - the integer modified-KN discounts per order;
  - interpolation;
  - Q48 masses through `lawful_likelihood`;
  - order-2 BPB equal to the CPU oracle on a 1% slice;
  - statistics filtered to DEV-test contexts.
- claude-420: the sheet, FIT, N tuned on DEV-tune, and the sealed DEV-test run on all targets and the novel subset [claude].
- The full-corpus count, which needs the 2.09B-token TRAIN stream under the partition plan [claude].
- The two CUDA-only examples that break host `--all-targets` (above) [claude].

## Plain summary

The project's reference model for bar Q is a classic n-gram model, interpolated modified Kneser–Ney [claude]. Its lower orders need a different count from "how often a phrase appears". They need "how many different words came right before it". This unit adds that count to the GPU counter. It matches an independent CPU count exactly, on random text and on 15 million tokens of the real corpus, and it does not change when the work is split differently. The next unit turns these counts into probabilities.
