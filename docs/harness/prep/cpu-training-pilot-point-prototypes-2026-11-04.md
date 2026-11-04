# Target-local point prototypes on unseen opening keys

**Status:** FIT

## Question

Can a target-balanced two-point local prototype table, learned from
training-only point rows, improve unseen-key opening transfer over point-off
and a matched shuffled-target prototype table on both public model seeds while
preserving exact CPU and integer-serving decisions through the complete
Hamming, turn-memory and point path?

This is D84 CPU development research on the Ryzen 9 9950X. It is not GPU
qualification or language capability.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifact

| Input | Identity |
|---|---|
| implementation source | `34103fab4bcbfac97adfcc8c3fbecaa8318ba62f` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Artifacts:

- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-{1,2}-raw_both.uorlut`
- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-{tokens.u16,response-mask.u8,manifest.json}`

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Fixed split

Replay deepseek-104/123/106 exactly: even document ordinals train, odd document
ordinals evaluate, eight training keys and eight disjoint evaluation keys, up
to eight examples per key, 60 training and 60 evaluation examples. The
selection code and dimensions are unchanged. Opening/reply memory is derived
only from training examples at key window 192 and reply turn 8. Every
evaluation opening key must miss the exact memory table.

The public manifest says `split=train`. "Unseen-key" therefore means the exact
opening keys were absent from the learned memory and point tables; base-model
exposure is not claimed to be zero.

## Training procedure

Build and coalesce the existing production raw point rows from the 60 training
prompts. For each distinct supervised first-reply token:

1. choose its observed medoid by exact within-target L1 sum;
2. among the remaining distinct observed points, choose the point with maximum
   minimum L1 distance to the selected set;
3. break every distance tie by component-wise signed lexicographic point
   order, then target id; and
4. retain at most two rows for that target.

The fixed treatment is `k = 2`. It is chosen before observing any
prototype-arm result because the prior one-medoid table erased the useful local
neighborhood. `k = 2` preserves one central and one residual local observation
per target without restoring every raw row.

The matched shuffled-target prototype control preserves the treatment point
multiset and target counts while permuting target assignments deterministically
at seed 1. If the first permutation is unchanged, rotate once; refuse an
unchanged or all-equal control.

## Arms

Every arm carries the same derived memory and evaluates the same 60 held-out
samples:

1. point-off;
2. raw-row nearest;
3. shuffled-target raw-row nearest;
4. raw-row vote at `k = 8`;
5. one target medoid;
6. shuffled-target medoid;
7. two target-local prototypes; and
8. shuffled-target two-prototype control.

Arms 2-6 are descriptive controls from the two preceding units. Arms 1, 7 and
8 are the registered comparison.

## Bar

For each seed, pair the treatment and each primary control by evaluation row:

- prototype opening top-1 must exceed point-off;
- prototype opening top-1 must exceed the shuffled-target prototypes;
- each paired gain must be at least
  `max(2, ceil(2 * sqrt(b + c)))`, where `b` is candidate-only correct and `c`
  is control-only correct; and
- prototype complete teacher-forced top-1 must not be below either primary
  control.

Both seeds must pass. The result is negative if any condition fails on either
seed. Do not tune `k`, keys, rows, targets, random seed or scoring after
observing results.

## Instrument gates

- Input hashes, fixed key selection, zero key overlap and every held-out
  exact-memory miss check.
- The prototype table has at least two targets, contains only observed points,
  has at most two rows per target and is strictly smaller than the raw table.
- The treatment is deterministic across two builds.
- The shuffled control preserves the point multiset and target-value multiset,
  changes the target vector, and changes at least one treatment opening.
- Point fallback changes all 60 treatment openings relative to point-off.
- The shuffled-target prototype control changes at least one treatment opening.
- Opening target distance is identical row by row for every arm.
- Every arm reports zero query-coordinate, token and kept-cell mismatches
  against `IntegerStackModel` over 2,110 positions and 540,160 Q12
  coordinates per seed.
- A seed-1 repeat reproduces every non-timing and non-resource report field.

## Steps

This compute runs only after FIT. It builds twice in separate fresh release
targets, binds each binary hash, and executes seed 1 and seed 2 with build A,
then a seed-1 repeat with build B, into fresh output roots under
`local/deepseek/point-prototypes/2026-11-04-attempt-1/`:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-point-prototypes
SHARED=/home/callard/projects/codex-playground/uor-r4
SOURCE_PIN=34103fab4bcbfac97adfcc8c3fbecaa8318ba62f
PIPELINE=$SHARED/local/deepseek/point-prototypes/2026-11-04-attempt-1
BUILD_A=$SOURCE/target/deepseek/point-prototypes-build-130-a
BUILD_B=$SOURCE/target/deepseek/point-prototypes-build-130-b
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
HARNESS=$SHARED/local/harness-target/release/uor-harness
MIN_BUILD_FREE_KIB=37748736
MIN_RESERVE_KIB=31588352
MAX_BUILD_BYTES=5368709120
MAX_BUILD_RSS_KIB=4194304
MAX_RETAINED_BYTES=67108864
SOURCE_GUARD_PATHS=(crates third_party Cargo.toml Cargo.lock .cargo rust-toolchain.toml)
free_kib() {
  df -Pk "$1" | awk 'NR == 2 { print $4 }'
}
test "$(free_kib "$SOURCE")" -ge "$MIN_BUILD_FREE_KIB"
test "$(free_kib "$SHARED")" -ge "$MIN_BUILD_FREE_KIB"
"$HARNESS" budget show --json | jq -e \
  '.model_time_ledger.present == false
   and .resource_ledger.chain.head == "cpu-training-pilot-point-prototypes-2026-11-04/resource-ledger.json"
   and .resource_ledger.chain.cumulative_projected_seconds == 35953' >/dev/null
test "$(env -u RUSTC -u RUSTUP_TOOLCHAIN rustc --version)" = \
  "rustc 1.97.1 (8bab26f4f 2026-07-14)"
test "$(env -u RUSTC -u RUSTUP_TOOLCHAIN cargo --version)" = \
  "cargo 1.97.1 (c980f4866 2026-06-30)"
test -z "$(git -C "$SOURCE" status --porcelain -- "${SOURCE_GUARD_PATHS[@]}")"
git -C "$SOURCE" diff --quiet "$SOURCE_PIN" -- "${SOURCE_GUARD_PATHS[@]}"
test ! -e "$PIPELINE"
test ! -e "$BUILD_A"
test ! -e "$BUILD_B"
mkdir -p "$PIPELINE/builds"
BUILD_STARTED=$(date -u +%s)
(
  cd "$SOURCE"
  /usr/bin/time -v -o "$PIPELINE/builds/build-a.time.txt" \
    env -u RUSTC -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER \
      -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u RUSTDOCFLAGS \
      -u RUSTUP_TOOLCHAIN CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD_A" \
      cargo build --locked --release \
        -p uor-r4-cpu-train --bin cpu-point-transfer-driver
)
(
  cd "$SOURCE"
  /usr/bin/time -v -o "$PIPELINE/builds/build-b.time.txt" \
    env -u RUSTC -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER \
      -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u RUSTDOCFLAGS \
      -u RUSTUP_TOOLCHAIN CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD_B" \
      cargo build --locked --release \
        -p uor-r4-cpu-train --bin cpu-point-transfer-driver
)
BUILD_SECONDS=$(( $(date -u +%s) - BUILD_STARTED ))
BUILD_BYTES=$(( $(du -sb "$BUILD_A" | cut -f1) + $(du -sb "$BUILD_B" | cut -f1) ))
BUILD_PEAK_RSS_KIB=$(awk -F': ' '/Maximum resident set size/ { print $2 }' \
  "$PIPELINE/builds/build-a.time.txt" "$PIPELINE/builds/build-b.time.txt" |
  sort -nr | head -n 1)
test "$BUILD_BYTES" -le "$MAX_BUILD_BYTES"
test "$BUILD_PEAK_RSS_KIB" -le "$MAX_BUILD_RSS_KIB"
printf '{"jobs":16,"wall_seconds":%s,"peak_rss_kib":%s,"target_bytes":%s,"max_peak_rss_kib":%s}\n' \
  "$BUILD_SECONDS" "$BUILD_PEAK_RSS_KIB" "$BUILD_BYTES" "$MAX_BUILD_RSS_KIB" \
  >"$PIPELINE/builds/summary.json"
(
  cd "$PIPELINE/builds"
  find . -type f ! -name SHA256SUMS -print0 |
    sort -z | xargs -0 sha256sum >SHA256SUMS
  sha256sum -c SHA256SUMS --quiet
)
test "$(free_kib "$SOURCE")" -ge "$MIN_RESERVE_KIB"
test "$(free_kib "$SHARED")" -ge "$MIN_RESERVE_KIB"
BIN_A="$BUILD_A/release/cpu-point-transfer-driver"
BIN_B="$BUILD_B/release/cpu-point-transfer-driver"
BIN_A_SHA=$(sha256sum "$BIN_A" | cut -d' ' -f1)
BIN_B_SHA=$(sha256sum "$BIN_B" | cut -d' ' -f1)
run() {
  bin=$1 bin_sha=$2 seed=$3 artifact_pin=$4 out=$5
  test ! -e "$out"
  /usr/bin/time -v timeout 600s env RAYON_NUM_THREADS=32 \
    "$bin" "$DUAL/dual-c131-seed-${seed}-raw_both.uorlut" \
    "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" \
    "$DEV/dev-manifest.json" "$out" \
    --expected-artifact-sha256 "$artifact_pin" \
    --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
    --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
    --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
    --train-keys 8 --evaluation-keys 8 --per-key 8 \
    --key-tokens 8 --reply-turn 8 --neighbor-vote 8 \
    --target-prototypes 2 --random-seed 1
  test "$(sha256sum "$bin" | cut -d' ' -f1)" = "$bin_sha"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 1048576
  test "$(jq -er '.target_prototypes.rows' "$out/report.json")" \
    -lt "$(jq -er '.point.rows' "$out/report.json")"
  test "$(jq -er '.point_fallback_exposure.point_prototypes' "$out/report.json")" -eq 60
  printf '%s  cpu-point-transfer-driver\n' "$bin_sha" >"$out/binary.sha256"
  (
    cd "$out"
    find . -type f ! -name SHA256SUMS -print0 |
      sort -z | xargs -0 sha256sum >SHA256SUMS
    sha256sum -c SHA256SUMS --quiet
  )
}
run "$BIN_A" "$BIN_A_SHA" 1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$PIPELINE/seed-1"
run "$BIN_A" "$BIN_A_SHA" 2 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$PIPELINE/seed-2"
run "$BIN_B" "$BIN_B_SHA" 1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$PIPELINE/seed-1-repeat"
diff <(jq -S 'del(.wall_elapsed_ms, .peak_rss_kib)' "$PIPELINE/seed-1/report.json") \
  <(jq -S 'del(.wall_elapsed_ms, .peak_rss_kib)' "$PIPELINE/seed-1-repeat/report.json")
test "$(jq -er '.target_prototypes.artifact_sha256' "$PIPELINE/seed-1/report.json")" \
  = "$(jq -er '.target_prototypes.artifact_sha256' "$PIPELINE/seed-1-repeat/report.json")"
test "$(jq -er '.target_prototypes.random_artifact_sha256' "$PIPELINE/seed-1/report.json")" \
  = "$(jq -er '.target_prototypes.random_artifact_sha256' "$PIPELINE/seed-1-repeat/report.json")"
test -z "$(git -C "$SOURCE" status --porcelain -- "${SOURCE_GUARD_PATHS[@]}")"
test "$(free_kib "$SOURCE")" -ge "$MIN_RESERVE_KIB"
test "$(free_kib "$SHARED")" -ge "$MIN_RESERVE_KIB"
test "$(du -sb "$PIPELINE" | cut -f1)" -le "$MAX_RETAINED_BYTES"
```

Any source, binary, argument, output-root, control or deterministic mismatch
is an instrument fault requiring a new FIT before retry.

At completion, update
`docs/labs/cpu-training-pilot-point-prototypes-2026-11-04/resource-ledger.json`
from `PROJECTED_NOT_RUN` to `COMPLETE`, replacing projected seconds and RSS
with actual values from the reports and build receipt. Preserve
`previous_ledger` and update the project cumulative as the preceding ledger's
34,633 seconds plus this run's actual increment. The current active chain has
no earlier DeepSeek entry, so set the lab cumulative to that actual increment.
The named shared model-time ledger is absent and is not treated as zero.

## Minimum test power

Two model seeds over 60 fixed held-out openings; both must pass the paired bar.
The complete teacher-forced check covers 2,050 decisions per seed and guards
against an opening-only gain that damages continuation. This is bounded
exploratory evidence, not a capability bar.

## Cost and stop

Local workstation only; no GPU, pod or paid API. Rust 1.97.1 on Linux
7.0.0-42-generic x86_64. At the repaired review, the project filesystem had
513,053,864 KiB (489.287 GiB) free. The named shared model-time ledger is
absent. This unit chains its projected 1,320 seconds to the existing current
repository ledger: 34,633 + 1,320 = 35,953 project seconds. Its 1,320-second
lab cumulative is based on the absence of an earlier DeepSeek entry in the
active chain, not on pretending the global model-time ledger exists.

| Stage | Projection |
|---|---|
| preparation and FIT | local inspection only; no model compute |
| builds | two sequential fresh release targets, 16 Cargo jobs each, under 5 GiB combined, peak RSS below 4 GiB per build and under five minutes total |
| training | none |
| controls and evaluation | seed 1, seed 2 and a seed-1 cross-build repeat, 32 Rayon workers, under two minutes total, peak RSS below 1 GiB |
| checkpointing | none; there is no training run |
| retry | none; an instrument fault requires a new FIT |
| retained output | three sealed roots below 64 MiB total |
| rebuildable output | the two fresh target directories remain available for audit and hygiene rather than being deleted |

The recipe requires 36 GiB free before building and at least the project's
30 GiB plus 128 MiB floor after builds and after the runs, on both the source
and shared output filesystems. Three roots must stay below 64 MiB. Stop on
input drift, nondeterminism, a malformed or unchanged control, memory-key
overlap, CPU/serving divergence, a non-reducing prototype table, a
storage-floor breach or evaluation-time tuning.

## Claim boundary

A positive supports only the fixed two-prototype construction on the fixed
unseen-key panel and two public seeds. A negative closes only this
deterministic target-balanced `k = 2` construction. Neither changes GPU
qualification or establishes complete replies, fresh phrasing or useful chat.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 target-local point prototypes
sheet: `docs/harness/prep/cpu-training-pilot-point-prototypes-2026-11-04.md`
head: `2d30dad9c0011ea0b9bd38d38793e066323e3c57`
implementation pin: `34103fab4bcbfac97adfcc8c3fbecaa8318ba62f`
reviewer: Codex fresh-context non-author
note: two-seed D84 screen; training-only target-local medoid plus farthest
residual, fixed unseen-key split, point-off and matched shuffled-target
controls, both-build deterministic repeat, all-60 treatment exposure, row-wise
distance equality, complete CPU/serving parity, fail-closed RSS measurement,
two-seed paired bar, 30 GiB plus 128 MiB floor, and cumulative resource-ledger
chain
date: 2026-11-04

A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone
before compute.
