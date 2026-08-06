# Capacity scaling audit (#460)

References #399, #393, #435, #407.

Status: **audit only**. Nothing in this document changes a production default.
The scaling formulas in the last section are proposals, each paired with the
measurement that would confirm or refute it.

## 1. The measurement that triggered this audit

Running the score pipeline on a 4.2x larger corpus (500 000 records ->
2 110 111 records, the 10k-article wiki observation set) produced a
**degenerate** configuration, not a better model.

| metric | 500k corpus | 2.11M corpus |
|---|---|---|
| induced cover regions | 38 | 14 (FEWER on more data) |
| mean records per region | 3.1M | 17.9M |
| emission contrast mean | 0.5012 | 0.0915 |
| mean_probability_mass_kept | 0.0157 | 0.0003 |
| EXCT resolving at FULL code | 85.4% | 97.8% |
| Gate C validity | measures generalization | "CANNOT measure generalization" (strict miss 2.2% < threshold) |
| Rule 1+2 top-1 | 36.5% | 26.1% |

Record counts are exact: `/tmp/c_meta.bin` declares `n = 500 000` over 2 507
stories at 48 bytes/record; `/tmp/wiki10k-obs/state.bin` declares
`n = 2 110 111` over 10 000 stories at 88 bytes/record. The growth factor is
**4.22x**.

Every metric in the right-hand column is the signature of a structure that ran
out of cells and started bucketing: fewer, larger regions; emission lists that
no longer distinguish anything; a full code that resolves nearly every query
exactly, which is memorization, not generalization.

## 2. The primary cause of the 38 -> 14 region collapse

**It is not a sampling cap.** State this explicitly, because a sampling cap was
the leading hypothesis:

- `run_score` builds observations from the *complete* train position list and
  passes all of them to `induce_cover`
  (`crates/uor-r4-graph-cli/src/lib.rs:646-660`). There is no subsample.
- `induce_cover` derives `batch_size` from the memory budget
  (`crates/uor-r4-graph-compiler/src/induction.rs:1680`,
  `derive_batch_size` at `induction.rs:792`), but `spherical_kmeans` iterates
  `while batch_start < n` and assigns **every** point
  (`induction.rs:922-949`). The batch is a memory shape, not a sample.
- `regions_budget` (default 256, `induction.rs:299`) never binds: with
  `DEFAULT_K0 = 8` (`induction.rs:168`), `SPLIT_CHILDREN = 2`
  (`induction.rs:170`) and `DEFAULT_DEPTHS = 3` (`induction.rs:166`) the
  ceiling is `8 + 8*2 + 16*2 = 56` regions, well under 256.
- `min_support` (default 64, `induction.rs:182`) never binds: at 2.11M records
  the construction split is about 1.69M observations, so a depth-1 region
  averages about 211 000 observations, four orders of magnitude above 64.

By elimination, the only rule that can reject a split is the split test itself.
`induce_cover` continues past a candidate iff `entropy_allows_split &&
objective_allows_split` (`induction.rs:1826-1828`), and `entropy_allows_split`
is:

```
crates/uor-r4-graph-compiler/src/induction.rs:606
    SplitCriterion::Absolute => gain_bits > self.entropy_gain_bits,
```

with `DEFAULT_SPLIT_ENTROPY_GAIN_BITS = 0.25`
(`crates/uor-r4-graph-compiler/src/induction.rs:184`) and
`SplitCriterion::Absolute` as the shipped default
(`induction.rs:493`, `split_criterion_override("R4_COVER_SPLIT_CRITERION")`).

**PRIMARY CAUSE: a fixed absolute entropy-gain floor of 0.25 bits/token,
evaluated on a plug-in maximum-likelihood entropy estimator whose finite-sample
bias shrinks as 1/support. The bar is constant in N; the quantity measured
against it falls as N grows, so more data buys fewer splits.**

### The arithmetic

`gain_bits` comes from `entropy_reduction`
(`induction.rs:1124-1143`), which is `H(parent) - sum_c (|c|/|parent|) H(c)`
using `entropy_bits` (`induction.rs:1108-1119`) — the naive plug-in estimator
`-sum p log2 p`, with **no bias correction**.

The plug-in estimator underestimates entropy by approximately the Miller-Madow
term `(T - 1) / (2 S ln 2)` for support `S` over `T` observed types. For a
binary split of a parent of support `S` into two children of support `S/2` that
share roughly the same type set, the *measured* gain therefore carries a
spurious positive component:

```
delta  =  (T-1) / (2 * (S/2) * ln 2)  -  (T-1) / (2 * S * ln 2)
       =  (T-1) / (2 * S * ln 2)
       ~= 0.721 * (T-1) / S      bits
```

Put numbers on it. The construction split is 80% of stories, so:

- 500k corpus: about 400 000 train observations, `k0 = 8` -> depth-1 support
  `S ~= 50 000`, depth-2 support `S ~= 25 000`.
- 2.11M corpus: about 1 690 000 train observations, `k0 = 8` -> depth-1 support
  `S ~= 211 000`, depth-2 support `S ~= 105 000`.

`T`, the count of distinct next tokens in a region, is bounded by the
vocabulary and saturates early — it is essentially the same at both sizes. So
with `T ~= 8 000` in a depth-2 region:

```
500k :  delta ~= 0.721 * 8000 /  25 000  = 0.231 bits
2.11M:  delta ~= 0.721 * 8000 / 105 000  = 0.055 bits
```

At 500k the estimator bias alone is worth 0.231 of the 0.25-bit budget: a split
whose *true* gain is a few hundredths of a bit clears the floor. At 2.11M the
same split measures 0.176 bits lower and is rejected. The drop is exactly the
corpus growth factor, because `delta` is proportional to `1/S` and `S` is
proportional to `N` at fixed `k0`.

The observed region counts are consistent with this and with nothing else.
With `k0 = 8`, `SPLIT_CHILDREN = 2`, `depths = 3`, region count is
`8 + 2 * (accepted splits)`:

```
38 regions -> 15 of 24 candidate splits accepted
14 regions ->  3 of 24 candidate splits accepted, and ZERO at depth 2
```

The collapse is superlinear because a rejected depth-1 split also deletes the
two depth-2 candidates below it. Three surviving depth-1 splits at 2.11M means
the floor rejected 5 of 8 at depth 1 and 6 of 6 at depth 2.

### Confirmed by measurement

`T` is corpus-specific, so the arithmetic above is an order-of-magnitude
argument. The instrumentation test
(`crates/uor-r4-graph-certify/tests/capacity_scaling.rs`) settles it: it prints
the split-decision histogram from `induce_cover`'s decision trace together with
the realized gain. Run on both corpora with the shipped defaults:

| | 500k corpus | 2.11M corpus |
|---|---|---|
| train observations | 399 694 | 1 707 309 |
| regions induced | 40 (8 / 14 / 18 by depth) | **8** (depth 1 only) |
| split candidates audited | 22 | 8 |
| `split` | 16 (0.7273) | **0** |
| `keep:entropy_floor` | 6 (0.2727) | **8 (1.0000)** |
| `keep:objective_cost` / `keep:objective_tie` | 0 | 0 |
| mean realized gain | 0.4288 bits | **0.0703 bits** |
| max realized gain | 0.8385 bits | **0.1761 bits** |
| floor | 0.2500 bits | 0.2500 bits |
| mean train observations per region | 9 992 | 213 414 |

Three things are now established rather than inferred:

- The entropy floor is the **only** rule that ever rejects a split. The
  objective comparison (`compare_region_decision`, `induction.rs:1808-1820`)
  rejected nothing at either size, and neither did `min_support` or
  `regions_budget`.
- At 2.11M **not a single candidate** reaches the floor: the maximum realized
  gain over all eight depth-1 candidates is 0.1761 bits against a 0.2500-bit
  bar. Recursion stops at depth 1 and the cover is exactly `k0`.
- The gain collapses by 6.1x (0.4288 -> 0.0703) while support grows 4.27x
  (399 694 -> 1 707 309). That is the 1/S signature of the plug-in estimator's
  finite-sample term, not a property of the data.

The floor did not get stricter. The noise that used to clear it went away.

### Secondary contributor in the same function

`k0` is fixed at 8 (`induction.rs:168`) and `depths` at 3
(`induction.rs:166`), so the cover's *ceiling* is 56 regions at any corpus
size. Even with a perfect split test, a 100x corpus would still get at most 56
regions. `CoverConfig` already carries `scale_k0` / `scale_regions_budget`
(`induction.rs:365-367`) with `capacity_alpha = 0.45` and
`capacity_ref_n = 400 000` (`induction.rs:197`, `induction.rs:203`) — but both
switches default to **off** (`induction.rs:502-503`), so the shipped path never
scales.

## 3. The other capacity knobs

Each row: where it is set, what it is fixed at, how it should scale with the
record count `N`, and the evidence of saturation.

### 3.1 Context codebook training sample — the second hard cap

| | |
|---|---|
| where | `crates/uor-r4-core/src/transformerless/compiler.rs:197` (`CTX_SAMPLE = 50_000`), `compiler.rs:201` (`RVQ_SAMPLE_CAP = 10_000`), applied at `compiler.rs:1515` |
| fixed at | 10 000 training vectors, independent of `N` |
| should scale as | `min(N/10, cap)` on the *k-means training set*, not just the draw pool |
| evidence | see below |

This is the one place where a sampling cap really is the mechanism, and it is
worse than it looks. Issue #407 raised `CTX_SAMPLE` from 6 000 to 50 000, so
50 000 bundles are extracted at `compiler.rs:1762-1781`. But
`sampled_kmeans_rvq` immediately subsamples them:

```
crates/uor-r4-core/src/transformerless/compiler.rs:1515
    let sample_size = nvec.min(capacity_override_usize("R4_RVQ_SAMPLE_CAP", RVQ_SAMPLE_CAP));
```

and every centroid update runs over `sample_residual` only
(`compiler.rs:1533-1560`). The caller discards the full-set codes
(`let (ctx_cb, _) = sampled_kmeans_rvq(...)`, `compiler.rs:1782`), so **only
10 000 vectors ever influence the codebook.** Raising `CTX_SAMPLE` 6k -> 50k
changed the pool the 10 000 are drawn from and the extraction cost; it did not
change the codebook's training-set size at all.

Arithmetic:

```
K = 256 centroids per stage (compiler.rs:191), STAGES = 4 (compiler.rs:190)
10 000 training vectors / 256 centroids = 39 vectors per centroid, per stage

as a share of the construction split:
  500k  corpus:  10 000 / ~400 000   = 2.50%
  2.11M corpus:  10 000 / ~1 690 000 = 0.59%
```

The code's resolution is therefore literally constant while the corpus grows
4.22x. This is the direct cause of the EXCT column: a codebook fit to 39 points
per centroid places corpus mass onto keys whose boundaries were never fit to
that mass.

### 3.2 Graded code capacity (STAGES x K)

| | |
|---|---|
| where | `crates/uor-r4-core/src/transformerless/compiler.rs:190-191` |
| fixed at | `STAGES = 4`, `K = 256` -> `256^4 = 4 294 967 296` nominal full-code keys |
| should scale as | nominal capacity is not the binding constraint; the *fit* is (see 3.1) |
| evidence | "EXCT resolving at FULL code" rose 85.4% -> 97.8% |

Nominal capacity is 2 035x the 2.11M record count, so the code is not
capacity-starved in the counting sense. The failure is that occupancy became
*more* concentrated per key while resolution stayed constant: at 97.8% full-code
resolution nearly every held-out query finds an exact key, which is why Gate C
reports "CANNOT measure generalization" — a strict miss rate of 2.2% is below
the threshold at which a generalization claim can be made at all. The graded
code stopped being a generalizing address and became a hash table.

`CTX_ITERS = 6` (`compiler.rs:198`) is also fixed: six Lloyd iterations over
10 000 points is adequate for 10 000 points and says nothing about 1.69M.

### 3.3 Emission selection

| | |
|---|---|
| where | `crates/uor-r4-graph-certify/src/score.rs:144` (`DEFAULT_EMISSION_ENTRIES = 64`), format ceiling `DEFAULT_MAX_EMISSION_ENTRIES = 64` at `score.rs:281` and `crates/uor-r4-graph-compiler/src/induction.rs:329` |
| fixed at | 64 tokens per region |
| should scale as | with region *mass*, not with `N` directly: `E ~ E0 * (records_in_region / ref_records)^beta` |
| evidence | `emission contrast mean` 0.5012 -> 0.0915; `mean_probability_mass_kept` 0.0157 -> 0.0003 |

`mean_probability_mass_kept = 0.0003` is the clearest single number in the
table: the 64 emitted tokens of a region cover **0.03%** of that region's actual
next-token mass. With 14 regions over 2.11M records a region holds about
150 000 records; 64 tokens selected by log-ratio against the parent cannot
cover that. The 52x collapse in mass kept (0.0157 -> 0.0003) is roughly the
product of the region-count collapse (2.7x) and the corpus growth (4.2x), i.e.
about 11x, plus the contrast collapse — consistent with a fixed-E list against
a growing per-region type count.

Contrast falling to 0.0915 says the same thing from the other side: the top-64
by log-ratio and the top-64 by probability have become nearly the same list,
because with that much mass per region the log-ratio ranking is dominated by
the head. The emission list stopped being distinctive.

### 3.4 Root and EXCT candidate caps

| | |
|---|---|
| where | `crates/uor-r4-graph-certify/src/score.rs:292` (`DEFAULT_ROOT_TOP_B = 64`), `score.rs:294` (`DEFAULT_EXCT_TOP_X = 64`), `score.rs:300` (`DEFAULT_WITNESS_SAMPLE = 64`), `score.rs:142` (`DEFAULT_TRANSITION_OUT_DEGREE = 8`) |
| fixed at | 64 / 64 / 64 / 8 |
| should scale as | `root_top_b` with vocabulary head coverage; `exct_top_x` with mean per-key type count |
| evidence | indirect: Rule 1+2 top-1 36.5% -> 26.1% |

These caps bound the candidate set a query can rank over. They do not by
themselves collapse the geometry, but they cap the achievable top-1 once the
per-key type count exceeds them: at 2.11M a full-code key carries more distinct
next tokens than at 500k, and the correct token can be ranked out of a
64-candidate list. The instrumentation test reports the per-key type
distribution so this becomes checkable rather than inferred.

### 3.5 FWDA forward-anchor table

| | |
|---|---|
| where | `crates/uor-r4-graph-certify/src/score.rs:766` (`FWDA_ENTRY_CAP = 64`), `score.rs:769` (`FWDA_MIN_TOTAL = 2`), applied at `score.rs:845-849` and `score.rs:851` |
| fixed at | 64 entries per row, rows dropped below total evidence 2 |
| should scale as | `FWDA_ENTRY_CAP` with mean row total; `FWDA_MIN_TOTAL` with `N` so that a row's evidence, not its existence, is the filter |
| evidence | 4.22x mass against unchanged caps |

Two failure modes, in opposite directions:

- `FWDA_ENTRY_CAP = 64` truncates each `(distance, anchor)` row to its 64
  highest-count entries. Row totals grow proportionally with `N`, so at 2.11M a
  hot anchor's row is clipped far harder than at 500k while the stored `total`
  still reports the full pre-truncation evidence. The serving loader derives
  smoothed residuals from a `total` that the retained entries no longer
  account for.
- `FWDA_MIN_TOTAL = 2` is a *floor*, and a floor of 2 admits essentially every
  row at any corpus size. At 4.22x the mass it admits 4.22x the noise rows: a
  two-observation row still carries no signal, it just now exists 4.22x more
  often. A floor fixed in absolute counts gets weaker, not stronger, as data
  grows.

`M2_STRIDE = 4` (`score.rs:2448`) bounds the lookahead distances to 1..4 and is
likewise fixed.

### 3.6 NGRAM context rows

| | |
|---|---|
| where | `crates/uor-r4-graph-certify/src/score.rs:296` (`DEFAULT_CONTEXT_ENTRIES = 64`), `score.rs:298` (`DEFAULT_CONTEXT_ORDER = 2`), applied at `score.rs:752` |
| fixed at | 64 entries per context row, orders 1 and 2 (bigram + trigram) |
| should scale as | `context_order` with `N` (a trigram table is under-fit at 2.11M); `context_entries` with mean row total |
| evidence | row count grows with `N` but per-row width does not |

Unlike the cover, the NGRAM table's *row count* does scale naturally with the
corpus — distinct contexts grow roughly as `N^0.44` (measured in #435). The
saturation is per row: `truncate(config.context_entries)` at `score.rs:752`
clips every row at 64 regardless of how much evidence it accumulated. And
`context_order = 2` caps the table at trigrams; the extra data at 2.11M is
exactly what would support a 4-gram table, and it is not used.

## 4. Summary of what is fixed

| knob | file:line | value | scales with N today |
|---|---|---|---|
| `DEFAULT_SPLIT_ENTROPY_GAIN_BITS` | `induction.rs:184` | 0.25 bits | no (PRIMARY CAUSE) |
| `DEFAULT_K0` | `induction.rs:168` | 8 | no (`scale_k0` defaults off) |
| `DEFAULT_DEPTHS` | `induction.rs:166` | 3 | no |
| `SPLIT_CHILDREN` | `induction.rs:170` | 2 | no |
| `DEFAULT_REGIONS_BUDGET` | `induction.rs:299` | 256 | no (never binds) |
| `DEFAULT_MIN_SUPPORT` | `induction.rs:182` | 64 | no (never binds) |
| `RVQ_SAMPLE_CAP` | `compiler.rs:201` | 10 000 | no (hard cap) |
| `CTX_SAMPLE` | `compiler.rs:197` | 50 000 | no (and inert past 10k) |
| `CTX_ITERS` | `compiler.rs:198` | 6 | no |
| `STAGES` x `K` | `compiler.rs:190-191` | 4 x 256 | no |
| `DEFAULT_EMISSION_ENTRIES` | `score.rs:144` | 64 | no |
| `DEFAULT_ROOT_TOP_B` | `score.rs:292` | 64 | no |
| `DEFAULT_EXCT_TOP_X` | `score.rs:294` | 64 | no |
| `DEFAULT_CONTEXT_ENTRIES` | `score.rs:296` | 64 | no |
| `DEFAULT_CONTEXT_ORDER` | `score.rs:298` | 2 | no |
| `FWDA_ENTRY_CAP` | `score.rs:766` | 64 | no |
| `FWDA_MIN_TOTAL` | `score.rs:769` | 2 | no |
| `M2_STRIDE` | `score.rs:2448` | 4 | no |

Every one of them is env-overridable for measurement (#399/#393 M-C2,
`compiler::capacity_override_usize`), and every one of them defaults to a
constant.

## 5. Scaling proposal

Do not adopt any of these without the measurement listed beside it. They are
written as formulas in `N`, the construction-split record count, with
`N_ref = 400 000` — the reference the existing `capacity_ref_n`
(`induction.rs:203`) already uses.

### 5.1 Replace the absolute entropy floor (highest priority)

The floor should be a fraction of what there is to remove, not an absolute
number of bits:

```
accept split iff  gain_bits > theta * H(parent)
theta = 0.0384      (calibrated so the bar equals 0.25 bits at H(parent) = 6.518)
```

This already exists as `SplitCriterion::RelativeGain` (`induction.rs:607-612`)
with `DEFAULT_RELATIVE_GAIN_THETA = 0.0384` (`induction.rs:188`); it is
default-off. Reasoning: a relative floor is invariant to how the parent's mass
is spread, so it does not tighten as `N` grows.

Alternatively, make the bar explicitly evidence-aware —
`SplitCriterion::Mdl` (`induction.rs:613-619`) charges
`penalty * added_params * log2(N)` against `gain * support`, so both sides grow
with the data.

An orthogonal and probably necessary correction: **bias-correct
`entropy_bits`**. Subtracting the Miller-Madow term `(T-1)/(2 S ln 2)` inside
`entropy_bits` (`induction.rs:1108`) removes the 1/S artifact from `gain_bits`
directly, so any of the three criteria then measures a scale-free quantity.
This is the change that addresses the mechanism rather than compensating for
it.

Measurement that confirms it: run `capacity_scaling` at 500k and 2.11M and
require that (1) `keep:entropy_floor` share falls below
`COVER_ENTROPY_FLOOR_REJECT_MAX`, and (2) region count is non-decreasing in
corpus size. `cover_scaling.rs` (#435) already implements the multi-size arm
comparison for exactly these criteria.

### 5.2 Scale the cover's ceiling

```
k0(N)             = clamp(round(8   * (N / 400 000)^0.45), 2, 65 536)
regions_budget(N) = clamp(round(256 * (N / 400 000)^0.45), 2, 65 536)
depths(N)         = 3 + floor(log2(N / 400 000))
```

The first two are `effective_k0` / `effective_regions_budget`
(`induction.rs:557-586`) with `scale_k0` and `scale_regions_budget` turned on;
`capacity_alpha = 0.45` (`induction.rs:197`) is already calibrated to the
measured class-growth law (distinct next-token-distribution signature classes
grow as about `N^0.46`). At `N = 1.69M` this gives `k0 = 15`,
`regions_budget = 471`. `depths` has no scaling path today and needs one, or
the 56-region ceiling simply becomes a 15 + 2*(...) ceiling.

Measurement: region count and emission contrast at 4 corpus prefixes; require
region count to grow at least as `N^0.3` and contrast mean to stay within a
factor of 2 of the 500k reference (0.5012), which is `cover_scaling.rs`'s
pre-declared exit rule.

### 5.3 Scale the codebook training set

```
RVQ_SAMPLE_CAP(N) = min(N / 10, 500 000)
CTX_SAMPLE(N)     = min(N / 10, 500 000)     -- keep the two equal
CTX_ITERS         = 6 + floor(log2(N / 400 000))
```

Reasoning: `N/10` keeps the k-means training set a constant *fraction* of the
corpus (10% is what 500k effectively had before #407, and the value the audit
of #407 assumed it was getting); the 500 000 ceiling bounds the cost, which is
`O(iters * sample * K * D)` and at 500k samples with `K = 256`, `D = 288` is
about 5.5e11 multiply-adds per stage — the point where a wall-clock budget has
to be set deliberately. Holding `CTX_SAMPLE = RVQ_SAMPLE_CAP` removes the
current trap where raising one has no effect.

Measurement: at fixed corpus, sweep `R4_RVQ_SAMPLE_CAP` over
10k / 50k / 200k and report distinct occupied full-code keys, mean records per
occupied key, and held-out top-1. If the codebook is under-fit, occupancy rises
and records-per-key falls monotonically with the cap.

### 5.4 Scale emission width by region mass

```
E(region) = clamp(round(64 * (records_in_region / 13 000)^0.5), 64, 1024)
```

13 000 is the 500k reference's records per region (about 400 000 train
observations over 38 regions ~= 10 500; 13 000 rounds to the measured
per-region mass in the report). The square root is the standard
type-token (Heaps) exponent: distinct types in a region grow roughly as
`mass^0.5`, so emission width should too. The format's
`max_emission_entries` header field (`crates/uor-r4-graph-format/src/head.rs:184`)
already carries the realized value, so widening is a format-compatible change.

Measurement: `mean_probability_mass_kept` and `mean_contrast` per `E`. Adopt
the smallest `E` for which `probability_mass_kept` at 2.11M is at least the
500k reference (0.0157) and `mean_contrast` has not fallen below 0.25.

### 5.5 Scale the FWDA caps by density

```
FWDA_ENTRY_CAP(N) = clamp(round(64 * (N / 400 000)^0.5), 64, 1024)
FWDA_MIN_TOTAL(N) = max(2, round(2 * N / 400 000))
```

The entry cap follows the same Heaps argument as emission width. `FWDA_MIN_TOTAL`
must scale *up* with `N` for the opposite reason: it is a noise floor, and a
noise floor fixed in absolute counts admits proportionally more noise as the
corpus grows. At `N = 1.69M` this gives an entry cap of 132 and a minimum row
total of 8.

Measurement: FWDA row count, mean row total, and share of rows at the entry cap
(all three printed by `capacity_scaling`), plus Gate C top-1 with and without
the FWDA residual. Adopt the pair for which the truncated-row share falls under
`ROW_TRUNCATION_FRACTION_MAX` (0.10) without the row count exploding.

### 5.6 Scale the NGRAM table

```
context_entries(N) = clamp(round(64 * (N / 400 000)^0.5), 64, 1024)
context_order(N)   = 2 + floor(log10(N / 400 000))
```

`context_order = 3` (4-grams) becomes supportable at about 4M records under
this rule. Reasoning: a higher-order table needs enough evidence per context
that its rows clear the runtime's exact-row-presence rule; that evidence is
exactly what a bigger corpus supplies.

Measurement: per-order row count, mean row total, and the share of held-out
positions whose most-specific present row is at each order. Raise the order
only when the next order's rows carry at least `EXCT_SUPPORT_MIN` mean
evidence.

### 5.7 Keep the EXCT support gate honest

`EXCT_SUPPORT_MIN = 5` (`crates/uor-r4-graph-certify/src/score_runtime.rs:179`)
is the gate that decides whether a full-code key is trusted. It is not itself
the cause of the collapse, but the Gate C validity failure — 2.2% strict miss,
below the threshold at which generalization can be measured — is a direct
consequence of a fixed gate against 4.22x mass. Under any of the fixes above
the code should resolve *less* often, not more; if it does not, the fix did not
work.

Measurement: `capacity_scaling`'s
`SATURATION VERDICT exct.supported_record_fraction` line. It must stay at or
below 0.90 for Gate C to have anything to measure.

## 6. Instrumentation

`crates/uor-r4-graph-certify/tests/capacity_scaling.rs` (ignored; run with
`--ignored --nocapture`) reports, for any corpus supplied via
`R4_CORPUS_META` / `R4_CORPUS_RECS`:

- COVER: regions per depth, region-budget occupancy, mean train observations
  per region and per leaf, the split-decision histogram, and the realized
  entropy-gain mean and max against the floor.
- GRADED CODE: occupied keys per prefix level against the nominal `K^level`,
  mean construction records per occupied full code, and the codebook's
  k-means training-set size as a fraction of the construction split.
- EXCT: occupancy histogram of full-code keys by total evidence, singleton-key
  share, and the share of evidence mass sitting in keys that clear
  `EXCT_SUPPORT_MIN`.
- FWDA: row count, mean and max row total, and the share of rows at the entry
  cap.
- NGRAM: bigram and trigram row counts and the share at the entry cap.

Each structure prints one `SATURATION VERDICT <name>: PASS|SATURATED` line
against the thresholds documented at the top of the test file
(`COVER_OBS_PER_REGION_MAX = 100 000`,
`COVER_ENTROPY_FLOOR_REJECT_MAX = 0.50`, `CODE_RECORDS_PER_KEY_MAX = 32`,
`CODE_TRAIN_SAMPLE_FRACTION_MIN = 0.05`,
`EXCT_SUPPORTED_RECORD_FRACTION_MAX = 0.90`,
`ROW_TRUNCATION_FRACTION_MAX = 0.10`).

The point of the verdict lines is procedural: a measurement taken on a
configuration that prints `SATURATED` is a measurement of the cap, not of the
model, and must not be quoted as a model result.

## 7. Measured instrumentation, both corpora

Shipped defaults, `R4_CAP_THREADS=2`, artifacts
`crates/uor-r4-core/tests/fixtures/tless_artifacts.bin`. Logs:
`/tmp/capacity_500k.log`, `/tmp/capacity_10k.log`.

| quantity | 500k corpus | 2.11M corpus | ratio |
|---|---|---|---|
| records | 500 000 | 2 110 111 | 4.22x |
| construction records | 399 694 | 1 707 309 | 4.27x |
| held-out records | 100 306 | 402 802 | 4.02x |
| cover regions | 40 | 8 | 0.20x |
| cover max depth | 3 | 1 | |
| split candidates accepted | 16 of 22 | 0 of 8 | |
| entropy-floor rejects | 6 (0.2727) | 8 (1.0000) | |
| mean realized split gain | 0.4288 bits | 0.0703 bits | 0.16x |
| max realized split gain | 0.8385 bits | 0.1761 bits | 0.21x |
| train observations per region | 9 992 | 213 414 | 21.4x |
| occupied keys, level 1 | 255 of 256 | 254 of 256 | |
| occupied keys, level 2 | 19 194 of 65 536 | 21 296 of 65 536 | 1.11x |
| occupied keys, level 3 | 63 441 of 16.8M | 110 038 of 16.8M | 1.73x |
| occupied keys, level 4 (FULL) | 96 233 of 4.295e9 | 217 546 of 4.295e9 | 2.26x |
| records per occupied full key | 4.15 | 7.85 | 1.89x |
| singleton full keys | 0.5927 | 0.6362 | |
| codebook k-means vectors | 10 000 | 10 000 | 1.00x |
| codebook share of construction split | 0.02502 | 0.00586 | 0.23x |
| held-out resolving at FULL code | 0.8510 | 0.9165 | |
| strict miss | 0.1490 | 0.0835 | |
| FWDA rows | 7 743 | 35 355 | 4.57x |
| FWDA mean row total | 38.16 | 31.89 | 0.84x |
| FWDA max row total | 6 076 | 16 478 | 2.71x |
| FWDA rows at the entry cap | 0.0378 | 0.0475 | |
| NGRAM rows | 64 774 | 748 107 | 11.55x |
| NGRAM rows at the entry cap | 0.0042 | 0.0045 | |

Verdicts:

| verdict | 500k | 2.11M |
|---|---|---|
| `cover.observations_per_region` | PASS (9 992) | **SATURATED (213 414)** |
| `cover.entropy_floor_rejects` | PASS (0.2727) | **SATURATED (1.0000)** |
| `code.records_per_full_key` | PASS (4.15) | PASS (7.85) |
| `code.codebook_sample_fraction` | **SATURATED (0.02502)** | **SATURATED (0.00586)** |
| `exct.supported_record_fraction` | PASS (0.8510) | **SATURATED (0.9165)** |
| `fwda.row_truncation` | PASS (0.0378) | PASS (0.0475) |
| `ngram.row_truncation` | PASS (0.0042) | PASS (0.0045) |

Reading the table:

- The instrument reproduces the original measurement's EXCT column: 0.8510 vs
  the reported 85.4% at 500k. The 2.11M figure here (0.9165) is lower than the
  reported 97.8% because this run uses the 500k-fitted artifact container
  rather than a recompiled one; the direction and the verdict flip are the
  same.
- The cover result is more extreme than the original report: with the shipped
  defaults and no artifact recompile, the 2.11M cover does not split at all.
  Eight regions, depth 1, zero accepted splits.
- The graded code's realized resolution grows as roughly `N^0.55` (2.26x keys
  for 4.27x records) while its codebook training set is literally constant.
  That sublinearity is the codebook cap showing through: `RVQ_SAMPLE_CAP` is
  the only thing between the corpus and the codes.
- FWDA and NGRAM row *counts* do scale (4.57x and 11.55x), and their entry
  caps are not yet the binding constraint at 2.11M — the truncated share rises
  only 0.0378 -> 0.0475 and 0.0042 -> 0.0045. These are the two structures
  whose capacity is genuinely data-driven today. They are also the two whose
  proposed scaling (5.5, 5.6) is lowest priority.
- `code.codebook_sample_fraction` was already SATURATED at 500k. The 500k
  measurements everything else is calibrated against were themselves taken on
  an under-fit codebook.
