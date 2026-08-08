# The reconstructability certificate's null arm (#456)

Issue #456 promoted the EXCT-disabled reconstruction error from a Gate C
observation into a first-class metric — held-out top-1 and bits/token of a
compiled cover scored with exact-context lookup disabled, reported per
`cover_sweep` frontier point (schema 3, PR #462). Its harvest confirmed the
metric *discriminates*: configurations the with-EXCT agreement metric calls
identical (36.3–36.6% top-1) differ by 2.36 bits/token in reconstruction bits.

The one pre-registered exit criterion left open was the **null arm** (K-2 mutation
discipline): *shuffled emission tables must score at the unigram floor — a
certificate that passes on corrupted structure is a harness failure.* This
record closes it, and the measurement changed the reading of the whole metric.

## Method

`cover_sweep::reconstruction_null` scores one cover point's EXCT-disabled
reconstruction twice on the **same** held-out slice: once with the compiled
emission tables, once with every region's ΔE list **deranged** (a seeded
derangement — no region keeps its own list; the root prior is untouched). The
harness is `crates/uor-r4-graph-cli/tests/reconstruction_null.rs` (`#[ignore]`d;
`R4_RECON_NULL_HELD` caps the slice — both arms and the floor share it, so the
verdict is scale-robust). Measured on the default cover (k0=8/gain=0.25, 42
regions), 20,000 held-out positions of the pinned 500k fixture:

| arm | top-1 | bits/token |
|---|---:|---:|
| real emission tables | 0.0148 | 16.3209 |
| **deranged (null)** | 0.0034 | 19.3144 |
| unigram floor (train-argmax) | **0.0636** | **8.7059** |

## Two findings

**1. The mutation guard passes.** Deranging the emission tables degrades the
reconstruction by **~3.0 bits/token** (16.32 → 19.31) and cuts top-1 from 1.48%
to 0.34%. A shape-matched shuffle does *not* preserve the score, so the metric
reflects the cover's actual residual structure, not merely its byte-shape. The
certificate is not vacuous.

**2. The reconstruction is sub-unigram — the pre-registered wording rested on a
false premise.** The criterion said the shuffle must fall *to* the unigram floor,
which assumed the real reconstruction sits *above* it. It does not: the
EXCT-disabled graph reconstruction (16.32 bits / 1.48% top-1) is **7.6 bits worse
and 4.9pp lower top-1 than a trivial unigram prior** (8.71 bits / 6.36%). This is
consistent with the repo's standing record that graph-resolved positions score
~1% top-1 / 16.3 bits while ~86% of positions resolve via EXCT exact-context
memory: the graph residuals, applied without exact-context gating, are net-worse
than the unigram prior. So the shuffle does not "collapse to the floor" — it
pushes a sub-floor score even further below it.

## Consequence (a recorded NEGATIVE for item 3)

`cover_sweep`'s recon-bits metric is a valid *relative* discriminator among
covers, but every cover it ranks is sub-unigram on the graph-only path.
Optimizing reconstruction bits as a compiler split/stop criterion (issue #456
item 3) would therefore optimize a quantity that never beats a trivial baseline —
it would select finer covers that are *less bad* graph-only reconstructors, not
covers that reconstruct. That is the same shape as the programme's standing
pattern (resolution levers do not pay; the graph path answers ~1–3% of
positions). **Item 3 is not worth building on this metric as it stands**; the
lever the sibling measurements keep pointing at is evidence quality — the teacher
and corpus (#320), not finer subdivision.

The metric stays useful for what it is: a null-guarded, discriminating *relative*
reconstructability probe for the cover sweep. It should not be read as an
absolute reconstruction score, and the sub-unigram gap is the reason.

A tripwire assertion in the harness fires if a future change ever makes the graph
reconstruction beat the unigram floor — at which point this finding, and item 3,
should be revisited.
