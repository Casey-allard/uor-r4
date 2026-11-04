# Modulo-256 generated-content signature

**Status:** FIT

## Question

Does preserving each generated Q12 query coordinate's low byte in a
2,048-bit modulo-256 code reduce signature ambiguity and extend the positive
content readout beyond the 256-bit sign signature?

This is D84 CPU development research on the Ryzen 9 9950X. It does not change
integer serving and is not a language or GPU-qualification claim.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `0ff468aae7cdc01d3e6b2f520a066534e1640da5` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Artifacts:

- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-{1,2}-raw_both.uorlut`
- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-{tokens.u16,response-mask.u8,manifest.json}`

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Split and mechanism

Use the same 16-key, 98-training/103-evaluation, training-document-disjoint
split and training-derived memory as deepseek-136.

Both modes derive rows only from actual generated training trajectories:

1. **sign**: one bit per 256 Q12 coordinate, set for non-negative values;
2. **code**: each coordinate's eight-bit residue modulo 256, packed at its
   coordinate position for 2,048 bits.

XOR plus popcount is the only lookup arithmetic. Each mode fits one maximum
distance covering at least 90% of leave-one-out rows. Decode chooses the
commonest target at minimum distance, breaks equal-count ties by smaller
token, and falls back to the exact writer beyond the threshold.

## Preflight

| Seed | Mode | Unique signatures | LOO top-1 | Ambiguous rows | Covered | Threshold | Changed shuffle assignments |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | sign | 45 | 199/894 | 892 | 891 | 0 | 45/45 |
| 1 | code | 619 | **300/894** | **277** | 807 | 537 | 618/619 |
| 2 | sign | 46 | 201/894 | 879 | 887 | 0 | 46/46 |
| 2 | code | 635 | **284/894** | **262** | 806 | 533 | 634/635 |

The byte code greatly expands distinct states and cuts ambiguous rows by
69-70%, while leave-one-out hits rise by 41-50%. Each mode covers at least
90% of its generated rows. The seed-1 selection report was byte-identical
between one-worker and 32-worker runs.

## Arms and bar

Run the exact writer, true and shuffled sign readouts, and true and shuffled
code readouts over complete 16-token generations.

Both seeds must pass:

- each true mode has at least 640 generated rows, 16 target values and 90%
  threshold coverage;
- each shuffled mode changes at least one effective commonest-target
  assignment keyed by unique signature;
- code held-out aligned top-1 beats writer and code-shuffled by the
  prompt-level rule `candidate_only >= max(2, ceil(3 * sqrt(control_only)))`
  and has a positive aggregate gain;
- code matching-prefix depth beats writer and code-shuffled by the same rule
  and has a positive aggregate gain;
- code top-1 and depth are not below sign on either seed, with at least one
  metric strictly above sign per seed;
- code leave-one-out top-1 exceeds sign on both seeds; and
- a seed-1 repeat matches every non-timing field and all four readout files.

Report exact references, stops, openings, nearest-distance and ambiguity
readings, readout/fallback decisions, distinct/repeated replies, workers,
wall and RSS. Exact replies are reported, not gated.

## Minimum test power

The split and two independent model seeds are fixed. The code-to-sign
comparison is paired at the prompt and uses the same signatures' generated
states. The sign arm controls for a generic effect of overriding writer
decisions.

## Instrument gates

- Source, inputs and binary hashes match.
- Each query contributes either 256 or 2,048 bits and lookup uses only XOR
  and popcount.
- Threshold fitting and lowest-token tie-breaking are deterministic.
- A shuffle must change the effective target map, not only row order.
- Evaluation is source-ordered and identical across one and 32 workers.
- The two modes share the exact training and evaluation sample selections.

## Stop rules

Stop on source/input/split drift, fewer than 640 rows, fewer than 16 targets,
less than 90% fitted coverage, a vacuous shuffle, worker nondeterminism,
resource breach or any need to change serving, integer-session, CUDA or
Claude-owned files.

## Decision consequence

A pass makes the modulo-256 code the preferred CPU content-readout signature
and a stronger serving-contract candidate. A fail closes this low-byte
packing and threshold configuration while preserving the sign result and any
other code packing or rotation for later comparison.

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
- Retained pipeline: 96 MiB maximum.
- Post-run free-space floor: 30 GiB plus 128 MiB.
- Previous ledger:
  `cpu-training-pilot-content-readout-2026-11-04/resource-ledger.json` at
  `35,291.977487699 s` measured and `38,527.677487699 s` admitted cumulative.
- Admitted cumulative projection: **40,327.677487699 s**.

## Steps

After FIT and a fresh GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-content-code
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/content-code/2026-11-04-attempt-1
BUILD=$SOURCE/target/deepseek/content-code-137
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=0ff468aae7cdc01d3e6b2f520a066534e1640da5
MIN_FREE_KIB=31588352
MAX_BUILD_BYTES=2147483648
MAX_PIPELINE_BYTES=100663296
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
  jq -e '.selection.training_samples == 98
    and .selection.evaluation_samples == 103
    and .bar.sign.structural_pass and .bar.code.structural_pass
    and .bar.code.exposure_pass' \
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
for file in sign-true-readout.json sign-shuffled-readout.json \
  code-true-readout.json code-shuffled-readout.json; do
  cmp "$PIPELINE/seed-1/$file" "$PIPELINE/seed-1-repeat/$file"
done
(cd "$PIPELINE" && find . -type f ! -name SHA256SUMS -print0 |
  sort -z | xargs -0 sha256sum >SHA256SUMS &&
  sha256sum -c SHA256SUMS --quiet)
test "$(du -sb "$PIPELINE" | cut -f1)" -le "$MAX_PIPELINE_BYTES"
```

Any source, input, binary, split, threshold, shuffle, worker-order,
deterministic repeat or resource mismatch is an instrument fault requiring a
new FIT.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 modulo-256 generated-content signature:
docs/harness/prep/cpu-training-pilot-content-code-2026-11-04.md
head: 0ff468aae7cdc01d3e6b2f520a066534e1640da5
implementation pin: 0ff468aae7cdc01d3e6b2f520a066534e1640da5
reviewer: independent-subagent content_code_fit_review
date: 2026-11-04

A FIT verdict clears compute for this configuration only (D22).
