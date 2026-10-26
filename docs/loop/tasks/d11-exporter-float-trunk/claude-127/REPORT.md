# REPORT: d11-exporter-float-trunk, claude-127

Lab: claude. Order: [ORDER.md](ORDER.md). Decision review: [owner log on #2820](https://github.com/UOR-Foundation/uor-r4/issues/2820#issuecomment-6413121841) (fresh, round 2 APPROVE after a round-1 REJECT).

References #2820.

## What did not work, first

- **`G-D11-ExportGap` stays blocked, and `I-D11-Artifact-Agreement` was not re-run (NOT_RUN, not FAIL).** Even after the repair below, the integer artifact still disagrees with its f64 specification on about 1% of single tokens at 6 layers. From single tokens alone, that projects to a P of at least about 110 to 140 rows (the bar's P takes the larger of the single-token and window rates), against claude-126's bar of 6. Even the low end of a 95% interval on 4 or 5 events in 512 (about 0.21% and 0.32%) puts P near 30 and 45.
  - The pod re-run (ORDER done-when 3) could not pass or change the next step. So by decision D2 it was skipped, with no GPU spent.
  - These are host diagnostics on the laptop, on the stored Stage 4b snapshots. They decide spend only. None counts toward a bar, and no negative is recorded.
- **The residual's cause is inferred, not tested.** At 6 layers, after the repair:

  | Seed | Positions sampled (every 8th token) | Differ in token or kept cells | Differ in token |
  |---|---|---|---|
  | 1 | 512 | 9 | 5 |
  | 2 | 512 | 17, all in their kept cells | 4 |

  - **On seed 2, mostly not cell-ranking near-ties.** At the 17 positions, the reference's margin between its 8th and 9th cells runs from 2 to 156 LSB, against a median of 44 over all 512 (the 10th smallest is 6). 14 of the 17 are at 10 LSB or more; 3 (2, 4 and 4 LSB) may be near-ties.
  - **Gate-snap near-ties contribute on seed 2 but do not explain most.** The 17 positions' Hamilton-gate snap margins rank from 15 to 490 of 512. Three fall among the 30 smallest (ranks 15, 18 and 29).
  - **On seed 1, ties cannot be ruled out.** The reference's own 8th-to-9th cell margin is 0 LSB at the median: the address collapse below leaves the cell ranking exactly tied on most positions.
  - **No bias within ±3 LSB; the tails are unexplained.** These are per-channel query differences, every 32nd token, binned with anything beyond ±3 LSB in the end buckets.
    - At 6 layers the binned counts are symmetric. Their mean is +0.007 LSB on seed 1 and −0.016 on seed 2.
    - The full means at 6 layers are −1.64 and +3.07 LSB, of opposite sign, pulled by values beyond ±3 LSB. Opposite signs across two seeds do not rule out a bias within either seed.
    - Large differences are present already at 1 layer on seed 2. After the repair its full mean there is −0.53 LSB (with 127 of 128 kept-cell sets agreeing). Before the repair its largest 1-layer query gap was 9,300 LSB.
  - **The cause is untested.**
    - One candidate is the artifact's Q16 fixed-point residual stream against f64, accumulating over the layers. The large differences at 1 layer on seed 2 do not fit accumulation alone.
    - A second, unlocalized defect is not excluded.
    - The control that would test the Q16 candidate is the f64 reference with its residual rounded to Q16 at the same points. It was not built here.
- **ORDER done-when 2 holds in part.**
  - At 1 layer, every 32nd token, seed 1 matches the reference on 128 of 128 tokens and kept-cell sets. Seed 2 matches on 128 of 128 tokens and 127 of 128 kept-cell sets.
  - At 6 layers, 2 to 3% of positions differ. On seed 2, 14 of the 17 have cell margins of 10 LSB or more, so most are not the near-tie exceptions the done-when allows.
  - The two synthetic controls pass (below).
- **The dense path still uses the truncating norm.** Only the FBS step was repaired. The dense step is outside this ORDER and is unchanged.

## The defect, found and repaired

The integer FBS step had a systematic rounding bias against its specification.
- **The stage.** `stack_rms_norm` floors its product (`>> 48`) at 2^-14. The FBS step then applied the final gain and rounded the query to Q12 a second time. The f64 reference rounds once.
- **The size.** On seed 1 at 1 layer, every 32nd token (128 tokens), the query differs by −0.088 LSB on average, and only 90 of 128 tokens agree.
- **Why seed 1.** The writer scores its 256 cells from the query's first and last 8 channels alone. On seed 1, the trained final-norm gains on those 16 channels collapsed:

  | Seed | Mean \|gain\|, first 8 channels | Mean \|gain\|, last 8 | Mean \|gain\|, other 240 |
  |---|---|---|---|
  | 1 | 0.00059 | 0.00071 | 1.885 |
  | 2 | 0.203 | 0.144 | 1.974 |

  So on seed 1 the cell ranking is decided at a few Q12 LSB, where a −0.088 LSB bias changes the kept cells. Before the repair, restoring those 16 gains to the other channels' mean gave 128 of 128 tokens at 1 and 6 layers. That is the control showing it was the address's sensitivity.
- **ORDER done-when 1: the localized difference** is none of the four the ORDER listed (the non-square MLP, the depth, the larger tables, the larger decay offsets). It is tiny writer-address gains, as seed 1 trained them, which expose the step's truncating norm.
  - The finite control first fails at the tiny-gain shape: 1 layer, width 64, address gains about 6e-4.
  - The non-square, deeper control with larger tables passes with or without the repair.
- **The repair.** `stack_rms_norm_fine` rounds to nearest at 2^-24, and only the FBS step uses it.
  - The FBS step reads all three of its norms from it. The query reaches Q12 from that value: the norm and the final gain each round at 2^-24, negligible against a Q12 step, before one rounding to Q12.
  - The integer rounding is half-up. The reference's `f64::round` rounds half away from zero. They differ only on exact ties of negative values, by at most 1 LSB.
- **After the repair,** on seed 1 every 32nd token: 128 of 128 tokens and kept-cell sets at 1 layer (bias −0.0066 LSB), and 127 of 128 tokens at 6 layers. The residual is in the table above.

## What changed

Size against `origin/main`: runtime +36/−15 in `stack/kernels.rs` and `stack/session.rs`; tests +133/−8 in `stack/tests.rs`; docs: the ORDER, this REPORT and the notebook row. The artifact format is unchanged.

- **`stack_rms_norm_fine` and `NORM_FINE_EXP`** (`kernels.rs`): the norm, rounded to nearest at 2^-24, sharing its body with `stack_rms_norm` through `rms_norm_into`.
- **`stack_step_fbs`** (`session.rs`): the three norms through `stack_rms_norm_fine`, their gathers at `NORM_FINE_EXP`, and one rounding of the query to Q12.
- **`FbsSynth`** (`tests.rs`): the synthetic FBS trunk's shape and value ranges as parameters. The default draw is unchanged, so every existing test sees the same trunk.
- **`fbs_trunk_matches_reference_at_a_non_square_multi_token_shape`:** width 64, MLP 96, 3 layers, vocabulary 512 (two tokens per writer cell), 8 kept cells, larger tables. Bars: tokens ≥ 504, kept cells ≥ 496 and queries within 2 LSB ≥ 480, of 512.
- **`fbs_writer_matches_reference_when_its_address_gains_are_tiny`:** width 64, 1 layer, vocabulary 512, 8 kept cells, with the address gains set to about 6e-4 as on seed 1. Bars: kept cells ≥ 500, tokens ≥ 500 and queries within 2 LSB ≥ 500, of 512.

## Measurements

- **Tokens per second of the repaired runtime on the laptop.** These are `fbs-toks` runs on the same Apple M1 as claude-126 (4 performance cores, 16 GiB, macOS 27.0, AC power), with 1 warmup and 20 timed runs each:

  | Seed | Run | Context (128-token prefill), median / p95 | Decode (64 tokens), median / p95 |
  |---|---|---|---|
  | 1 | first | 156 / 113 tok/s | 156 / 107 tok/s |
  | 1 | second | 263 / 222 tok/s | 265 / 190 tok/s |
  | 2 | first | 234 / 122 tok/s | 239 / 114 tok/s |

  - The machine was shared: its 1-minute load average was 8 to 10. The first seed-1 run's prefill took anywhere from 0.49 s to 1.31 s.
  - Each seed's runs wrote the same reply.
  - At this load, a slowdown from the repair cannot be resolved. The second seed-1 run is inside claude-126's range (medians 231 to 269). Seed 2's run is about 12% under its claude-126 run (269 and 268).
  - These time the integer runtime for this shape (width 256, 6 layers, vocabulary 4,096). The artifact's gap is not measured, so they are not a qualified serving rate.
- **The artifact is unchanged.** Re-exported from the stored snapshots, both artifacts have the sha256s that claude-126 stored (`314c3909…` and `d5b43596…`, in `d11gap/SHA256SUMS`). The repair changes how the runtime computes, not what is shipped.
- **The controls at the head.**
  - The tiny-gain control: 512 of 512 tokens, kept-cell sets and queries within 2 LSB.
  - The non-square control: 511 of 512 tokens, 511 of 512 kept-cell sets, and 510 of 512 queries within 2 LSB.
- **Their set-asides, each failing:**
  - the repair set aside: the tiny-gain control drops to 142 of 512 kept-cell sets and 421 of 512 tokens (re-run at the head). 509 of 512 queries still fall within 2 LSB, so a 2-LSB query tolerance alone cannot see a −0.088 LSB bias; the kept-cell check does;
  - the writer scanning the wrong tokens of a cell (a stride of 255): the non-square control drops to 227 of 512 tokens;
  - the down projection reading only the first width activations: the non-square control fails at once.

  The non-square control still passes with the repair set aside (510, 506 and 507 of 512), so the tiny-gain control is the one that pins the repair. The older square, one-token-per-cell control catches none of the three.
- **The diagnostics** came from temporary host-check tests comparing the integer session with the f64 quantized reference on the stored snapshots (`results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-<s>/snapshot.json`). They were removed before commit.

## Checks run at the head

See the exact-head review record on the PR.
- **Suites:**
  - integer serving library 325 and no-feature library 55;
  - `m5_full_path_cost_controls` 3/3 and `memory_shape` 3/3;
  - `fbs-compare` 1 and `fbs-toks` 2;
  - float-train 37 and 2.
- **The negative gate:** the repair set aside (`git diff origin/main` of `kernels.rs` and `session.rs`, reversed) fails the tiny-gain control; restored, it passes.
- **Build checks:**
  - zero-multiply audits FULL PASS (rlib 56 and 40; `uor-r4-stack` 44 and 99). The new `inline(never)` norm is in the audit's call-graph walk: with the repair set aside, the stack binary's walk reaches 98 functions;
  - CUDA type-check of float-train with a stub nvcc: exit 0;
  - clippy: no findings in the changed hunks;
  - `cargo fmt` clean.
- **Loop checks:** `loop report --check`, `loop pr-check --local` and `loop gates --full`.

## Skills used

- **holding-the-goal.** The goal was the artifact's own gap. A first reading called the runtime sound and blamed the trained trunk alone. The decision review rejected it for a measured bias, and following that bias found the defect.
- **writing-meaningful-tests.** Each new control fails on a set-aside the older control misses, and the tiny-gain control fails with the repair set aside.

## Still open

- **The decisive route: a GPU evaluator of the FBS container (decision D3).** This is the item's other route: an integer CUDA port of the session step, qualified bit-exact against the integer session on finite controls on the pod. With it, the artifact itself is scored on the 14,287-row panel and its export gap is read directly. It needs exact parity only with the integer session, which is integer against integer.
- **The residual's cause.** The Q16 control named above would test it. If it is precision, it is the artifact's own arithmetic, and it shows up in the export gap that the GPU evaluator measures.
- **The trained trunk's address collapse** is a hypothesis for the float-training line. Seed 1's writer-address gains fell about 3,000× below its other gains, leaving the cell choice at the rounding floor. A fix belongs in training and must come from the geometry, for example keeping the writer's address channels on the root shell. This unit does not test it.
- **The dense path's truncating norm,** and the mechanism page and the item's "What is missing" text, are outside this ORDER's scope.

## Plain summary

The last unit found that the shipped integer file disagreed with the GPU's emulation of it.
- **Part of that was a real bug, now fixed.** The integer file rounded one step down instead of to the nearest value, and then rounded again. On the first trained model this mattered, because training had shrunk the 16 numbers that pick the output's candidate group to almost nothing, so a tiny rounding bias changed the pick. With the fix, the file gives the specified token on every one sampled at one layer.
- **A small part remains.** At six layers, about 1 in 100 output tokens still differs. The cause is not yet tested; one candidate is the file's fixed-point arithmetic adding up over the layers. That is too much for the GPU emulation to vouch for the file at the precision needed.
- **Next:** run the integer file itself on the GPU and score it directly.
