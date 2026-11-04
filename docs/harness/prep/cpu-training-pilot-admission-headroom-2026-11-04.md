# Generated-state writer admission headroom

**Status:** FIT

## Question

Can the existing native writer's maximum K=16 cell budget expose enough
admitted, positive-margin targets on the model's generated states to resume
writer training, after K=8 exposed only about 30 repairable decisions per
public seed?

This is D84 CPU development research on the Ryzen 9 9950X. It performs no
training update and does not change integer serving or establish language
quality.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `5565bf9b432b4937fb4de8d532321f20966bee1d` |
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

## Data split and mechanism

Use 16 eight-token keys, even documents for training, odd documents for
evaluation and up to eight examples per key. Require 98 training and 103
evaluation samples. Derive memory from training only.

Scan generated and reference-feedback states through exact CPU FBS and memory.
For each observed decision:

1. rank `reference_target & 255` among all 256 writer cells;
2. evaluate target admission, positive margin and writer top-1 at
   K=1,2,4,8,12,16;
3. evaluate the same decision with the top eight plus eight deterministic
   non-top cells; and
4. patch the K=8 artifact to K=16 by changing only `shape.kept_cells`, then
   execute it through the same CPU FBS and integer serving audit.

The K=16 patch may move table payloads to a new alignment but must not change
any table payload byte.

## Arms and bar

Both seeds must pass:

- K=16 generated-state training positive-margin decisions are at least
  **640**, with a gain of at least **610** over K=8 and at least **64**
  positive-margin same-cell pairs;
- K=16 adds at least **128** admitted held-out target decisions over K=8;
- K=16 fixed-horizon matching depth is not below K=8;
- teacher-forced top-1 is not below K=8;
- K=16 admits more held-out targets and positive-margin target cells than the
  equal-budget random-extra cells;
- every table payload byte is identical between K=8 and K=16;
- K=8 and K=16 generated token sequences have zero CPU/integer query, token
  and kept-cell mismatches; and
- a seed-1 repeat matches every non-timing field and artifact.

Rank histograms, minimum-K readiness, exact references, stops, workers, wall
and RSS are reported. Exact replies are not a passing bar.

## Minimum test power

Two independent public model seeds must pass. The 640-row threshold matches
the writer-training readiness gate from deepseek-133; the +128 held-out row
threshold is a bounded exposure check, not a language bar.

## Instrument gates

- Source, inputs and binary hashes match.
- K=8 is the unchanged source artifact with memory attached.
- K=16 parses, reports `kept_cells=16`, and preserves all table payload bytes.
- Cell ranks contain all 256 unique cells and deterministic lower-id ties.
- Random-extra cells are disjoint from top eight and deterministic.
- Generated and reference scans use the established reply-start/memory path.
- Pairwise panels have equal length and identical prompt ids.

## Stop rules

Stop on source/input/split drift, a changed table payload, a malformed K=16
artifact, any CPU/integer mismatch, non-determinism, resource breach or any
need to change serving, integer-session, CUDA or Claude-owned files. Keep every
bounded negative/insufficiency result without retuning K.

## Decision consequence

A pass establishes K=16 admission headroom and unlocks the following
generated-prefix writer-training unit. If K=16 remains below 640 positive
margin rows, close the fixed existing 8→16 admission-budget configuration and
move to direct generated-query/admission training. No outcome changes serving
or closes #3506.

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
  `cpu-training-pilot-generated-prefix-2026-11-04/resource-ledger.json` at
  `34876.977487699 s`.
- Admitted cumulative projection: **36,676.977487699 s**.

## Steps

After FIT and a fresh GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-admission-headroom
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/admission-headroom/2026-11-04-attempt-1
BUILD=$SOURCE/target/deepseek/admission-headroom-134
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=5565bf9b432b4937fb4de8d532321f20966bee1d
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
mkdir -p "$PIPELINE/selection"
BUILD_STARTED=$(date -u +%s)
/usr/bin/time -v -o "$PIPELINE/build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" \
  cargo build --locked --release \
    -p uor-r4-cpu-train --bin cpu-admission-headroom-driver
BUILD_SECONDS=$(( $(date -u +%s) - BUILD_STARTED ))
BUILD_BYTES=$(du -sb "$BUILD" | cut -f1)
BUILD_RSS=$(awk -F': ' '/Maximum resident set size/ { print $2 }' \
  "$PIPELINE/build.time.txt")
test "$BUILD_BYTES" -le "$MAX_BUILD_BYTES"
test "$BUILD_RSS" -le 4194304
printf '{"jobs":16,"wall_seconds":%s,"peak_rss_kib":%s,"target_bytes":%s}\n' \
  "$BUILD_SECONDS" "$BUILD_RSS" "$BUILD_BYTES" >"$PIPELINE/build-summary.json"
BIN=$BUILD/release/cpu-admission-headroom-driver
sha256sum "$BIN" | sed 's# .*/#  #' >"$PIPELINE/binary.sha256"
for seed in 1 2; do
  case "$seed" in
    1) artifact=a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd ;;
    2) artifact=da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 ;;
  esac
  "$BIN" "$DUAL/dual-c131-seed-${seed}-raw_both.uorlut" \
    "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" "$DEV/dev-manifest.json" \
    "$PIPELINE/selection/seed-$seed.out" \
    --expected-artifact-sha256 "$artifact" \
    --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
    --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
    --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
    --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
    --reply-cap 16 --seed 1 --select-only >"$PIPELINE/selection/seed-$seed.json"
  jq -e '.selection.training_samples == 98
    and .selection.evaluation_samples == 103
    and .artifacts.payload_identical == true' \
    "$PIPELINE/selection/seed-$seed.json" >/dev/null
done
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
  jq -e '.bar.payload_identical == true
    and .bar.all_parity_exact == true
    and .inputs.k8_sha256 != .inputs.k16_sha256' "$out/report.json" >/dev/null
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

Any source, input, binary, shape, payload, control, parity or deterministic
mismatch is an instrument fault requiring a new FIT before retry.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 generated-state admission headroom
sheet: docs/harness/prep/cpu-training-pilot-admission-headroom-2026-11-04.md
head: d5d742c198ef3c978d8341c7b36f42879c9992aa
implementation pin: 5565bf9b432b4937fb4de8d532321f20966bee1d
reviewer: independent-subagent admission-headroom-fit-2
note: two-seed D84 K=8/K=16 generated-trajectory census; actual K16
eligibility and held-out admissions, 640-row/64-pair readiness, equal-budget
random-extra cells, header-only artifact patch, exact parity and deterministic
repeat
date: 2026-11-04

A FIT verdict clears compute for this configuration only (D22).
