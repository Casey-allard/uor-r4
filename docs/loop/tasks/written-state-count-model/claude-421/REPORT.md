# REPORT: written-state-count-model, claude-421 (the GPU modified Kneser–Ney scorer, the query filter and the bits instrument)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The first scale run refused the real data** [claude]. On the whole TRAIN stream, order 0's continuation spectrum is n1..n4 = [7, 6, 11, 11]. Only 4,079 unigrams exist, and nearly every one follows many distinct tokens. Chen–Goodman then gives D2 = −7,407,236,229,228 / 2^48 (about −0.026), and `KnTables::new` refused it (wick run 2 at c095c0413). The GPU path had done its part: the counts were done in about 28 s. The refusal came from claude-419's discount rule meeting a degenerate spectrum it had never seen.
- **A change to R\*'s definition, from my own decision review** [claude]. An order whose spectrum has a zero n_i, or a Chen–Goodman estimate outside [0, i], now takes KenLM's `--discount_fallback` default, (1/2, 1, 3/2), for all three discounts. That is what KenLM's `lmplz` does on such an order with that flag.
  - `KnTables` and the independent `KnOracle` each apply it on their own counts.
  - `KnDiscounts::from_count_of_counts` keeps its strict refusal.
  - The instrument's JSON records each order's discounts and its `discount_fallback` flag.
  - On whole TRAIN at N = 2 only order 0 falls back; orders 1 and 2 keep their Chen–Goodman estimates.
  - **claude-422's sheet must name this rule as part of R\*.**
- **Four instrument errors on wick, each fixed before the run reported here** [claude]:
  - nvcc's `__int128` division calls a routine holding `.f32`, which the integer PTX admission refuses. The kernel now divides by hand: one native 64-bit divide, then 64 shift-and-subtract steps.
  - The standalone instrument crate lacked the workspace's `[patch.crates-io]` for the vendored cudarc, so it did not build. It now carries the same patch as the k16 instrument.
  - `/usr/bin/time` does not exist on wick, so run 1's real-slice test and scale run never ran (a pipe hid the failure). Runs 2 and 3 time with `date +%s`.
  - My first bits negative rebuilt nothing (0.36 s), so it ran the old binary. The rerun builds into a fresh target folder.
- **The ORDER's bits negative could not show anything** [claude]. Bits of mass + 1 equal bits of mass exactly, at 1,984,097,051 Q12 over 107,257 targets. That is expected: +1 on a mass near 2^40 moves −log2 by about 10^-12 bits, far below `bits_q12`'s step of 1/4096 bit. I replaced it with bits of mass / 2. That moved the total by exactly 4,096 × 107,257 = 439,324,672 Q12, while `masses_sha256` and the host check were unchanged.
- **Three review findings that were real gaps** [claude, from the reviews]:
  - The CUDA tests ran only N ≤ 4, which never touches the key's low word or the slot that straddles bit 64. A test at N = 5..10 was added. Dropping the straddle carry fails it.
  - The class file was checked only against the query prefix. It must now cover the whole query.
  - The oracle's fallback for an all-nonzero, out-of-range spectrum was untested. Its rule is now checked directly against the host's on six spectra.

## What changed

- `crates/uor-r4-discrete/src/kn_filter.rs` (new) [claude]:
  - `KnQuery::from_stream`: each order's sorted context keys and (key, target) pairs of the query's non-BOS targets.
  - `KnFilter::keep` / `finish`: keep only those records, in canonical order. Order 0's per-partition contexts are merged.
- `ngram_count_device.rs` [claude]: `count_order_adjusted_with`, a sink form. `count_order_adjusted` calls it with `NgramAssembler::push`, so the whole order is still hashed.
- `kn_score.cu`, `kn_score_device.rs` (new), `build.rs` [claude]: the scorer is built by nvcc into its own admitted cubin, never NVRTC. `KnScorer::new` uploads one `KnTables`; `score` returns every non-BOS target's Q48 mass in stream order.
- `kn_score.rs` [claude]: `order_tables`, `discount_fallback`, `KnDiscounts::FALLBACK` and `with_fallback`.
- `kn_score_oracle.rs` [claude]: `oracle_discounts` with the same rule, sharing no code with `KnTables`.
- `kn_score_cuda.rs` (new) [claude], all compared with `KnTables::scored_masses`:
  - GPU = host at N = 0..4 on three queries (train, another seed, BOS in every slot);
  - N = 5..10 on long-copy streams;
  - fallback orders on a 40-id stream;
  - empty and BOS-only queries;
  - an ignored real 1% slice against `KnOracle`.
- `docs/labs/kn-reference/instrument` (new, standalone crate) [claude]:
  - Arguments: TRAIN, QUERY, ID-BYTES, CLASSES, N, BUDGET, OUT.json, plus `--per-mille`, `--check-host` and `--sealed-run`.
  - It asserts at compile time that both `PROBABILITY_ONE`s are 2^48.
  - It refuses the sealed DEV-test stream (by sha256) without `--sealed-run`.
  - Its JSON holds the inputs' sha256, per-order digests, kept counts and discounts, `masses_sha256`, and Q12 bits, bytes and micro-BPB for all targets and for class 1 (novel). It holds no float.
- Committed records [claude]: `run1/*.json` (counts and digests only) and `logs/`. No stream, mask, table or corpus text is committed.

## Measurements

All on wick's RTX 4090, run 3 at f9a51d738, with DEV-tune as the query [claude]. Logs are in `logs/`.

| Check | Result |
|---|---|
| CUDA tests | 10 / 10 pass, 3 ignored (real-stream tests) |
| Real 1% slice: GPU path = `KnOracle`, N = 2 | 107,257 / 107,257 targets equal; TRAIN slice 20,914,089 tokens; 17 s |
| Instrument 1% ×2, `--check-host` | host = GPU at 107,257 targets; JSON `cmp`-identical |
| Seal guard on DEV-test without `--sealed-run` | rc 1, no JSON written |
| Whole TRAIN (2,091,560,001 tokens), N = 2, all of DEV-tune | host = GPU at all 10,807,417 targets; 35 s wall; 8,056 MiB peak device memory; 17 partitions per order |
| Order 0 / 1 / 2 grams | 4,079 / 3,287,899 / 52,997,638; kept triples 3,921 / 401,191 / 1,720,333 |
| Discounts | order 0 fallback; orders 1 and 2 Chen–Goodman |
| BPB at N = 2 (engineering check only) | all 2.004736 (21,928,475 bytes); novel 2.194312 (7,566,174 targets, 15,995,822 bytes) |

Negatives, each failing and then restored [claude]:

| Negative | Where |
|---|---|
| straddle carry dropped → the high-orders test fails | run 3 |
| γ skipped at k = 1 → GPU = host fails | c421-neg-1, before the high-order tests |
| the filter drops (key, target) pairs → filtered = unfiltered fails | builder A, host |
| strict discounts in `KnTables::new` → the fallback test fails | laptop |
| the oracle's range test dropped → the spectrum check fails | laptop |
| bits of mass / 2 → +439,324,672 Q12 exactly | run 3 |

## What the numbers say

- The GPU path is exact: counter, filter and scorer together equal an independent CPU oracle on real data. It is fast enough that claude-422 can tune N on DEV-tune across several orders in minutes [claude].
- At N = 2, R\* costs about 2.0 bits per byte on DEV-tune and 2.19 on novel targets [claude]. N = 2 is not the bar's N, and DEV-tune is not the bar's split.

## Checks run at the head

- Laptop host tests: `kn_` 9 / 9. The 30 / 30 counter and driver-source tests ran at c095c0413; the code after that touches `kn_score*.rs` only. The stub-nvcc cuda check of the lib, tests and bins is clean. The instrument's 4 tests pass [claude].
- After wick run 3 (f9a51d738), the only commits are the oracle and test review fixes, a doc comment and one test-line destructure. No GPU, filter or instrument code changed, so run 3's measurements stand at the head [claude].
- `uor-harness loop pr-check --local`: verdict PASS. Its advisories are D76 only: scope, read path and filemap [claude].
- Reviews [claude, delegate:reviewer]:
  - the GPU scorer: APPROVE-WITH-LOW;
  - the filter and instrument: APPROVE-WITH-LOW;
  - the fallback commit: APPROVE-WITH-LOW.

  Each finding was fixed or accepted with a reason in the PR's review record.

## Skills used

None invoked [claude]. I followed AGENTS.md: Rust only, the D87 copy read only, DEV-test sealed, and no corpus data committed.

## Still open

- claude-422: the sheet (naming the discount fallback), FIT, N tuned on DEV-tune, and the sealed DEV-test run on all targets and on the novel subset [claude].
- Wick keeps the streams at `/workspace/out/c420/run1` and this unit's outputs at `/workspace/out/c421` [claude].

## Plain summary

The baseline the model must beat is a standard counting model of text [claude]. This unit makes it run on the GPU and checks that it gives exactly the same answers as a slow, separate reference program. It does, on every one of 107,257 test tokens. Over the whole 2.1-billion-token training set it scores 10.8 million tuning tokens in 35 seconds. One rule had to change: when the training data's count statistics are too lopsided to estimate a correction factor, the baseline uses the standard fallback values that the reference tool KenLM uses. Nothing was scored on the sealed test set.
