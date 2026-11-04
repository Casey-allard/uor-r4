# Generated-prefix writer training

**Status:** FIT

## Question

Can the exact CPU writer learn from the model's own generated prefixes well
enough to delay divergence, while reference-prefix, static-prefix and
same-cell shuffled-label controls separate the effect on both public model
seeds?

This is D84 CPU development research on the Ryzen 9 9950X. It does not change
integer serving and is not GPU qualification, complete-reply correctness,
fresh-phrasing transfer or useful conversation.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `77524f3e47422a4c86578b2798bc98cd6d6d83b1` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Artifacts:

- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-{1,2}-raw_both.uorlut`
- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-{tokens.u16,response-mask.u8,manifest.json}`

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

The public manifest says `split=train`. This is a repeated-key,
document-disjoint screen, not unseen-key, fresh-phrasing or language evidence.

## Data split

Use 16 eight-token keys, even documents for training, odd documents for
evaluation and up to eight examples per key. Derive opening/reply memory from
training examples only.

Expected prior selection is 98 training and 103 evaluation examples. The run
must report actual counts. Each reply has a 16-content-token horizon.

## Mechanism

Reset one exact `CpuFbsForward` per prompt. Call `mark_reply_start()` before the
final prompt token, then:

1. record the live Q12 writer query and selected token at reply offset `j`;
2. align it to the reference content token at offset `j`;
3. generate the next query from the model's own selected token; and
4. apply one existing K=1 target/rival BVG-K writer update when the target cell
   is admitted and margin loss is positive.

Candidate acceptance is lexicographic over aligned generated top-1, target rank
sum and target L1 distance over every fixed-horizon prompt. Early BOS therefore
cannot win by removing later decisions. Each accepted artifact is reparsed,
audited against integer serving, and used for the next dynamic update.

Reference mode feeds the reference token at the same offset. Static mode
captures frozen generated queries once. Dynamic mode recaptures after accepted
updates. Same-cell shuffle rotates targets within each target-cell group,
preserving the target multiset and admission class.

## Arms

1. frozen memory artifact;
2. reference-prefix true labels;
3. static generated-prefix true labels;
4. static generated-prefix same-cell shuffled labels;
5. dynamic generated-prefix true labels; and
6. dynamic generated-prefix same-cell shuffled labels.

Every trained arm gets exactly 80 K=1 attempts. No outcome-based retune.

## Eligibility gate

Before training, each seed must find at least 640 generated training decisions
with an admitted target and positive margin loss, across at least 16 prompts
and 8 target cells, with at least 64 same-cell shuffle pairs. Failure is an
insufficiency result and spends no training compute.

## Bar

Both seeds must pass:

- dynamic true has positive summed held-out matching-prefix depth over frozen;
- dynamic true has positive summed depth over dynamic shuffled;
- each paired depth comparison has at least
  `max(2, ceil(3 * sqrt(b + c)))` candidate-only wins; and
- dynamic true aligned generated top-1 exceeds both frozen and dynamic
  shuffled.

Dynamic true teacher-forced content top-1 must not fall below frozen. Every arm
must retain exact CPU/integer query-coordinate, token and kept-cell parity.

Report exact references, BOS/cap/context stops, opening accuracy,
first-divergence histogram, admission, admitted-only accuracy,
distinct/repeated replies, effective workers, wall and peak RSS.

## Minimum test power

The fixed panel has 103 held-out prompts. Both independent model seeds must
pass. The paired threshold
`max(2, ceil(3 * sqrt(b + c)))` requires at least two candidate-only wins
after discordance, and the depth and aligned-top-1 differences must both have
the registered direction. This is a bounded configuration screen, not a
precision or language-capability result.

## Instrument gates

- Source, inputs and binary hashes match.
- Generated and reference feedback produce different query states.
- Opening and reply memory are active in the scanned trajectory.
- Fixed-horizon accounting counts decisions missing after early BOS as wrong.
- Same-cell shuffles preserve cells and counts and change targets.
- No admitted generated row or requested shuffle is vacuous.
- Seed-1 repeat reproduces every non-timing and non-resource field and artifact.
- Integer-serving files and the serving contract remain unchanged.

## Stop rules

Stop on source, split, panel, input or binary drift; a vacuous same-cell
shuffle; no eligible generated rows; any CPU/integer parity mismatch; a
teacher-forced top-1 loss; nondeterminism; or any need to change serving,
integer-session, CUDA or Claude-owned files. An eligibility failure is
retained as an insufficiency result without training compute.

## Decision consequence

A pass makes generated-prefix writer training a supported CPU candidate for the
next content-training unit and justifies bringing the same mechanism to a GPU
qualification sheet. A fail closes only this K=1, 80-attempt, generated-prefix
configuration and redirects work to admission/query training or another
content mechanism. An eligibility failure changes no mechanism status. No
outcome changes integer serving or closes #3506.

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
  `cpu-training-pilot-stop-signature-2026-11-04/resource-ledger.json` at
  `34856.977487699 s`.
- Admitted cumulative projection: **36,656.977487699 s**.

## Steps

After this FIT and a fresh GO, run this exact recipe from the named worktree:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4-generated-prefix
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/generated-prefix/2026-11-04-attempt-1
BUILD=$SOURCE/target/deepseek/generated-prefix-133
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=77524f3e47422a4c86578b2798bc98cd6d6d83b1
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
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR=$BUILD \
  cargo build --locked --release \
    -p uor-r4-cpu-train --bin cpu-generated-prefix-driver
BUILD_SECONDS=$(( $(date -u +%s) - BUILD_STARTED ))
BUILD_BYTES=$(du -sb "$BUILD" | cut -f1)
BUILD_RSS=$(awk -F': ' '/Maximum resident set size/ { print $2 }' \
  "$PIPELINE/build.time.txt")
test "$BUILD_BYTES" -le "$MAX_BUILD_BYTES"
test "$BUILD_RSS" -le 4194304
printf '{"jobs":16,"wall_seconds":%s,"peak_rss_kib":%s,"target_bytes":%s}\n' \
  "$BUILD_SECONDS" "$BUILD_RSS" "$BUILD_BYTES" >"$PIPELINE/build-summary.json"
BIN=$BUILD/release/cpu-generated-prefix-driver
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
    --reply-cap 16 --attempts 80 --seed 1 --select-only \
    >"$PIPELINE/selection/seed-$seed.json"
  jq -e '.preflight.eligible_decisions >= 640
    and .preflight.prompts >= 16 and .preflight.target_cells >= 8
    and .preflight.same_cell_shuffle_pairs >= 64
    and .shuffled_preflight.eligible_decisions >= 640
    and .shuffled_preflight.prompts >= 16
    and .shuffled_preflight.target_cells >= 8
    and .shuffled_preflight.same_cell_shuffle_pairs >= 64' \
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
      --reply-cap 16 --attempts 80 --seed 1 \
      >"$PIPELINE/$label.stdout.json"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 1048576
  jq -e '.bar.all_parity_exact == true
    and .bar.static_shuffled_accepted > 0
    and .bar.dynamic_shuffled_accepted > 0' "$out/report.json" >/dev/null
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

Any source, input, binary, split, control, parity or deterministic mismatch is
an instrument fault requiring a new FIT before retry.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 generated-prefix writer training
sheet: docs/harness/prep/cpu-training-pilot-generated-prefix-2026-11-04.md
head: efd9c9781690a6b3a121408dc83ac08126bad443
implementation pin: 77524f3e47422a4c86578b2798bc98cd6d6d83b1
reviewer: independent-subagent generated-prefix-fit-3
note: two-seed D84 screen; live generated-prefix writer updates through exact
FBS/memory, reference and static/dynamic controls, fixed-horizon targets,
same-cell shuffled labels, admitted-row eligibility, exact parity, worker and
resource gates, deterministic repeat, and no serving change
date: 2026-11-04

A FIT verdict clears compute for this configuration only (D22).
