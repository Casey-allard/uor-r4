# W2 code-utilization diagnostic: QA A1a, QB B2, QC C1

Item `m1-w2-code-utilization` (#3179). Lab: claude. Order: [claude-95](../../loop/tasks/m1-w2-code-utilization/claude-95/ORDER.md). Sheet: [`m1-w2-code-utilization-2026-10-25.md`](../../harness/prep/m1-w2-code-utilization-2026-10-25.md), FIT in round 2 by a non-author fresh-context reviewer.

**Result.** All three registered questions resolve. No precedence-0 fault, and none of the three underpowered.
- **QA = A1a.** A rule keyed on the read code would admit the target's cell far more often than the writer's address key. That holds even against a fixed list of the most frequent cells.
- **QB = B2.** The registered data-scaling projection of the code table (3,552) is below W2's H_law line (about 6,435), so the full-TRAIN GPU capture (G1) is declined.
- **QC = C1.** A table keyed by the code and the previous token together beats the previous-token table, so the code carries next-token information the last token does not.

**Scope.** One checkpoint (seed-1 Q4, epoch 407), one DEV panel (14,287 rows), the model-matched TRAIN stream (1,049,856 targets).
- These are host post-processing measurements of the stored W2 capture.
- The count tables are diagnostic comparators, not models. The joint table's 7,262 says what the code and the query token carry together; it is not a model result and not an M1 claim.
- No GPU ran, and no model code changed.

## The numbers

| Question | Comparison (DEV, n = 14,287) | Count | Paired (b, c) | Band |
|---|---|---|---|---|
| QA | A_T (code-count admission) against A_W (writer) | 9,064 against 1,920 | (307, 7,451) | A1a |
| QA | A_T against A_U (fixed list of the most frequent cells) | 9,064 against 5,935 | (882, 4,011) | (A_T beats A_U) |
| QB | T(N_0) against T(N_1), stride subsamples | 3,414 against 3,345 | (235, 304) | B2: d_T = 69, T_proj = 3,552 |
| QC | J (code × query table) against G (query bigram) | 7,262 against 6,300 | (571, 1,533) | C1 |

Here b counts rows where the second arm hits and the first misses, and c the reverse. "Beats" is W2's paired floor in squares: c > b and (c − b)² > max(576, 4(b + c)).

### QA: admission

- **The two arms.** The writer admits the target's cell on 1,920 rows, while a code-count admission of the same size admits it on 9,064. A_G (the bigram's cells) admits it on 12,214.
- **The ceiling.** The DEV oracle A_Ω, the best any code-keyed admission could do on this panel, is 13,726.
- **The flag fires.** A_U (5,935) beats A_W (1,920). The trained address key admits fewer target cells than a fixed list of the 8 most frequent cells. Per the sheet, this is an address-objective finding for #2834's decision review, after the instrument-versus-mechanism classification. It matches the training history in the rung-0 admission sheet: target-admitted rows fell from 5,680 at baseline to 1,920 at Q4.
- **Load caveat.** The sets are not load-matched: a mass-ranked set admits larger cells. The number of candidate tokens in the admitted cells, per row:

  | Set | Minimum | Lower quartile | Median | Upper quartile | Maximum |
  |---|---|---|---|---|---|
  | Writer S_r | 8 | 16 | 255 | 261 | 508 |
  | A_T | 16 | 972 | 988 | 1,225 | 1,955 |
  | A_U | 1,225 | 1,225 | 1,225 | 1,225 | 1,225 |
  | A_Ω | 256 | 982 | 1,218 | 1,225 | 1,724 |

  So part of A_T's gain is that it hands the output map about four times as many candidates as the writer's admission does.
- **Exact split of the code table's hits.** Of T's 3,414 hits, 798 fall on rows the writer admitted and 2,616 on rows it did not. This replaces W2's bound of at least 2,276.
- **Cell split.**
  - The writer's top-1 is in the target's cell on 310 rows, and right on 149.
  - The code table is in the right cell on 3,671 rows, and right on 3,414.

### QB: headroom

| TRAIN rows (stride subsample) | T | G |
|---|---|---|
| 65,616 | 2,785 | 6,264 |
| 131,232 | 2,981 | 6,298 |
| 262,464 | 3,176 | 6,309 |
| 524,928 | 3,345 | 6,288 |
| 1,049,856 | 3,414 | 6,300 |

- **The trend.** T's gain per doubling shrinks: 196, 195, 169, then 69, while G is flat near 6,300.
- **The projection.** The last gain does beat the paired floor, so d_T = 69, and T_proj = 3,414 + 2 · 69 = 3,552. That is far below W2's H_law line, about 6,435 (B1 needed T(N_1) ≤ 1,903). So G1, the full-TRAIN capture, is **declined**.
- **What it implies.** Repairs that keep the read fixed are not expected to lift the writer past about 3,552. That is a measurement, not a proof: the DEV oracle Ω_code is 8,182, which is above 6,300, so the proof flag does not fire.
- **The prefixes agree.** The first-row prefixes of the same sizes, reported only as descriptive figures, give T 2,803, 2,970, 3,160 and 3,288.

### QC: beyond the last token

- **The result.** J beats G: 7,262 against 6,300, with (b, c) = (571, 1,533).
- **The coverage guard holds.** 11,727 DEV rows (82.1%) have a seen joint key, well above the guard of 1,429.
- **Under the stricter rule,** using the joint key only when it was seen at least twice, J_m2 is 7,251.

Per the sheet, C1 is one input favouring a read that keeps both the code and the query token's identity; J is the reference number for that card. The descriptive arms show why the code adds and does not replace:
- **Query determinacy.** On TRAIN, 443,216 rows (42.2%) have their code's most common query token; on DEV, 9,696 (67.9%). The DEV figure is higher because many DEV codes occur on a single row (4,020 single-row code keys), and a single-row key trivially has one query token. On TRAIN, the code mixes query tokens rather than preserving the last token.
- **At a matched key count, the query token carries far more.**
  - At 12 bits, the code with one block dropped scores 2,333, 2,433, 2,594 and 2,413 (dropping block 0, 1, 2 or 3). The query token, also 4,096 keys, scores 6,300.
  - At 8 bits, the best block pair (blocks 0 and 1) scores 1,816, against 3,564 for the query token's own cell.

## Descriptive arms

| Quantity | Value |
|---|---|
| Distinct codes, TRAIN | 52,450 of 65,536 (80.0%) |
| Distinct codes, DEV | 6,235 |
| DEV rows whose code is unseen in TRAIN | 112 |
| Roots used per block | 16, 16, 16, 16 |
| Collision effective code count N₀² / Σ n(c)² | 1,102,197,620,736 / 526,975,734 (about 2,092) |
| DEV oracle top-1, Ω: code / query / joint | 8,182 / 6,759 / 12,191 |
| Single-row DEV keys: code / query / joint | 4,020 / 140 / 8,518 |
| TRAIN leave-one-out top-1: U / T / G / J | 92,195 / 245,540 / 492,570 / 538,071 (8.8% / 23.4% / 46.9% / 51.3%) |
| TRAIN resubstitution top-1: U / T / G / J | 92,195 / 297,925 / 493,985 / 711,581 |
| Collision agreement (two rows sharing a key share a target): unigram / code / joint / query | 1.9% / 10.4% / 25.7% / 36.3% (exact fractions in the record) |

- **The code is used, but unevenly.** All 16 roots appear in every block and 80% of the 65,536 codes occur. But the collision count shows the TRAIN mass behaves like only about 2,092 equally used codes.
- **Two exact information forms rank the same way.** The leave-one-out top-1 and the collision agreement both put the code's target information above the unigram's and below the query token's, with the joint key between, or above, depending on the form.
- **Arm R is NOT_RUN** and was not attempted: the lab searched the HF store's `local-archive/2026-10-20` file list and `archive/storage/` for the Q1–Q3 checkpoints (`a5a22dce…`, `dde889aa…`, `102beed4…`) and found none. The tool does not probe for them. Per the sheet, nothing substitutes.

## What it unlocks (per the sheet; the decisions themselves are decision reviews)

- **A1a.** `m1-address-key-full-vector` (#2834) is the next M1 mechanism unit.
  - Its zero-training reference is A_T = 9,064, and its ceiling on this panel is A_Ω = 13,726.
  - It must also beat the fixed list A_U = 5,935, which the trained key loses to.
  - It should report candidate load beside admission.
- **B2.** G1 is declined. The data-scaling curve is not a reason to spend GPU on the full TRAIN.
- **C1.** One input for a read-change card: J = 7,262 is the reference for a read that keeps both the code and the query token's identity.
- **Stage T's gate.** It ("only warranted if W2 blames the learning law (H_law)", #2836) is not changed here. These bands are inputs a decision review would weigh.
- **Not changed.** W2's H_mixed band stands, and W4 stays untriggered.

## Instrument

The tool is the standalone crate [`instrument/`](instrument/). It has a detached `[workspace]` and a read-only path dependency on `uor-r4-discrete`, built without `cuda`. It uses exact integers throughout.

**Precedence 0** recomputes W2's numbers three ways:
- with W2's own public functions (`build_tables`, `score_dev`, `aggregate`, `score_admitted`, `admission_matched`);
- against the sealed `record.json`;
- with the instrument's own tables, compared row by row with W2's scoring for the table, bigram, unigram, address-only, interior-only and target-admitted arms.

All 11 checks and all 6 row comparisons pass, with 0 differing rows:
- T 3,414, G 6,300, U 1,226, W 149;
- A_W 1,920, restricted table 1,138, restricted bigram 1,482;
- coverage 14,175 and codes with count ≥ 2 41,806;
- address-only 1,675 and interior-only 1,628.

**Other checks:**
- Σk_r = 114,296, and F² = 18,288.
- The count tables are conserved on every subsample.
- Every leave-one-out hit implies a resubstitution hit, and T ≤ Ω_code.
- A_T ≤ A_Ω holds within every code whose rows share one k_r.
- No DEV row repeats an admitted cell.
- The run is computed twice and must be byte-identical.

**What the unit tests pin:**
- the floor boundary and the precedence-1 rule;
- every band, including A1u and the decisive-pair scope;
- the table tie rule and leave-one-out, including the edge cases;
- the cell ranking (`ranked_for`) and the row-matched admission size (`in_top`);
- the joint-key fallback (`joint_prediction`);
- the spread and projection rules.

A planted fault, `in_top` taking k − 1 cells, makes `in_top_respects_the_row_k` fail.

## Artifacts

In the HF store `caseyallard/uor-r4-store`, under `results/claude/w2-utilization-2026-10-25/`:

| File | sha256 |
|---|---|
| `record.json` (`uor-r4.w2-code-utilization/1`) | `d9aa6cd1a9a46876d4e50f1143d481e810fdcfb216beeccf6422ce1b7d70db90` |

The record carries per-row DEV hit flags for every arm and the `uor-r4-discrete` commit it was built against (`9d509310be5aa8bb59e8909cefa8a2d24e173789`, from `git rev-parse HEAD`). It replaces the first run's `a0af69ae…`. Every number is identical; only the recorded commit and arm R's reason text differ, the latter reworded after review to say R was not attempted.

Inputs, authenticated before any count:
- `run-b.tar` `6aa0a766…` was authenticated with `shasum -a 256` before extraction, outside the tool. The tool then pins what it reads: `w2-out-b/train-stream.bin` `70a58767…` (4,199,424 B), `w2-out-b/query-sidecar.bin` `6d5d4f5d…` (2,099,712 B) and `w2-out-b/capture.json` `d22a4eb2…`.
- W2's `record.json` `69717971…`, pinned by the tool.
- The `w2-inputs/2026-10-25/` corpora and `SHA256SUMS` are not read by this diagnostic.

## Check it yourself

```sh
uor-store get results/claude/w2-capacity-2026-10-25/ <dl>   # run-b.tar, record.json
tar -xf <dl>/results/claude/w2-capacity-2026-10-25/run-b.tar -C <cap> w2-out-b
cd docs/labs/m1-w2-code-utilization/instrument
CARGO_TARGET_DIR=<target> ~/.cargo/bin/cargo test --release        # unit tests
UOR_W2_CAPTURE=<cap>/w2-out-b UOR_W2_RECORD=<dl>/results/claude/w2-capacity-2026-10-25/record.json \
  CARGO_TARGET_DIR=<target> ~/.cargo/bin/cargo test --release --test real_capture -- --nocapture
<target>/release/m1-w2-code-utilization-instrument --capture <cap>/w2-out-b \
  --record <dl>/results/claude/w2-capacity-2026-10-25/record.json --out <out> --discrete-commit <40-hex>
```

## Limits

- **One artifact.** One checkpoint, one seed, one panel. The bands classify this artifact, not the mechanism (CHARTER §4.1).
- **QA's sets are not load-matched.** See the candidate-load table. A_T's gain is partly a larger candidate set.
- **QB's projection is a pre-registered rule,** two more doublings of the last gain, not a fitted law.
- **C1 says the joint key carries more than the query token on this panel.** It does not say a model can realize J, and J is a count table over an unbounded key space.
- **Arm R is NOT_RUN** (missing input), so route-learning coverage is not measured here. The rung-0 sheet's earlier receipt reading (16 of 20 route slots accepted) is unchanged.
