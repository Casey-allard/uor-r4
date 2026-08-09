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

### Measured saturation sweep — SmolLM2-360M broad corpus (#514)

The first end-to-end run of `scale_sweep.sh` over the pinned #516 broad corpus
(360,924 records, SmolLM2-360M teacher on Simple-Wiki) sub-samples that one
corpus and re-runs compile → cover → score at each size:

| records | held-out | top-1 (Rule 1+2) | EXCT-miss |
|---:|---:|---:|---:|
| 25,000 | 5,008 | 15.83% | 56.65% |
| 50,000 | 10,127 | 14.85% | 48.65% |
| 100,000 | 18,373 | 18.54% | 38.13% |
| 200,000 | 40,149 | 19.66% | 33.53% |
| 360,000 | 72,195 | 23.91% | 26.77% |

Two reads:

- **The curve densifies and confirms the anchors.** The 25k point (56.65% miss)
  sits right on the 21,235→62.5% anchor, and the monotone decay through 360k is
  continuous with the 500,000→14.6% anchor above it. Same coverage law, more
  points.
- **No knee at 360k — this corpus is coverage-limited, not saturated.** Both
  top-1 (still climbing, +4.3pp from 200k→360k) and EXCT-miss (still falling,
  −6.8pp) are moving at the full corpus size. The 360M teacher's 360k-record
  corpus is well below the ~1–2M coverage knee, so it yields **no** `N_knee`
  point to fit β against. This is exactly why the calculator's headroom rule
  recommends ~2–3M records for the 360M baseline (#516): the measured sweep
  shows the substrate is still paying for data at the largest corpus we can
  observe on dev hardware.

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

Runbook (one observe already in hand → its `state.bin`/`merged.bin`, or a
`corpus.meta`/`corpus.records` pair). The sweep is one script:

```bash
# <src-meta> <src-recs> <vocab-size> [record sizes...]
scripts/scale_sweep.sh obs/state.bin obs/merged.bin 49152 \
    50000 200000 800000 2000000
```

It sub-samples with `scripts/mc1_subsample_corpus.py` (truncating to the last
complete story-run boundary at or before the target, so the 80/20 `train_cut`
split stays on the same story-id partition), then runs compile-recorded → cover
→ score at each size and prints `records | held_out | top1_rule12 | exct_miss_%`.
The knee is the smallest N past which top-1 and EXCT-miss stop moving.

The harness is in place (`scale_sweep.sh` + the existing `mc1_subsample_corpus.py`);
the compile/cover/score legs are the same ones #509 used. Until β is calibrated
from real knees, treat the calculator as an order-of-magnitude guide, not a
promise.

## Status

- `recommend-scale` estimator: **shipped** (#517).
- Saturation-sweep harness: **shipped and verified end-to-end** —
  `scripts/scale_sweep.sh` over `scripts/mc1_subsample_corpus.py`. The first run
  surfaced a latent bug: `compile-recorded --out $D` re-emits
  `corpus.meta`/`corpus.records` into `$D`, clobbering the sub-sampled inputs
  when they share those names. Fixed by writing the sub-sample under
  `sub.meta`/`sub.records` (the #516 pipeline had dodged it only because
  `obs_bundle_to_corpus.py` uses `.bin` names).
- β calibration: the 360M observe landed (#516) and its sweep is recorded above.
  It yields **no knee** — the largest corpus we can observe on dev hardware
  (360k records) is still on the rising part of the coverage curve, below the
  ~1–2M knee — so it cannot pin β on its own. β therefore stays **provisional
  0.5, bounded `[0.45, 1.0]`**; pinning it needs at least one teacher observed
  *past* its knee (a 2M-record observe is a multi-hour-to-multi-day teacher run).
  The law, the calculator, and the harness are complete and mutually consistent;
  what remains is compute, not tooling.
