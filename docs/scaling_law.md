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

### Measured saturation sweep — past the knee at 2M records, two teachers (#531)

The #514 sweep above topped out at 360k records and never reached the knee. #531
is the first observe *past* it. Two teachers — SmolLM2-360M and SmolLM2-135M —
were each teacher-forced over the same 17,000-article Simple-Wiki corpus to
**1,995,026 records** (batched inference, batch 64; 360M merged
κ `blake3:d122ab38…`, 135M merged κ `blake3:cde559db…`). One `scale_sweep.sh` per
teacher sub-samples that one corpus and re-runs compile → cover → score at each
size:

| records | held-out | 360M top-1 | 360M EXCT-miss | 135M top-1 | 135M EXCT-miss |
|---:|---:|---:|---:|---:|---:|
| 50,000 | 9,636 | 5.05% | 45.95% | 5.77% | 46.17% |
| 200,000 | 25,265 | 5.89% | 37.73% | 6.22% | 35.92% |
| 500,000 | 100,359 | 5.48% | 26.89% | 6.11% | 27.76% |
| 1,000,000 | 200,619 | 5.37% | 21.52% | 5.95% | 20.94% |
| 1,500,000 | 300,658 | 5.31% | 17.52% | 5.78% | 18.85% |
| 1,900,000 | 376,754 | 5.38% | 16.67% | 5.84% | 16.42% |

Three reads:

- **The coverage knee is real and now measured.** EXCT-miss falls steeply, then
  bends. The 360M marginal slope collapses from −1.07 pp / 100k records (around
  750k) to −0.21 pp / 100k in the 1.5M→1.9M segment; 135M mirrors it (−1.36 →
  −0.61). The elbow sits at **~1.5M records** — the first empirical confirmation
  of the "~1–2M knee" the anchors implied, and it grounds the calculator's
  `N_REF = 2,000,000` floor in a measured knee rather than an extrapolation.

- **Coverage is teacher-independent, exactly as the mechanism predicts.** The two
  EXCT-miss columns track within ~1–2pp at every scale (21.52% vs 20.94% at 1M;
  16.67% vs 16.42% at 1.9M). Coverage is a corpus n-gram property; swapping a
  360M teacher for a 135M one on the *same* articles barely moves it.

- **top-1 (resolution) did not scale — so β is still not pinnable.** Held-out
  top-1 stayed flat at ~5–6% for *both* teachers across the whole 50k→1.9M range
  (360M 5.05–5.89%, 135M 5.77–6.22%), with no upward trend. β is the exponent of
  the *resolution* demand — how much more corpus a deeper/wider teacher needs to
  keep its finer distinctions apart — and it can only be fit from a
  *teacher-dependent* knee in that resolution signal. Here the resolution signal
  is flat and the only knee (coverage) is shared, so the two teachers yield one
  coincident knee, not the two distinct knees a β slope requires. Fitting β to
  coincident coverage knees would return ≈0 — an artifact of measuring the wrong
  quantity, not a scaling exponent. We therefore do **not** pin β from this run.

What #531 settled: the coverage-knee floor is measured (~1.5M, `N_REF = 2M`
grounded) and teacher-independent, and the batched-inference path (#616) turns a
2M-record teacher observe into a ~2-hour run rather than a multi-day one. What
remains to pin β is a held-out *resolution* metric that climbs with corpus scale
**and** separates by teacher size; the flat top-1 here is a corpus/metric limit,
not evidence that β = 0.

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
- β calibration: **coverage knee now measured; resolution exponent still open.**
  #531 observed two teachers (360M, 135M) past the knee — 1,995,026 records each
  on 17k Simple-Wiki via batched inference (#616) — and its dual sweep (recorded
  above) measures the coverage knee at **~1.5M records** and shows it is
  teacher-independent, empirically grounding `N_REF = 2,000,000`. But held-out
  top-1 stayed flat (~5–6%) for both teachers, so the run exposes no
  teacher-dependent *resolution* knee: the two coverage knees coincide and cannot
  fit a slope. β therefore stays **provisional 0.5, bounded `[0.45, 1.0]`** — no
  longer for want of a past-knee observe (that box is now checked), but for want
  of a resolution metric that scales with corpus size and separates teachers. The
  earlier 360M-only sweep (#516, below the knee at 360k) is superseded by this
  past-knee measurement. The law, the calculator, and the harness remain complete
  and mutually consistent; what remains is a resolution metric, not compute.
