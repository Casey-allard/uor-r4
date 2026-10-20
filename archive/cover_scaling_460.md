# Cover split criterion: the capacity law works, and the cover is not the bottleneck

Issue #460 lever 1. Measured 2026-08-07 on the committed 500k fixture corpus
(`crates/uor-r4-core/tests/fixtures/c_{meta,recs}.bin` + `tless_artifacts.bin`),
four story-prefix sizes, held-out top-1 of a region-path-keyed store.

Reproduce with:

```
cargo test --release -p uor-r4-graph-certify --test cover_scaling -- --ignored --nocapture
R4_SCALING_ARMS="absolute,scaled-k0@ref50k,relative+scaled@ref50k" \
  cargo test --release -p uor-r4-graph-certify --test cover_scaling -- --ignored --nocapture
```

About four minutes for five arms; no teacher, no checkpoint.

## The harness was reporting PASS on one data point

Before any result: running `cover_scaling.rs` as it stood produced

```
frac 13%: empty partition, skipped
frac 25%: empty partition, skipped
frac 50%: empty partition, skipped
PASS absolute monotone_regions=true ... predictive_ok=true (top1 0.1270 vs baseline 0.1270)
```

Three of four sizes skipped, then a PASS whose condition 1 (monotone region
count *in corpus size*) was evaluated over a single size, and whose condition 3
compared the baseline arm against itself. This is the #354 vacuous-green class,
in the harness whose only job is to decide this lever, and it is the most
likely reason the lever stayed dark: the run does not fail, it just says
nothing.

Cause: nested prefixes need every prefix to contain both partitions. The
`R4_STORIES` path partitions by `blake3(article id) % 5`, which interleaves.
The fallback was a contiguous tail split, so all held-out stories sat above the
80% mark and no smaller prefix had any. Fixed by making the fallback an
interleaved 80/20 (`sid % 5 != 0`), and by refusing to print PASS for an arm
with fewer than two sizes.

## The shipped criterion does not scale — visible at 500k, not just at 2.11M

With four real sizes, the `absolute` arm's region count against an 8× data
range:

| train observations | 51,696 | 99,455 | 199,562 | 400,006 |
|---|---:|---:|---:|---:|
| regions | 50 | 48 | 46 | 48 |

Flat, and non-monotone — **FAIL on condition 1**. The defect was previously
inferred from 500k (38 regions) versus 2.11M (14 regions); it is now visible
inside a single 500k corpus. Capacity does not track the data at any point in
the tested range.

## The capacity law works when it is actually exercised

`scaled-k0` as shipped **passes degenerately on this corpus and the pass must
not be believed.** The capacity law is `(n / capacity_ref_n)^alpha` with
`DEFAULT_CAPACITY_REF_N = 400_000`, and the fixture's full size is 400,006
train observations. At the anchor every scaled knob equals its base value, so
at the largest size `scaled-k0` *is* `absolute` — identical regions (48),
contrast (0.6217) and top-1 (0.1307) — and condition 3 compares it with
itself. Its monotonicity comes from being throttled below the anchor, not from
demonstrated growth above it.

Lowering the anchor to 50,000 puts the largest size 8× above it and tests the
law rather than the coincidence:

| arm | regions @51k | @99k | @199k | @400k | held-out top-1 @400k |
|---|---:|---:|---:|---:|---:|
| `absolute` (shipped) | 50 | 48 | 46 | 48 | 0.1307 |
| `scaled-k0@ref50k` | 50 | 63 | 71 | **104** | **0.1713** |
| `relative+scaled@ref50k` | 46 | 65 | 81 | **110** | **0.1809** |

Both anchored arms **PASS all three pre-declared conditions**:

1. monotone region count — 50 → 63 → 71 → 104 and 46 → 65 → 81 → 110;
2. contrast does not collapse — 0.6649 and 0.6643 against a 0.2506 floor, in
   fact slightly *above* the `absolute` arm's 0.6217;
3. prediction is not bought at a loss — **+4.1pp and +5.0pp** of held-out top-1
   on the region-path-keyed store.

Mean support per region falls 20,722 → 8,886 while top-1 rises. That is the
same fit-versus-resolution distinction the codebook measurement drew
(`docs/codebook_fit_460.md`), in the one place where added resolution genuinely
helps: at 48 regions for 400,006 records, the cover is so far under-resolved
that each region's emission is close to the global prior and the structure is
barely partitioned at all.

The two failing alternatives are informative too. `mdl` scales monotonically
(10 → 26 regions) but costs 1.7pp of held-out top-1 — capacity bought at a
loss, which is exactly what condition 3 exists to catch. `relative` alone,
without scaled capacity, stays flat: making the *bar* track the data does
nothing while `k0` and the region budget stay fixed. Capacity, not the
criterion, is the binding knob.

## What this is worth, and what it is not

**The serving impact is small and that must be said plainly.** The metric above
is the region-path-keyed store's own top-1. The deployed stack consults exact
context first: `exct.supported_record_fraction` is 0.9882 at the 4-stage
baseline, and the STAGES=5 run resolved 283 of 10,000 positions on the graph
path. So a cover-side change is bounded by roughly 1–3pp of headline movement,
and +5pp on a path that answers ~1–3% of positions is on the order of **0.15pp**
of headline today.

That bound is not a reason to discard the result. It locates the bottleneck:
**the cover is not what is limiting the engine — exact-context dominance is.**
The cover fix is cheap, already implemented, default-off, and now measured. It
pays when exact-context coverage falls, which is precisely what the broad-corpus
directions (#320's teacher upgrade, the #433 frontier) would cause. This result
means that when a broad corpus lands, the cover is ready rather than being a
fresh unknown.

It also revises a standing reading. #435 and the #460 STAGES=5 outcome both
recorded the cover as simply not participating — "8 regions, mass kept 0.0006,
the absolute entropy floor rejects every split at this scale." True of the
shipped configuration, and now known to be **fixable with an existing knob**
rather than a property of the geometry.

## Scope limits

- The `@ref50k` anchor is a measurement device, not a proposed default.
  Choosing a production anchor is a calibration question this run does not
  settle; `DEFAULT_CAPACITY_REF_N = 400_000` was itself calibrated to the 500k
  reference and is only degenerate *on a corpus of that exact size*.
- Adoption of any non-`Absolute` criterion or of scaled capacity is
  κ-affecting and needs the #407 re-pin ceremony.
- Measured to 400,006 train observations. Whether region growth continues at
  2.11M, and whether contrast holds there, is untested — that is the run that
  would justify adoption, and it is the same corpus the original 14-region
  collapse was observed on.
- The fallback partition changed from a tail split to interleaved, so absolute
  numbers here are not comparable with runs that supplied `R4_STORIES`.
  Arm-to-arm and size-to-size comparisons within one run are.
