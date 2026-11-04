# Generated-query final-gain admission training

**Status:** FIT

## Question

Can direct final-gain training on actual generated trajectories move enough
reference target cells into K=8 writer admission to resume writer training,
without losing generated depth or teacher-forced top-1?

This is D84 CPU development research on the Ryzen 9 9950X. It does not change
integer serving and is not a language or GPU-qualification claim.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `266de4a06ae7e50ecd849950495cb3a2f4b95b4a` |
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

The K=8 CPU trainer scans actual generated trajectories after every accepted
change. Its objective counts observed target cells admitted in the top eight,
target-cell rank sum, distinct admitted cells and same-cell pairs. Missing
fixed-horizon decisions may not increase.

Candidate final-gain coordinates are fixed to `0,8,...,248`; each trial moves
one coordinate by `+/-32`. Select the lexicographic best safe candidate and
accept at most 12 moves.

## Arms and bar

1. frozen K=8 artifact;
2. true-target final-gain training;
3. deterministic cross-cell target permutation; and
4. an equal-move deterministic random-gain control.

Both seeds must pass:

- true K=8 positive-margin training rows reach **640**, with at least **64**
  same-cell pairs;
- true held-out admitted targets exceed frozen by **128** and exceed the
  permuted and random controls;
- held-out fixed-horizon matching depth and teacher-forced top-1 do not fall;
- all artifact changes are inside `final_norm_gain`;
- all arms retain exact CPU/integer generated-sequence parity; and
- a seed-1 repeat matches every non-timing field and artifact.

Report rank histograms, exact references, stops, moves, workers, wall and RSS.
Exact replies are not a bar.

## Minimum test power

The 98/103 split and two independent model seeds are fixed. The 640-row and
64-pair thresholds match the writer-training readiness gate. The 128 held-out
admitted-target addition is a bounded coverage bar, not a language result.

## Instrument gates

- Source, inputs and binary hashes match.
- Candidate coordinates and signs are deterministic and bounded.
- A target permutation changes assignments while preserving target counts.
- Random moves equal the true arm's accepted move count.
- Gain-only byte comparison excludes every non-gain artifact byte.
- Paired panels have matching prompt identities and lengths.

## Stop rules

Stop on source/input/split drift, gain-table byte leakage, malformed target
permutation, parity mismatch, nondeterminism, resource breach or any need to
change serving, integer-session, CUDA or Claude-owned files.

## Decision consequence

A pass makes generated-query final-gain admission a supported CPU candidate
and unlocks resuming generated-prefix writer training. A fail closes this
fixed 32-coordinate/12-move objective and redirects work to another query
coordinate family or trunk-level admission path. No outcome changes serving.

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
  `cpu-training-pilot-admission-headroom-2026-11-04/resource-ledger.json` at
  `34927.677487699 s`.
- Admitted cumulative projection: **36,727.677487699 s**.

## Steps

After FIT and a fresh GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-query-admission
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/query-admission/2026-11-04-attempt-1
BUILD=$SOURCE/target/deepseek/query-admission-135
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=266de4a06ae7e50ecd849950495cb3a2f4b95b4a
MIN_FREE_KIB=31588352
MAX_BUILD_BYTES=2147483648
MAX_PIPELINE_BYTES=67108864
free_kib() { df -Pk "$1" | awk 'NR == 2 { print $4 }'; }
test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
test -z "$(git -C "$SOURCE" status --porcelain -- crates docs/harness/prep)"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- crates/uor-r4-cpu-train
test ! -e "$PIPELINE"
test ! -e "$BUILD"
mkdir -p "$PIPELINE"
BUILD_STARTED=$(date -u +%s)
/usr/bin/time -v -o "$PIPELINE/build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" \
  cargo build --locked --release \
    -p uor-r4-cpu-train --bin cpu-query-admission-driver
BUILD_SECONDS=$(( $(date -u +%s) - BUILD_STARTED ))
BUILD_BYTES=$(du -sb "$BUILD" | cut -f1)
BUILD_RSS=$(awk -F': ' '/Maximum resident set size/ { print $2 }' \
  "$PIPELINE/build.time.txt")
test "$BUILD_BYTES" -le "$MAX_BUILD_BYTES"
test "$BUILD_RSS" -le 4194304
printf '{"jobs":16,"wall_seconds":%s,"peak_rss_kib":%s,"target_bytes":%s}\n' \
  "$BUILD_SECONDS" "$BUILD_RSS" "$BUILD_BYTES" >"$PIPELINE/build-summary.json"
BIN=$BUILD/release/cpu-query-admission-driver
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
      --reply-cap 16 --moves 12 --seed 1 >"$PIPELINE/$label.stdout.json"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 1048576
  jq -e '.bar.all_parity_exact == true
    and .bar.permutation_cross_cell_assignments > 0
    and .bar.random_moves == .arms.true.accepted
    and (.arms.true.accepted == 0
      or (.bar.gain_only and .bar.permuted_gain_only and .bar.random_gain_only))' \
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
(cd "$PIPELINE/seed-1" && sha256sum ./*.uorlut) >"$PIPELINE/seed-1.artifacts.sha256"
(cd "$PIPELINE/seed-1-repeat" && sha256sum ./*.uorlut) \
  >"$PIPELINE/seed-1-repeat.artifacts.sha256"
cmp "$PIPELINE/seed-1.artifacts.sha256" \
  "$PIPELINE/seed-1-repeat.artifacts.sha256"
(cd "$PIPELINE" && find . -type f ! -name SHA256SUMS -print0 |
  sort -z | xargs -0 sha256sum >SHA256SUMS &&
  sha256sum -c SHA256SUMS --quiet)
test "$(du -sb "$PIPELINE" | cut -f1)" -le "$MAX_PIPELINE_BYTES"
```

Any source, input, binary, split, move, permutation, byte-isolation, parity or
deterministic mismatch is an instrument fault requiring a new FIT.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 generated-query final-gain admission training
sheet: docs/harness/prep/cpu-training-pilot-query-admission-2026-11-04.md
head: 5f65d260f26c8850b9ec9c73a7e1a4d29982ec3d
implementation pin: 266de4a06ae7e50ecd849950495cb3a2f4b95b4a
reviewer: independent-subagent query-admission-fit-2
note: actual generated-trajectory final-gain objective, fixed 32-coordinate
family, cross-cell target permutation with mapping audit, equal-move random
control with per-move identity, 640-row/64-pair readiness, depth and teacher
non-loss, gain-only isolation, parity and deterministic repeat
date: 2026-11-04

A FIT verdict clears compute for this configuration only (D22).
