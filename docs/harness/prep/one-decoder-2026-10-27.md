# Prep sheet: one decoder, the writer under `voronoi` past its collapse step (one-decoder, claude-150, 27 Oct 2026)

Lab: claude. Item: [one-decoder](../../roadmap/items/one-decoder.md). Order: [claude-150](../../loop/tasks/one-decoder/claude-150/ORDER.md). Code: branch `claude/one-decoder`. Every pod builds and tests its own binary from the pinned commit first.

TEST FITNESS: FIT (round 3 of a fresh-context review; with the D67 pre-run review PASS in the same round, light gates per the owner on #2028. Round 1 found I-2 could false-alarm, round 2 that its tie account was wrong; I-2 is now equality, gated at both seeds' probes and over the run.)

## The question

**Under the `voronoi` objective, does the served decoder (the writer) hold, without collapse and level with whole-table nearest, past the step where the writer objective collapsed?** If it does, the writer stays the served decoder. If not, serving moves to whole-table nearest.

This is unit U2 of the [pipeline map](../../model/pipeline-map.md). The serving session decodes with the writer; every float run is evaluated with whole-table nearest; on one checkpoint they disagree, 345 against 7,813 (the map's stage 7). The writer objective's long run (`long2/g-long`, claude-142) peaked at 7,593 at step 12,288 and collapsed to 1,363 by step 18,432. `voronoi`, whose address margin can be met, gave the best writer score, 7,906, but only ever ran 16,384 steps, so it never met that regime.

## Steps: the arms

One arm, two seeds, one RTX 5090 pod per seed, each arm alone on its GPU:

`train-in-space --objective voronoi --retain-rates 1 --decay-mult 1 --batch 16 --steps 32768 --eval-every 2048 --lr 0.0005 --lr-end 0 --samples 6 --reply-tokens 48 --seed S`, for `S` in 1 and 2.

This is `long2/g-long`'s schedule exactly (its `config.json`: 32,768 steps, batch 16, lr 5e-4 → 0, retention on, decay multiplier 1), with `--objective voronoi` in place of the writer objective. The runner's options added since `long2` (`--fold-scores`, `--reflect-floor`, `--fix-input-table`, `--flock-read`, `--content-read`, `--hamming-read`) all default off (`train-in-space.rs`, lines 58–63), so the model is `long2`'s with only the objective changed. Every checkpoint (17, including step 0) logs, on all 14,287 scored DEV rows:
- `dev_top1_hits`: the writer panel's top-1;
- `dev_writer`: the writer's top-1, openings and distinct opening picks on the same rows and points as `dev_nearest` (new in this unit);
- `dev_nearest`: whole-table nearest's top-1, openings and distinct opening picks.

At the end, `--decode-speed` on the step-32,768 checkpoint, once with `--objective voronoi` (the writer) and once with `--objective nearest`, so both decoders are timed on one model.

## Bar (pre-registered), at both seeds

Let `W(s)` be the writer's top-1 at step `s` and `N(s)` whole-table nearest's; `Wo` and `No` their openings.

**The writer holds** when both hold at a seed:
1. **No collapse:** at every checkpoint after step 16,384, `W(s) ≥ 0.7 · max(W(t), t ≤ s)`. (`long2` fell to 18% of its peak; 70% leaves room for checkpoint noise.)
2. **Level with nearest at the end:** `W(32,768) ≥ N(32,768) − 500` and `Wo(32,768) ≥ No(32,768) − 36`.

**Decision:**
- **The writer holds at both seeds:** the writer stays the served decoder.
- **Otherwise,** a split included: the decision is to serve whole-table nearest. A decoder that fails at one seed of two is not the one to serve. The REPORT then gives nearest's per-token cost at 4,096 tokens (1,048,576 absolute differences) and the vocabulary at which a sublinear index is needed. Changing the serving session is its own unit.

**Reported, not barred:**
- `W` and `N` at every checkpoint, both seeds;
- each decoder's openings and distinct picks;
- the writer's greedy replies (`samples.md`; for `voronoi` the runner's replies are the writer's);
- both decoders' decode tokens/s against context.

## Instrument checks

- **I-1:** the crate's CUDA library tests pass on each pod's build.
- **I-2:** `dev_writer.hits = dev_top1_hits` at every checkpoint. With every read off, as in this arm, both decode the same forward pass: `flock_read::points` returns `model.hidden` unchanged when no read is set (`flock_read.rs`, `points`), and the panel reads `model.hidden` in the same gate mode (`HamiltonActive`, same seed and step, export mode and telemetry off). Both use the same writer (8 kept cells, `set_native_vector_head(SHEET_KEPT_CELLS)`) and the same tie rule: `write()` and `top_token_ids` both take the smallest token id among tied best scores, and the panel itself calls `writer.write`. So only a float difference between two runs of the same forward pass could separate them. **Any nonzero gap is an instrument fault, explained before any bar is read;** the probe stops the job on one before the long arms.
- **I-3:** each seed's step-0 `dev_nearest` equals claude-145's no-read twin at the same seed (same `build_fbs_model(OneAtomActive, seed)`, nothing trained): seed 1 nearest 2, openings 0, 730 distinct picks; seed 2 nearest 5, openings 3, 681 picks.
- **I-4:** no non-finite loss (the runner stops on one).

## Probe ladder

1. **The build and the CUDA tests** on each pod, from the pinned commit.
2. **A 256-step throughput probe** at each seed, on its own pod, before its long arm: it projects the wall time and reads I-2 and I-3 at steps 0 and 256. The job stops before the long arm if I-2 fails there.
3. **The two arms,** one per pod.

## Data split

The pinned TRAIN and DEV splits the in-space trunk always uses, checked by sha256 at load (`hamilton_sheet.rs`: `PINNED_TRAIN_TOKENS_SHA256`, `PINNED_TRAIN_MASK_SHA256`, `PINNED_DEV_TOKENS_SHA256`, `PINNED_DEV_MASK_SHA256` and `PINNED_TOKENIZER_SHA256`), from `caseyallard/uor-r4-store:w2-inputs/2026-10-25/driver-data/` (`tokenizer.json`, `mw-balp/train/`, `ft/dev/`). DEV has 14,287 scored rows, 1,226 of them reply openings.

## Base artifact

None. Each arm starts from `build_fbs_model(OneAtomActive, seed)`.

## Minimum test power

The two decoders read the same model's points, so no run-to-run noise enters their difference: the 500-row and 36-opening margins are **practical-equivalence tolerances**, not noise bounds. 500 rows is 3.5% of DEV's 14,287; one count's binomial standard deviation at a top-1 rate near 0.55 is about 60 rows. 36 openings is carried over from claude-145's sheet, where it is three binomial standard deviations at the commonest opening's rate on 1,226 rows. Either way, a stage-7-sized gap (345 against 7,813) cannot pass, and `long2`'s fall to 18% of its peak fails the 0.7 collapse threshold by far. The collapse test compares a run with its own earlier peak. The collapse is known at seed 1 only (`long2`); seed 2 has no writer-objective reference.

## Cost

Two RTX 5090 pods at about `$0.56/h`. A writer-objective run alone trained at 35,597 context tokens/s (claude-142), so 32,768 steps × 16 × 192 tokens is about 48 minutes of training per arm, plus 17 evaluations. Projected under `$2` in all; the throughput probe checks it before the long arms.

## Stop rule

- Stop on a non-finite loss, a crash, or an I-2 mismatch. A healthy run is never cut short.
- Results go to `caseyallard/uor-r4-store:results/claude/one-decoder-2026-10-27/`, sealed with a SHA256SUMS, and are read back against the pod before any pod comes down.
