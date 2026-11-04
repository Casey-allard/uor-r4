# Generated-state Hamming content readout

**Status:** FIT

## Question

Can an exact 256-bit Hamming nearest-example readout over actual generated
writer queries predict held-out reply content better than the exact writer and
a deterministic shuffled-target control, while retaining writer fallback
outside a training-only coverage threshold?

This is D84 CPU development research on the Ryzen 9 9950X. It does not change
integer serving and is not a language or GPU-qualification claim.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `acaeb3e490a1965539be15f6e17e3e7a6308cf92` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Artifacts:

- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-{1,2}-raw_both.uorlut`
- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-{tokens.u16,response-mask.u8,manifest.json}`

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

The manifest says `split=train`; this is a repeated-key, document-disjoint
development screen.

## Split and mechanism

Use 16 eight-token keys, even documents for training, odd documents for
evaluation and at most eight examples per key. Require 98 training and 103
evaluation samples. Derive memory from training only.

For each training sample, generate the reply trajectory with the exact writer
and memory path. Feed each generated token back through the complete CPU FBS
forward. At every decision with a reference target, pack the complete 256-Q12
writer query as 256 sign bits and store `(signature, reference target)`.

Fit one maximum Hamming distance covering at least 90% of leave-one-out
training rows. At decode, choose the commonest target among rows at the
minimum distance and break equal-count target ties by the smaller token id.
When the nearest distance exceeds the fitted threshold, use the exact writer
token.

The three arms are the exact writer, the true-target readout and a
deterministic target rotation over the same signatures and threshold. The
rotation is accepted only when the effective commonest-target map keyed by
unique signature changes; a row reordering that leaves every signature's vote
unchanged is a refused control, not a run.

## Preflight

The seed-1 selection path generated **894** rows over **98** training samples,
with **207** target values. The 90% threshold was distance **0** and covered
**891/894** rows. Leave-one-out target top-1 was only **199/894**, and
**892/894** rows had multiple target labels at their minimum distance.

This is a weak training-only nearest-target signal and is reported, not hidden
or converted into a pass. It remains inside the registered stop rules, whose
minimum is 640 rows, 16 target values and 90% fitted distance coverage. The
held-out writer-paired comparison decides this configuration.

The seed-1 sparse selection report was byte-identical between one-worker and
32-worker runs. The true map has **45** unique signatures. The seed-1 target
rotation changed all **45** effective assignments; the true/shuffled mapping
SHA-256 values are `dad81de4...c80edd` and `3d2da2d7...273cc853`.

## Arms and bar

1. exact writer with training-derived memory;
2. true-target Hamming readout with writer fallback; and
3. deterministic shuffled-target readout over identical signatures.

Both seeds must pass:

- at least **640** generated rows, at least **16** target values and a fitted
  threshold covering at least **90%** of leave-one-out rows;
- positive held-out readout exposure and positive writer fallback;
- the true readout's held-out aligned generated top-1 exceeds both writer and
  shuffled by the prompt-level paired rule
  `candidate_only >= max(2, ceil(3 * sqrt(control_only)))`, with an aggregate
  gain;
- true matching-prefix depth exceeds both controls by the same rule and has a
  positive aggregate gain;
- the shuffled mapping hash differs from the true mapping; and
- a seed-1 repeat matches every non-timing field and every generated evidence
  file.

Report exact references, stop classes, openings, nearest-distance and
ambiguity readings, distinct and repeated replies, workers, wall and RSS.
Exact replies are reported, not gated.

## Minimum test power

The 98/103 split and two independent model seeds are fixed. The prompt-level
paired rule requires an aggregate gain and three-sigma disagreement on both
seeds. Exact-reference counts are descriptive because the readout replaces
only individual decisions and may alter the subsequent generated trajectory.

## Instrument gates

- Source, inputs and binary hashes match.
- Rows come only from actual generated training trajectories.
- Every query is exactly 256 Q12 coordinates and every lookup is four XOR plus
  popcount operations at 32-bit distance.
- Threshold fitting is deterministic and covers at least 90% of leave-one-out
  rows.
- Shuffling preserves signatures and changes effective commonest-target
  assignments, not merely row order.
- Paired panels have matching prompt identities and lengths.
- Evaluation is source-ordered and identical across bounded worker counts.

## Stop rules

Stop on source/input/split drift, fewer than 640 rows, fewer than 16 target
values, less than 90% fitted coverage, a vacuous target shuffle,
nondeterminism, resource breach or any need to change serving,
integer-session, CUDA or Claude-owned files.

## Decision consequence

A pass makes generated-state nearest-example content readout a supported CPU
candidate for later serving-contract and parity work. A fail closes this
zero-threshold full-query sign signature, exact XOR distance, one-row vote and
writer-fallback configuration. It does not reject other vector spaces,
multiple-row votes, learned rotations, or another content readout geometry.
No outcome changes serving.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT and pre-run review | 600 s |
| release build | 300 s |
| two seeds plus repeat | 600 s |
| delivery and evidence | 300 s |
| total increment | **1,800 s** |

- Build: 16 jobs, 4 GiB RSS maximum, 2 GiB target maximum.
- Runs: 32 Rayon workers, 1 GiB RSS maximum, 600 s each.
- Retained pipeline: 64 MiB maximum.
- Post-run free-space floor: 30 GiB plus 128 MiB.
- Previous ledger:
  `cpu-training-pilot-query-admission-2026-11-04/resource-ledger.json` at
  `35,273.077487699 s` measured and `36,727.677487699 s` admitted cumulative.
- Admitted cumulative projection: **38,527.677487699 s**.

## Steps

After FIT and a fresh GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-content-readout
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/content-readout/2026-11-04-attempt-1
BUILD=$SOURCE/target/deepseek/content-readout-136
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=acaeb3e490a1965539be15f6e17e3e7a6308cf92
MIN_FREE_KIB=31588352
MAX_BUILD_BYTES=2147483648
MAX_PIPELINE_BYTES=67108864
free_kib() { df -Pk "$1" | awk 'NR == 2 { print $4 }'; }
test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
test -z "$(git -C "$SOURCE" status --porcelain -- crates)"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- crates/uor-r4-cpu-train
test ! -e "$PIPELINE"
test ! -e "$BUILD"
mkdir -p "$PIPELINE"
BUILD_STARTED=$(date -u +%s)
/usr/bin/time -v -o "$PIPELINE/build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" \
  cargo build --locked --release \
    -p uor-r4-cpu-train --bin cpu-content-readout-driver
BUILD_SECONDS=$(( $(date -u +%s) - BUILD_STARTED ))
BUILD_BYTES=$(du -sb "$BUILD" | cut -f1)
BUILD_RSS=$(awk -F': ' '/Maximum resident set size/ { print $2 }' \
  "$PIPELINE/build.time.txt")
test "$BUILD_BYTES" -le "$MAX_BUILD_BYTES"
test "$BUILD_RSS" -le 4194304
printf '{"jobs":16,"wall_seconds":%s,"peak_rss_kib":%s,"target_bytes":%s}\n' \
  "$BUILD_SECONDS" "$BUILD_RSS" "$BUILD_BYTES" >"$PIPELINE/build-summary.json"
BIN=$BUILD/release/cpu-content-readout-driver
sha256sum "$BIN" | sed 's# .*/#  #' >"$PIPELINE/binary.sha256"
run() {
  seed=$1 label=$2 artifact=$3
  out=$PIPELINE/$label
  /usr/bin/time -v -o "$PIPELINE/$label.time.txt" \
    timeout 600s env RAYON_NUM_THREADS=32 \
    "$BIN" "$DUAL/dual-c131-seed-${seed}-raw_both.uorlut" \
      "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" "$DEV/dev-manifest.json" \
      "$out" --expected-artifact-sha256 "$artifact" \
      --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
      --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
      --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
      --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
      --reply-cap 16 --seed 1 >"$PIPELINE/$label.stdout.json"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 1048576
  jq -e '.bar.rows_pass and .bar.target_diversity_pass
    and .bar.coverage_pass and .bar.exposure_pass
    and .bar.changed_mapping_assignments > 0
    and .bar.shuffled_mapping_differs
    and .selection.training_samples == 98
    and .selection.evaluation_samples == 103' \
    "$out/report.json" >/dev/null
  test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
  test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
}
run 1 seed-1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run 2 seed-2 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
run 1 seed-1-repeat a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
jq 'del(.wall_elapsed_ms, .peak_rss_kib)' \
  "$PIPELINE/seed-1/report.json" >"$PIPELINE/seed-1.normalized.json"
jq 'del(.wall_elapsed_ms, .peak_rss_kib)' \
  "$PIPELINE/seed-1-repeat/report.json" >"$PIPELINE/seed-1-repeat.normalized.json"
cmp "$PIPELINE/seed-1.normalized.json" "$PIPELINE/seed-1-repeat.normalized.json"
cmp "$PIPELINE/seed-1/true-readout.json" "$PIPELINE/seed-1-repeat/true-readout.json"
cmp "$PIPELINE/seed-1/shuffled-readout.json" "$PIPELINE/seed-1-repeat/shuffled-readout.json"
(cd "$PIPELINE" && find . -type f ! -name SHA256SUMS -print0 |
  sort -z | xargs -0 sha256sum >SHA256SUMS &&
  sha256sum -c SHA256SUMS --quiet)
test "$(du -sb "$PIPELINE" | cut -f1)" -le "$MAX_PIPELINE_BYTES"
```

Any source, input, binary, split, threshold, shuffle, worker-order,
deterministic repeat or resource mismatch is an instrument fault requiring a
new FIT.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 generated-state Hamming content readout sheet:
docs/harness/prep/cpu-training-pilot-content-readout-2026-11-04.md
head: acaeb3e490a1965539be15f6e17e3e7a6308cf92
implementation pin: acaeb3e490a1965539be15f6e17e3e7a6308cf92
reviewer: independent-subagent content_readout_rereview_fix
note: actual generated 256-bit sign signatures, 90% leave-one-out threshold,
exact XOR/popcount nearest target with writer fallback, shuffled assignment,
prompt-paired top-1/depth controls, ordered 32-worker evaluation and
seed-1 deterministic repeat
date: 2026-11-04

A FIT verdict clears compute for this configuration only (D22).
