# Prep sheet: centroid refit on the current D3 checkpoint (one-model-end-to-end, deepseek-16, 6 Nov 2026)

Lab: deepseek. Item: one-model-end-to-end (#3407). Order:
[deepseek-16](../../loop/tasks/one-model-end-to-end/deepseek-16/ORDER.md).

TEST FITNESS: PENDING - this execution package is complete, but it has not received a
fresh non-author review. No capture or evaluation is authorized until that review writes
`TEST FITNESS: FIT`.

## Question

On the current stable D3/8,192-step FBS checkpoint at seeds 1 and 2, does replacing
eligible output-table rows with the mean TRAIN query for each target improve the
whole-table nearest top-1, beyond the original table and an equal-observation-count
shuffled-label control, without lowering top-8?

This tests the output readout only. The trunk, input query path, table cells and writer
admission remain frozen.

## Why now

Claude-297 found that D3 moves the trunk at both seeds, while (n3S) remains behind its
TRAIN's T2. Its REPORT explicitly leaves centroid refit of the readout as the next unit.
DeepSeek-237 rejected median and mean-centroid rows on two old public artifacts, so the
prior is negative. This run asks whether that negative survives on the current
human-text checkpoint and the active scorer.

## Base artifact

The base is the current pinned FBS implementation plus the two Claude-297 n3S
step-008192 freeze archives listed below. The implementation pin is the merged parent of
deepseek-16; the exact capture binary hash is recorded on the pod.

## Raw data

| Input | Store path | Identity |
|---|---|---|
| D3 TRAIN | `results/claude/scale-probe-2026-11-05/inputs/d3` | tokens `ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52`; mask `ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875`; 27,416,554 scored |
| oasst2 heldout | `results/claude/real-text-corpus-2026-11-05/oasst2/heldout` | tokens `02a04e97bbb8936c993bf5eff0f29c171ebeae88988217a4100fdc77701f169e`; mask `9c5bb6567c6a4cded2dfb2280ae4acdc798e283a3beae8aaebc695656e1b4833` |
| wiki-00000 heldout | `results/claude/real-text-corpus-2026-11-05/wiki-00000/heldout` | tokens `82b3a40c3ffe2cabd9164f636d9c2710fb054e171913eca944ebf819b70209f7`; mask `34fe6b68e0d039b914bb1cde789223ef463b37ea77af7f3d77ab90eb0977386f` |
| wiki-00020 heldout | `results/claude/real-text-corpus-2026-11-05/wiki-00020/heldout` | tokens `9b7f1432b105f161ed8b54b01900d8fa791efc6dbfd5ad1d62a8d4d94a86d640`; mask `a6803ab3de3fae60c382416639ffa583c26ac822db78a37afd2c2d718767e00d` |
| wiki-00040 heldout | `results/claude/real-text-corpus-2026-11-05/wiki-00040/heldout` | tokens `c927243b540bccdda8af8fb5a11aafa8b9041d4af30dd614f23d2ebb9aa5962d`; mask `9d95ea6de0eba07d4cdbc258b5fa6ae88def952f062d166ba417d783724fa625` |

The four held-out roots contain exactly 1,460,697 scored rows. Their pooled identity is
`5f88a5b3295ba5d684a8c75206787642fd65a2377d2dbb268992ec5c71a621eb`.

## Data split

Training uses D3 only. Evaluation uses the four complete held-out roots. No panel row is
used to fit a centroid, and no D3 row is scored as a panel row.

### Checkpoints

The archives are:

- seed 1:
  `results/claude/one-model-end-to-end-steps-data-2026-11-06/runs/s1/steps-data-s1.tar.gz`,
  SHA-256 `1c7a09781c60475067b4fc2319db92a16ef0f589dcf355db1d09f35edcc4ec69`;
- seed 2:
  `results/claude/one-model-end-to-end-steps-data-2026-11-06/runs/s2/steps-data-s2.tar.gz`,
  SHA-256 `dd3cc8885cf1b35aa840da9f7d029d8a8dc6097a8321d963e4abdfcc8ce5c467`.

Before capture, the job must verify each archive hash, extract it into a fresh directory,
and require its inner `SHA256SUMS` to pass. It must locate exactly one
`n3S/models/step-008192/` checkpoint in each archive and record the config and model-file
hashes in the run log. More or fewer matches is a stop.

## Steps

Run once per seed on one RTX 5090 through `scripts/pod/uor-pod`:

```sh
train-in-space \
  --data-root "$DATA" \
  --score-rows "$CHECKPOINT" \
  --out "$OUT" \
  --train-split "$D3" \
  --train-sha256 ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52 \
  --train-mask-sha256 ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875 \
  --panel oasst2="$PANEL/oasst2" \
  --panel wiki-00000="$PANEL/wiki-00000" \
  --panel wiki-00020="$PANEL/wiki-00020" \
  --panel wiki-00040="$PANEL/wiki-00040" \
  --samples 0 \
  --retain-rates 1 \
  --fold-scores 0 \
  --reflect-floor 0 \
  --seed "$SEED" \
  --context 192 \
  --served-forward 1 \
  --query-cache-out "$CACHE" \
  --query-cache-memory 0
```

The scorer may still perform its ordinary memory-on pass for normal output files, but the
cache records only no-memory. A capture run writes no samples and trains nothing.

## Memory control

The cache captures:

- each query immediately before a scored target;
- target and corpus position;
- frozen whole-table nearest and writer decisions;
- opening/copy flags;
- the unchanged shared table.

TRAIN fitting uses only targets with at least eight observations. The control groups
eligible labels by exact observation count and cycles each group's centroid rows once,
preserving the multiset of row counts and the global query set.

## CPU evaluation

```sh
fbs-query-cache evaluate "$CACHE" \
  --train train \
  --panel oasst2 \
  --panel wiki-00000 \
  --panel wiki-00020 \
  --panel wiki-00040 \
  --memory 0 \
  --minimum 8 \
  --out centroid.json
```

The same command runs for both seed caches.

## Instrument checks

- **I-0:** the float-training library tests pass at the capture pin.
- **I-1:** every input file matches the hashes above.
- **I-2:** the archive hash and inner checkpoint `SHA256SUMS` pass.
- **I-3:** each cache manifest names the expected checkpoint, D3 hashes, four panel hashes,
  query space, table hash and no-memory split.
- **I-4:** the CPU original table reproduces the frozen nearest decision at every panel
  row. Any difference is `CONTROL_FAULT`.
- **I-5:** each panel has 1,460,697 pooled rows with identity `5f88a5b3...`.
- **I-6:** the matched shuffle changes at least one eligible row and preserves every
  target's observation count.

## Minimum test power

- at least 100 fitted token rows;
- at least 10,000 TRAIN observations across fitted rows;
- all four panels and all 1,460,697 rows;
- both seeds.

Insufficient power is `INSUFFICIENT_POWER`, not a negative.

## Bar

For each seed, pool the four panels:

- original decisions must match every frozen nearest decision exactly;
- centroid top-8 must be at least original top-8;
- centroid top-1 must exceed original and shuffled-control top-1;
- centroid-only wins against original must satisfy
  `candidate_only >= max(2, ceil(2 * sqrt(candidate_only + control_only)))`;
- the same paired rule must hold against the matched shuffle.

The mechanism is retained only if all conditions hold at both seeds. Otherwise this
closes mean-centroid refit for the tested n3S configuration.

## Cost and stop

- Two sequential one-card captures, each reading 27.42M TRAIN rows and 1.46M panel rows.
- Estimate: 2-4 hours total and at most $5 at the observed 5090 rates.
- Each no-memory cache is about 29 GiB; stop below 80 GiB free.
- Stop on any input/checkpoint mismatch, `CONTROL_FAULT`, underpowered fit, archive
  ambiguity, or missing store/pod credentials.

## Outcome use

- **Positive at both seeds:** keep centroid as an output-only candidate and test it in
  the next active served-readout comparison. It is not yet a language or milestone result.
- **Negative:** close mean-centroid refit on n3S and move to a rank-aware or two-table
  output objective. Do not retune the threshold or shuffle after opening the panels.
- **Control fault:** repair the capture/readout identity before interpreting any centroid
  number.
