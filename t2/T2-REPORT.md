# T2 — Is the gold source position the argmax of the read's score vector?

**Answer: no, not remotely.** On the exact sealed-reference configuration the gold source is
the argmax of the read's score vector in **1.14 %** of last-layer read rows (2.9 % pooled over
every read layer and head) and **0.00 %** of rows once the learned NoRead slot joins the
comparison. That is the pre-registered branch

> **gold is the argmax in fewer than 50 % of items → the failure is in the SCORE
> representation, upstream of everything read-out.**

No read-out, normaliser, temperature or routing change can repair this: the score vector does not
rank the gold source first, and it does not even concentrate on the fact that contains it
(the whole fact span is the argmax in 3.6 % of rows; the read puts 1.2 % of its mass there).

The pre-registered rule was fixed before the dump was read (it is the rule given in the task, and
the analysis below is a direct application of it; the script that computes it was written and the
decision threshold stated before the first analysis run).

---

## 1. What was changed, and where

All changes are on branch `deepseek/t2-argmax` in worktree `~/uor-r4-worktrees/t2-argmax`
(created with `git worktree add … -b deepseek/t2-argmax origin/main`). Nothing was written to
`~/uor-r4` or to any pre-existing worktree or report root.

### Library — `crates/uor-r4-training/src/geometric_stack.rs`

| line | change |
|---|---|
| 548 | `BindingCapture.weights: Option<WeightDumpCapture>` (new optional capture, `None` everywhere else) |
| 556 | `struct WeightDumpCapture { rows, layers }` |
| 2193–2222 | in `read_binding_values`: when the dump is armed, append an **identity value block** (one auxiliary value channel per source position) to the read's value matrix. Exactly the mechanism the existing label mask and span probe use: it shares the read's scores, admission (age, NoRead) and normalisation, never enters the scores, and is removed before `read.out`. Exclusive with the label mask / span probe (refused, not silently mixed). |
| 2253–2268 | in `finish_geometric_read`: narrow the identity block out and keep the requested `(batch, query)` rows per read layer, `[rows, heads, time]` |
| 6593–6662 | new `pub fn StackModel::read_weight_rows(ids, batch, time, rows) -> (Vec<(layer, Tensor)>, Tensor)`. It runs `forward`'s exact path (`layer_range_with_source(..., ReadSource::default(), LatchGates::Soft)`) and returns both the weight rows and the logits of that same forward, so the caller can prove the observation changed nothing. |

A channel `c` of row `t` is `sum_{j<=t} a_j * [j == c] = a_c`, i.e. the read's own softmax
weight on source `c`; `a_c = 0` for `c > t`. This yields the **exact** weight vector, and since
the softmax is strictly monotone in the score, `argmax_j a_j` is the argmax of the score vector,
and `ln a_j` equals the score vector up to one additive constant per row (so every rank, and every
score *gap*, is exact — the additive constant cancels).

### Bench — `crates/uor-r4-training/examples/mqar-bench.rs`

| line | change |
|---|---|
| 10, 26–32 | document the flag |
| 376–389 | `ContextArm::read_weight_rows` default (returns an error for arms without a read) |
| 781–789 | `StackArm::read_weight_rows` → `StackModel::read_weight_rows` |
| 1108, 1133 | `Common.dump_scores: bool`, recorded in `config.json` |
| 1193–1197 | parse `dump_scores=0|1|false|true`, default `false` |
| 1715–1722 | **test-only** completion of a `Common { … }` literal in `#[cfg(test)]` (`pointer`, `dump_scores`). The example's test target did not compile on `origin/main` either (`pointer` was already missing); this repairs it so the tests can be run. No non-test code. |

### Bench facts — `crates/uor-r4-training/examples/mqar_bench_step2/fact.rs`

| line | change |
|---|---|
| 545–575 | `nlet_rule` split into `nlet_match_end` + a thin wrapper (same expression, same early return — behaviour-identical refactor) and `pub(super) rule_source`, which returns the position the R-nlet reference rule copies from |
| 722–930 | `dump_fact_scores`: per held-out item, its first-content predicting row, the gold source, the structural landmarks, the model's own argmax there, the rule's source, and the raw f32 weight rows |
| 1234–1258 | called from `run_fact` **after** the final evaluation (so no tally can move), only when `dump_scores=1` |
| 1304 | `results.read_score_dump` summary in `report.json` |

**Gold definition used.** In `layout=fact` an item's `predict[i]` is the position whose
next-token target is `value[i]`; the row analysed is `predict[first_content]` — the same row the
scored `first_content_piece_accuracy` uses. The gold source is `value_first() + first_content`,
where `value_first() = fact_start + key.len() + gap` is the window position that *holds* the
target piece in the fact. That is the only causal position that carries the target token
(the query's own copy of `value[i]` sits at `predict[i] + 1 > predict[i]`, i.e. in the future), and
it is the position `reference_rule` copies from (`rule_source`) and the one `fact_probe` already
probes (`item.value_first()`).

**Default path unchanged.** `weights` is `None` at every construction site except
`read_weight_rows`, so `read_binding_values` returns its input unchanged and none of the new code
runs; the `nlet_rule` refactor is the same expression on the same inputs. Empirically (see §5)
the instrumented binary at the exact reference configuration reproduced the sealed root's step log
and final numbers exactly, and inside the dump the observed forward's logits were **bit-identical**
to `arm.logits` on the same batch (`max |gap| = 0.0`, not a tolerance).

## 2. Exact commands

Fast replication (run A):

```
/Users/casey.allard/.cache/uor-r4-t2-argmax/release/examples/mqar-bench \
  out=/Users/casey.allard/uor-r4-worktrees/reports/t2-argmax-20261005-1 \
  layout=fact steps=200 batch=4 final_sequences=64 seed=7 probe_steps=none dump_scores=1 \
  tokenizer=/Users/casey.allard/uor-r4-local/inputs/claude-t4-1433-resume/bundle-learned-1/tokenizer.json
```

Exact sealed-reference configuration (run B; `~/uor-r4-worktrees/reports/ro-full-base/attempt.json`
argv, plus only `probe_steps=none` and `dump_scores=1`):

```
/Users/casey.allard/.cache/uor-r4-t2-argmax/release/examples/mqar-bench \
  out=/Users/casey.allard/uor-r4-worktrees/reports/t2-argmax-20261005-2 \
  layout=fact steps=1200 batch=8 final_sequences=256 seed=7 probe_steps=none dump_scores=1 \
  tokenizer=/Users/casey.allard/uor-r4-local/inputs/claude-t4-1433-resume/bundle-learned-1/tokenizer.json
```

Built with `CARGO_TARGET_DIR=/Users/casey.allard/.cache/uor-r4-t2-argmax cargo build --release
--offline -p uor-r4-training --example mqar-bench` (CPU only, no GPU, no network).

Analysis:

```
python3 t2/analyze_read_scores.py ~/uor-r4-worktrees/reports/t2-argmax-20261005-2 \
  --json t2/run2/analysis.json --md t2/run2/analysis.md
```

## 3. Numbers

Run B is the primary result (exact reference configuration, 1536 held-out items, 128 sequences,
6 read layers × 4 heads = 24 read rows per item, context 384). Run A is the cheap replication
(384 items). "Row" = one `(item, layer, head)` read weight vector; `n_candidates` = `query + 1`.

### (a) gold IS the argmax of the score vector

| population | run B (exact cfg) | run A (200 steps) |
|---|---|---|
| all 36 864 / 9 216 read rows pooled | **0.0292** (1075) | **0.0080** (74) |
| last read layer (layer 5), 6 144 / 1 536 rows | **0.0114** (70) | **0.0039** (6) |
| per item, best of the last layer's 4 heads | **0.0417** (64/1536) | **0.0130** (5/384) |
| incl. the NoRead slot (overall argmax) | 0.0077 pooled / **0.0000** last layer | 0.0031 / 0.0020 |
| per layer L0…L5 | .0234 .0011 .0542 .0207 .0641 **.0114** | .0059 .0091 .0111 .0104 .0078 .0039 |
| shortest distance bucket `d16` (last layer) | 0.0342 | 0.0117 |
| `d64` / `d200` (last layer) | 0.0000 / 0.0000 | 0.0000 / 0.0000 |

**Upper bound on any "wrong gold convention" objection:** treating the *entire fact span*
(`fact_start … value_first+len-1`: key, gap and value — every plausible source) as the target,
it is the argmax in **0.0356** of last-layer rows (run B) / **0.0046** (run A); a position within
±1 of the gold in **0.0301** / **0.0156**. No position convention inside the fact reaches 50 %.

### (b) top-3 / top-10

| | run B | run A |
|---|---|---|
| top-3, pooled | 0.0699 | 0.0303 |
| top-3, last layer | 0.0438 | 0.0280 |
| top-10, pooled | 0.1565 | 0.0728 |
| top-10, last layer | 0.1576 | 0.0527 |

### (c) softmax weight (probability) on the gold

| | run B pooled | run B last layer | run A pooled | run A last layer |
|---|---|---|---|---|
| mean | 0.00539 | 0.00465 | 0.00526 | 0.00491 |
| median | 0.00011 | 0.00100 | 0.00038 | 0.00074 |
| p90 | 0.0109 | 0.0123 | 0.0101 | 0.0118 |
| max | 0.782 | 0.244 | 0.899 | 0.301 |
| fraction > 0.1 | 0.0084 | 0.0026 | 0.0077 | 0.0033 |
| fraction < 1/n_candidates | 0.8293 | 0.7308 | 0.8367 | 0.7591 |

The gold carries *less* weight than a uniform draw over the ~244 candidates in ~73–83 % of rows.

### (d) NoRead probability

| | run B | run A |
|---|---|---|
| mean, pooled | 0.623 | 0.406 |
| mean, last layer | 0.598 | 0.317 |
| NoRead is the row's **largest single entry** | 0.845 pooled / **0.993** last layer | 0.770 / 0.790 |
| total source mass `sum_j w_j = 1 − NoRead` | 0.377 (range 2.7e-6 … 0.9996) | 0.594 |

### (e) rank of the gold position

run B, pooled (36 864 rows): `1: 1075, 2: 859, 3: 644, 4–10: 3193, 11–50: 10422, >50: 20671`
run B, last layer (6 144): `1: 70, 2: 100, 3: 99, 4–10: 699, 11–50: 1674, >50: 3502` (57 % below rank 50)
run A, pooled (9 216): `1: 74, 2: 102, 3: 103, 4–10: 392, 11–50: 2418, >50: 6127`
run A, last layer (1 536): `1: 6, 2: 31, 3: 6, 4–10: 38, 11–50: 420, >50: 1035`

### (f) how many candidates compete

| | run B | run A |
|---|---|---|
| nominal row length `query+1` | mean 244.1, range 15–383 | mean 241.9, range 15–383 |
| effective support (`w > 1e-6`) | mean 150.0 pooled / 163.1 last layer | 166.7 / 168.4 |
| participation ratio `(Σw)²/Σw²` | mean 38.5 pooled / 63.0 last layer | 40.6 / 68.3 |
| largest single weight in the row | mean 0.068 pooled / 0.041 last layer | 0.083 / 0.064 |

The read is **diffuse**, not peaked: its biggest weight averages 4–7 % and it spreads mass over
150–170 positions. Last-layer span mass (run B): last 16 positions 0.143, query span 0.051,
**whole fact span 0.0117**, fact value span 0.0051, gold position 0.0047, position 0 0.0006.

### Where the read actually looks (last layer, run B, argmax identity)

`other` 3539 (57.6 %) · `query span` 1452 (23.6 %) · `query row's own position` 653 (10.6 %) ·
`query key last` 340 (5.5 %) · `gold` 70 (1.1 %) · `fact key last` 34 · `previous position` 27 ·
`fact key span` 23 · other fact value piece 6.

### Score shape in distance (run B, last read layer)

`ln w[t−d] − ln w[t]` is exactly `s[t−d] − s[t]` (the per-row additive constant of the score
vector cancels). Pooled over heads: `d=0: 0.00, d=1: +0.51, d=8: +0.16, d=32: −1.81, d=64: −4.46,
d=128: −9.75, d=256: −20.41` nats. Per head: head 0 is a steep recency head (−31.4 nats at
d=128), head 3 is nearly flat (+0.79 at d=128 — effectively uniform), head 1 is short-range
peaked, head 2 mildly decaying. **No head shows a positive score at the distance of the matching
key**: the score is a smooth function of *distance* with no content-matching structure, i.e. the
score is a recency prior plus a nearly flat geometry term.

### Reference rule (non-learned), same items

| | run B | run A |
|---|---|---|
| rule's implied source == gold | 0.4505 | 0.4557 |
| rule first-content hit | 0.6263 | 0.6276 |
| model first-content hit (from the dump) | 0.0312 | 0.0417 |
| rule's source == model's last-layer argmax | 0.0117 | 0.0026 |
| rule's source == model's argmax in *any* layer | 0.2272 | 0.0234 |

The rule locates the gold source in 45 % of items (and its copy is right in 63 %); the read's
scores agree with the rule's source in ~1 % of last-layer rows. The score representation is not a
noisy version of the classical rule — it is unrelated to it.

## 4. Decision-rule branch

The pre-registered rule was:

* **gold is the argmax in fewer than 50 % of items → failure in the SCORE representation,
  upstream of everything read-out**;
* gold is the argmax in a large majority but its weight is small → read-out / temperature /
  normalisation.

**The data lands squarely in the first branch.** The largest gold-argmax rate anywhere in either
run is 6.4 % (layer 4, run B) and the headline rates are 1.1–4.2 %; the NoRead-inclusive rate is
essentially 0. The gap to the 50 % threshold is two orders of magnitude in relative terms and far
outside sampling error (run B: 70/6144 last-layer rows; a 95 % interval is ≈ [0.9 %, 1.4 %]).

Corollary, and the actionable part: the score must first be able to *identify* the source. Until
the query/key encoding (and/or the distance term that currently dominates the score) can put the
gold source at rank 1 in a large majority of rows, no change to the softmax, the temperature, the
NoRead normalisation, the read-out projection or any routing can produce recall. The score is
currently a distance prior, not a content match.

**Secondary observation (underpowered, do not build on it yet).** The model's rare correct answers
do not coincide with the read selecting the gold: at 1200 steps 0 of the 64 items whose last-layer
read has the gold as top-1 are answered correctly (0/195 for top-3), against a 3.1 % base rate; at
200 steps 0/5 and 4/39. The few correct items are therefore produced by something other than the
read (residual/MLP path). With only 16 and 48 correct items this is suggestive, not established,
but it means that repairing the score representation may be necessary *and* not sufficient — the
value read-out deserves its own check once the scores can find the source.

## 5. Reproduction check (is the instrument trustworthy?)

1. **Configuration.** Run B used the sealed root's exact argv (`layout=fact steps=1200 batch=8
   final_sequences=256 seed=7`, same tokenizer), with the model config recorded identically:
   `pattern=aaaaaa, read=l2, rotation=true, width=128, heads=4, mlp=384, context=384,
   vocab_size=4096, seed=7`, 1 816 264 parameters — identical to `ro-full-base`'s log line.
2. **Accuracy replicates.** Run B final: held-out-class `full 0.031 first-content 0.031
   rehearse 0.030 bare 0.032 relevant-min 0.015 (rule 0.626)` — **the sealed root's final line,
   verbatim**. Every step line's shared fields (lr, NLL, grad norm, in-class accuracies, rule)
   are identical to `ro-full-base/log.txt` for all 12 evaluations. Run A (200 steps, batch 4)
   reached held-out-class first-content 0.0417 (16/384) with the curve at 2/96, the same regime as
   the reference's 1–6 /96 band.
3. **The instrument does not perturb the model.** Inside the dump, the observed forward's logits
   are bit-identical to `arm.logits` on the same batch (`max |gap| = 0.0`), and the per-item
   correctness reconstructed from the dump (0.03125 run B, 0.04167 run A) equals the bench's own
   reported `first_content_piece_accuracy` exactly. The dump runs after the final evaluation, so it
   cannot move any reported metric.
4. **The weights are a probability row.** `sum_j w_j = 1 − NoRead` lies in [2.7e-6, 0.9996]
   (run B) and [0.0077, 0.9974] (run A) — inside [0, 1] everywhere, as required.
5. **Default path.** `cargo test --release --example mqar-bench` → 14 passed, including
   `fact::tests::the_nlet_rule_copies_after_the_latest_earlier_occurrence` (the refactored rule)
   and `fact::tests::fact_windows_hold_their_facts_queries_and_targets` (the fact/gold invariants).

Conclusion: the instrument is validated and the run reproduces the known baseline profile.

## 6. Caveats that could undermine the conclusion

* **The gold definition.** If "the value the model must retrieve" meant a different window
  position, the rate would change — but the *whole fact span* is the argmax in only 3.6 % of
  last-layer rows and ±1 around the gold in 3.0 %, so no relabelling of the source position inside
  the fact can move the answer across the 50 % threshold. The read simply does not put mass on the
  fact (1.2 % on average at the last layer).
* **Scores are recovered from weights.** The dump records the exact softmax weights, not the raw
  pre-softmax scores. This is sufficient for every statistic reported (the softmax is monotone, so
  ranks and the argmax are identical; `ln w` is the score vector shifted by one constant per row,
  so every score *gap* is exact), but an absolute score scale is not directly observable. The
  distance profile above is therefore reported relative to the row's own position.
* **NoRead is inferred.** The NoRead probability is `1 − Σ_j w_j` from the same row, not an
  independently measured channel (the sanity range above bounds the error at ≲1e-6). The
  `NoRead is the largest entry` statistic compares that inferred value with the largest weight.
* **Short-run caveat, resolved.** Run A (200 steps) was ~3× more favourable to the read than the
  exact 1200-step configuration (pooled 0.80 % vs 2.92 %; last layer 0.39 % vs 1.14 %), so the
  short run is *not* quantitatively interchangeable — but both are two orders of magnitude below
  the threshold and the layer-wise maximum never exceeds 6.4 %. The headline uses the exact
  configuration.
* **One diagnostic row per item.** The analysis uses the first-content predicting row only (the
  row the reported metric scores). Other value pieces' rows are not dumped.
* **Ties.** Two sources with identical scores would make "the argmax" ambiguous. Measured over
  every read row of both runs (9 216 + 36 864 rows) the maximum weight was unique in **every**
  row, so no tie-breaking convention is involved.
* **This is one training seed (7) and one configuration** (`read=l2`, `age=default`,
  `lineage=none`, `pattern=aaaaaa`), i.e. the sealed reference's own. Other read scores
  (Lorentz/Dot), age initialisations or lineages were not measured here.

## 7. Raw data, durability

* **Sealed report roots (raw per-item dump; read-only):**
  * `~/uor-r4-worktrees/reports/t2-argmax-20261005-2/` — exact reference configuration. Holds
    `dump_scores/held_out.index.json` (per item: row, gold, landmarks, `weights_offsets`, the
    model's argmax, the rule's source) and `dump_scores/held_out.weights.f32` (14 155 776 f32 =
    1536 items × 6 layers × 4 heads × 384 sources; layout `chunk → layer → row → head`,
    little-endian f32, offset per row/layer in `weights_offsets`), plus `report.json`,
    `config.json`, `log.txt`, `attempt.json`, `manifest.json` (blake3-sealed).
  * `~/uor-r4-worktrees/reports/t2-argmax-20261005-1/` — the 200-step replication (3 538 944 f32).
* **Analysis (in the worktree, committed):** `t2/analyze_read_scores.py` (all statistics above),
  `t2/run1/analysis.{json,md}` and `t2/run2/analysis.{json,md}` (full numeric output including
  every per-layer/per-head/percentile table), `t2/run1/console.txt`, `t2/run2/console.txt`.

Neither root existed before this task; neither is reused; both were claimed exclusively by the
bench before any model work and sealed by it. No other worktree or report root was touched.


The full numeric run1/run2 outputs referenced above are retained in the canonical
archive `/workspace/uor-r4/codex/reconciliation/main-run-outputs-f4940919-20261006.tar.gz`
(SHA256 `72982f9a84f8ac410c043d57b9e7c9466508114499493a8d1ff19f537f33f61d`).
The script and this summary remain in Git; generated run outputs are no longer tracked.
