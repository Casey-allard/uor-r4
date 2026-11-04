# Generated-state terminal content rows

**Status:** FIT

## Question

Can a modulo-256 content readout terminate held-out replies when BOS is
learned from actual generated states at the reference endpoint, without
losing the content gains of the ordinary code readout?

This is D84 CPU development research on the Ryzen 9 9950X. It does not change
integer serving and is not a language or GPU-qualification claim.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `70a0ae4d36eee03fc6a08f470fc9784782fe052d` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Split and mechanism

Use deepseek-137's 16-key, 98-training/103-evaluation split,
training-derived memory, 2,048-bit modulo-256 signatures, 90% Hamming radius,
commonest-target vote, smaller-token tie and exact writer fallback.

The ordinary code table stores only generated decisions inside the reference.
The augmented table adds target BOS for the generated decision whose offset
equals the reference length. Decisions after that offset are not added.

## Preflight

| Seed | Arm | Rows | LOO top-1 | Ambiguous | Covered | Threshold | Changed shuffle |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | code | 894 | 300 | 277 | 807 | 537 | 618/619 |
| 1 | code + endpoint | 978 | 325 | 340 | 881 | 529 | 663/664 |
| 2 | code | 894 | 284 | 262 | 806 | 533 | 634/635 |
| 2 | code + endpoint | 978 | 315 | 300 | 883 | 522 | 688/689 |

Each seed reaches 84 endpoint rows. The augmented table retains 90% coverage
and more correct leave-one-out targets than the ordinary code table.

The seed-1 selection output was byte-identical across one-worker and 32-worker
runs.

## Arms and bar

Run writer, code, code-shuffled, code plus endpoint rows and
code-stop-shuffled over complete 16-token generations.

Both seeds must pass:

- both code tables retain at least 640 rows, 16 targets and 90%
  leave-one-out coverage;
- both shuffled controls change their effective signature maps;
- code-stop top-1 and depth are not below ordinary code;
- code-stop BOS-complete replies exceed code and code-stop-shuffled by the
  prompt-level rule `candidate_only >= max(2, ceil(3 * sqrt(control_only)))`;
- at least **8 of 103** code-stop replies exactly match the reference and stop
  at BOS; the paired BOS rule makes **9** the effective minimum when the
  control emits none; and
- at least **8 of 103** code-stop replies stop at BOS without exceeding their
  reference length.

Report exact, BOS-complete, premature/shorter, cap and context stops,
top-1/depth, openings, readout/fallback decisions, distinct/repeated replies,
workers, wall and RSS.

## Minimum test power

The split and two model seeds are fixed. The ordinary code arm is the matched
content control; the shuffled endpoint table controls target assignment. A
minimum of eight exact/BOS outcomes prevents a single accidental stop from
supporting the mechanism.

## Instrument gates

- Source and five public input hashes match.
- Endpoint rows exist only at the generated offset equal to normalized
  reference length.
- No post-reference row is added.
- Threshold selection, target voting and stop execution are deterministic.
- Both effective-map shuffled controls are non-vacuous.
- Evaluation is source-ordered and identical across one and 32 workers.

## Stop rules

Stop on source/input/split drift, failure to retain 640 rows, 16 targets or 90%
coverage, missing endpoint rows, a vacuous shuffle, worker nondeterminism,
resource breach or any need to change serving, integer-session, CUDA or
Claude-owned files.

## Decision consequence

A pass makes generated-state endpoint rows a candidate termination readout
for a serving-contract unit. A fail closes this exact endpoint-row
augmentation and directs termination work toward a separate two-class
generated-state gate. It does not invalidate the positive content code.

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
- Retained pipeline: 128 MiB maximum.
- Previous ledger:
  `cpu-training-pilot-content-code-2026-11-04/resource-ledger.json` at
  `35,312.777487699 s` measured and `40,327.677487699 s` admitted cumulative.
- Admitted cumulative projection: **42,127.677487699 s**.

## Steps

After FIT and a fresh GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-content-stop
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/content-stop/2026-11-04-attempt-1
BUILD=$SOURCE/target/deepseek/content-stop-138
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=70a0ae4d36eee03fc6a08f470fc9784782fe052d
MIN_FREE_KIB=31588352
MAX_BUILD_BYTES=2147483648
free_kib() { df -Pk "$1" | awk 'NR == 2 { print $4 }'; }
test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
test -z "$(git -C "$SOURCE" status --porcelain -- crates)"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- crates/uor-r4-cpu-train
test ! -e "$PIPELINE"
test ! -e "$BUILD"
mkdir -p "$PIPELINE"
BUILD_STARTED=$(date -u +%s)
/usr/bin/time -v -o "$PIPELINE/build.time.txt" env CARGO_BUILD_JOBS=16 \
  CARGO_TARGET_DIR="$BUILD" cargo build --locked --release \
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
  /usr/bin/time -v -o "$PIPELINE/$label.time.txt" timeout 600s \
    env RAYON_NUM_THREADS=32 "$BIN" \
    "$DUAL/dual-c131-seed-${seed}-raw_both.uorlut" \
    "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" "$DEV/dev-manifest.json" \
    "$out" --expected-artifact-sha256 "$artifact" \
    --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
    --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
    --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
    --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
    --reply-cap 16 --seed 1 >"$PIPELINE/$label.stdout.json"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 1048576
  jq -e '.bar.code.structural_pass and .bar.code_stop.structural_pass' \
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
for file in "$PIPELINE"/seed-1/*-readout.json; do
  cmp "$file" "$PIPELINE/seed-1-repeat/$(basename "$file")"
done
(cd "$PIPELINE" && find . -type f ! -name SHA256SUMS -print0 |
  sort -z | xargs -0 sha256sum >SHA256SUMS &&
  sha256sum -c SHA256SUMS --quiet)
test "$(du -sb "$PIPELINE" | cut -f1)" -le 134217728
```

Any source, input, binary, endpoint selection, stop, shuffle, worker-order,
repeat or resource mismatch is an instrument fault requiring a new FIT.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 generated-state terminal content rows:
docs/harness/prep/cpu-training-pilot-content-stop-2026-11-04.md
head: 70a0ae4d36eee03fc6a08f470fc9784782fe052d
implementation pin: 70a0ae4d36eee03fc6a08f470fc9784782fe052d
reviewer: independent-subagent content_stop_finalfit
date: 2026-11-04

A FIT verdict clears compute for this configuration only (D22).
