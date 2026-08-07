# Codebook fit: how much the RVQ training-set cap actually costs

Issue #460 lever 2. Measured 2026-08-07 on the pinned 500k fixture corpus
(`crates/uor-r4-core/tests/fixtures/c_{meta,recs}.bin`, 500,000 records /
2,507 stories, 399,694 construction / 100,306 held out), teacher-free
`compile_recorded`, cover held fixed.

Reproduce with:

```
cargo test --release -p uor-r4-core --test codebook_fit -- --ignored --nocapture
```

Five recorded compiles, about 30 minutes on two cores. No teacher, no
checkpoint, no Gate C.

## Result

| arm | pool | cap | training set | occupied keys | rec/key | held-out top-1 | vs shipped |
|---|---:|---:|---:|---:|---:|---:|---:|
| A | 10k | 10k | 10,000 | 77,799 | 5.14 | 35.53% ± 0.15 | −0.16pp |
| **A — SHIPPED** | **50k** | **10k** | **10,000** | **79,244** | **5.04** | **35.69% ± 0.15** | — |
| A | 200k | 10k | 10,000 | 74,952 | 5.33 | 35.74% ± 0.15 | +0.05pp |
| B | 50k | 50k | 50,000 | 85,496 | 4.68 | **36.14% ± 0.15** | **+0.44pp** |
| B | 200k | 200k | 200,000 | 79,933 | 5.00 | 36.06% ± 0.15 | +0.37pp |

All five arms produced distinct codebook digests, so no comparison here is
vacuous.

**Verdict: NEGATIVE against the pre-declared +1.0pp exit rule, but not null.**
The best arm is **+0.44pp at 2.1 SE** on the difference (conservative: the
arms are paired on the held-out set, so the independent-sample error used here
overstates the uncertainty). And it **saturates** — quadrupling the training
set again, 50k → 200k, buys nothing.

## Three things this settles

### 1. Raising `CTX_SAMPLE` above the cap does nothing (sweep A)

Over a 20× pool range with the cap pinned at the shipped 10,000, top-1 moves
0.21pp — inside noise. The k-means training set is
`min(CTX_SAMPLE, RVQ_SAMPLE_CAP)`, and once the pool exceeds the cap the pool
is irrelevant.

**But §3.1 of `capacity_scaling_432.md` is one step too strong**, and the
arithmetic matters. It states that #407's `CTX_SAMPLE` 6,000 → 50,000 raise
"did not change the codebook's training-set size at all". At
`CTX_SAMPLE = 6_000` the 10,000 cap **does not bind**, so that raise did move
the training set — from 6,000 to 10,000. #407's attributed +0.5–0.7pp
starvation share was therefore real, and bought by a 4,000-vector increment.
What is dead is raising `CTX_SAMPLE` any further while the cap stands, which
is every configuration since #407.

Read together with sweep B, the three points form one coherent
diminishing-returns curve: 6k → 10k bought roughly +0.5pp (#407), 10k → 50k
buys +0.44pp (here), 50k → 200k buys nothing.

### 2. `RVQ_SAMPLE_CAP` is a real but small lever, and it saturates near N/10

The audit's §5.3 prescription is `RVQ_SAMPLE_CAP(N) = min(N/10, 500_000)`. At
this corpus `N/10 = 39,969`, which lands almost exactly on the measured
saturation point of 50,000. **The formula's shape is right and its ceiling is
about +0.44pp** — it should be narrowed to the saturation point rather than
left implying that more is better, because 200,000 measurably is not.

Whether to adopt it is a judgement call this record does not make: the change
is κ-affecting and would require the #407 re-pin ceremony, which is a lot of
process for 0.44pp. The reasonable course is to bundle it into the next re-pin
that happens for another reason, not to trigger one.

This also bounds a standing worry. §6 records
`code.codebook_sample_fraction` as SATURATED at 500k, noting that "the 500k
measurements everything else is calibrated against were themselves taken on an
under-fit codebook." True — and the under-fit is worth **0.44pp**. The
calibration concern is real and small, which closes it rather than leaving it
open as an unbounded doubt about every pinned row.

### 3. Records-per-key is a symptom, not the binding quantity

This is the result with consequences beyond #460.

The STAGES=5 negative was read as: records/key fell 36.02 → 18.80 and top-1
fell with it, which is "the signature of thinner per-key evidence rather than
sharper context."

Here, records/key fell **5.04 → 4.68** and top-1 **rose +0.44pp**.

Both changes thin the evidence per key. They differ in *why*: STAGES=5 thinned
it by adding key **resolution**, splitting evidence that belonged together; a
larger codebook training set thins it by improving **fit**, moving evidence
onto the key that represents it. So a falling records-per-key is not itself a
warning sign, and #460's causal reading should be narrowed to resolution
specifically rather than left as a general claim about per-key evidence.

The programme-level pattern in `README.md` — every key-*resolution* lever
failed, and the only changes that helped improved *evidence quality per key* —
survives this and is sharpened by it. Codebook fit is an evidence-quality
lever and it moved the metric in the right direction; it is simply a small
one. It is the first independent confirmation of that pattern on a lever that
touches the code itself rather than storage or calibration.

## Scope limits

- Measured at 500k records with a teacher-free recorded compile. The pinned
  era rows are TLA7 compiles against a teacher; absolute numbers here are not
  comparable to `BASELINE.md`, only the arm-to-arm deltas are.
- Saturation was located between 50k and 200k on a 500k corpus. Whether the
  saturation point tracks `N/10` at 2.11M is untested, and that is the one
  thing that would change the recommendation — if it scales, the lever stays
  worth ~0.44pp at every corpus size; if it is absolute, larger corpora are
  already past it.
- Held-out store top-1 is the metric, matching the "store baseline" row #460
  reported. Full Gate C Rule 1+2 was not run: at a 0.44pp effect it could not
  have separated the arms any better, and it costs hours.
