# Corpus-scale law — sizing an observe run for any teacher (#514)

How much corpus does a teacher of a given weight class need before the substrate
can cash its ceiling? This turns that question from a guess into a calculator:
`r4 transformerless recommend-scale`. The closed form below is a first estimate
from the runs we have; a sub-sample saturation sweep on a real observe measures
the true knee and supersedes it.

## The mechanism

The substrate covers the teacher's induced context state-space. Two demands
scale, and they scale differently:

- **Coverage** — enough records that a held-out context has an exact match, i.e.
  a low EXCT full-code miss rate. This is mostly a corpus n-gram property and
  saturates: it is largely *teacher-independent*.
- **Resolution** — a deeper, wider teacher conditions on longer, finer context,
  so the substrate must represent more distinct keys to keep the teacher's
  distinctions apart. This is *teacher-dependent* and is where model size enters.

## The anchors (measured)

Coverage vs corpus size — EXCT full-code resolution:

| records | EXCT full-code resolution | miss | source |
|---|---:|---:|---|
| 21,235 (176 wiki articles) | 37.5% | 62.5% | #509 |
| 500,000 (2,507 stories) | 85.4% | 14.6% | #432 |
| 2,110,111 (10k wiki articles) | 97.8% | 2.2% | #432 |

The coverage knee is ~1–2M records; past ~2M coverage plateaus. The cover
*capacity* keeps paying past the knee, but only with scaled capacity on:
`regions ≈ (N / ref_n)^0.45` grew regions 50→104 and held-out top-1 **+5.0pp**
at 400k (#460). That 0.45 is the measured sub-linear exponent the law inherits.

Config-only capacity proxy `S = d_model · n_layers · log2(vocab)`:

| teacher | d_model | layers | vocab | S | S / S(360M) |
|---|---:|---:|---:|---:|---:|
| stories15M | 288 | 6 | 32,000 | 25,861 | 0.054 |
| SmolLM2-135M | 576 | 30 | 49,152 | 269,308 | 0.562 |
| SmolLM2-360M | 960 | 32 | 49,152 | 478,770 | 1.000 |

## The law

```text
S(model) = d_model · n_layers · log2(vocab)
N_needed = N_REF · (S / S_REF) ^ BETA
```

Shipped anchors: `N_REF = 2,000,000` (the wiki coverage knee), `S_REF = S(360M)`,
`BETA = 0.5` **(provisional)**. β is bounded by the evidence in `[0.45, 1.0]` —
the lower bound is the measured cover-capacity exponent (#460); the upper bound is
linear. It is **not yet pinned**: the estimator reports its output with a
provisional-β caveat, and `--beta` overrides it.

`recommend-scale` returns the coverage-knee *floor*. Add headroom for the
capacity regime (which keeps paying as N^0.45): the 360M baseline (#516) is
recommended at ~10–15k articles / ~2–3M records, above the calculator's ~9.5k
floor, for exactly this reason.

## Calibrating β cheaply — observe once, sub-sample

You do not re-observe per scale. Observe once at the top scale, then sub-sample
the record stream and re-run the cheap downstream (cover → score) at each size;
the point where held-out top-1 and EXCT-miss flatten is that teacher's knee.
Three teachers → three knees → β by least-squares on `log N_knee vs log S`.

Runbook (one observe already in hand at `$OBS` → `corpus.meta`/`corpus.records`):

```bash
# Sub-sample the record stream to a target N (records are fixed-width; take a
# uniform stride so the construction/held-out split is preserved in proportion).
for N in 50000 200000 800000 2000000; do
  r4 transformerless subsample-corpus --in $OBS --out /tmp/sweep-$N --records $N   # (harness, #514)
  r4 transformerless compile-recorded --corpus-meta /tmp/sweep-$N/corpus.meta \
     --corpus-recs /tmp/sweep-$N/corpus.records --vocab-size 49152 --out /tmp/sweep-$N
  r4 transformerless cover --corpus-meta /tmp/sweep-$N/corpus.meta \
     --corpus-recs /tmp/sweep-$N/corpus.records --artifacts /tmp/sweep-$N/tless_artifacts.bin \
     --out /tmp/sweep-$N/graph-cover
  r4 transformerless score --corpus-meta /tmp/sweep-$N/corpus.meta \
     --corpus-recs /tmp/sweep-$N/corpus.records --artifacts /tmp/sweep-$N/tless_artifacts.bin \
     --cover /tmp/sweep-$N/graph-cover/cover.r4g1 --quality-profile relative_tla \
     --out /tmp/sweep-$N/graph
  # record top-1 and EXCT-miss from graph/score_report.json
done
# The knee is the smallest N past which top-1 and EXCT-miss stop moving.
```

The `subsample-corpus` harness is the remaining code piece; the compile/cover/
score legs already exist and are what #509 used. Until β is calibrated, treat
the calculator as an order-of-magnitude guide, not a promise.

## Status

- `recommend-scale` estimator: **shipped** (this change).
- `subsample-corpus` sweep harness: pending.
- β calibration: pending the 360M/135M/15M observes (dev hardware; a 2M-record
  observe is a multi-day teacher run — the #509 observe managed 176/3000 articles
  in this sandbox before it was interrupted).
