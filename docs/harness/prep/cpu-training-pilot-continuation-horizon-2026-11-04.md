# Generated-state exact-offset continuation horizon

**Status:** STOPPED AT PREFLIGHT

## Question

Can generated modulo-256 content states predict their ordinal remaining
reference horizon well enough to stop at the exact reply endpoint, where the
tested prototype and class-density gates did not?

This is D84 CPU development research on the Ryzen 9 9950X. It does not change
integer serving, `UORLUT01`, CUDA, float training or Claude-owned source, and
it makes no language or GPU-qualification claim.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `0a8631d4e033af39025b06c64fc50496782269f4` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Data split

Use the deepseek-137 16-key split: 98 training and 103 evaluation samples per
artifact, runner seed 1. Both fitted artifacts use the same prompt data but
different trained model artifacts.

## Minimum test power

The comparison is deterministic over complete generated trajectories. Two
trained artifacts and a separate one-worker seed-1 repeat prevent claims from
one seed or one worker order. The result remains local to the named public
development inputs.

## Fixed construction

Generate each training reply with the positive modulo-256 content arm. For
every generated decision record `remaining = normalized_reference_length -
generated_offset` as a signed integer, including negative values after the
reference.

- Candidate k is 1, 2, 4 or 8.
- Neighbours have exactly the same generated offset.
- Rank by Hamming distance, then complete signature bytes and horizon.
- Exclude the query row during fitting.
- Predict the lower median horizon of the k nearest rows.
- Emit BOS only when the prediction is exactly zero. Negative predictions
  continue with the unchanged content token.
- Select k by maximum leave-one-out terminal recall, then fewer false stops,
  lower horizon absolute error, then smaller k.
- The control rotates horizons within each exact-offset group, preserving the
  group size, offsets and complete horizon multiset while changing their
  signature association.

No class-balanced distance, terminal/nonterminal prototype, missing-neighbour
penalty or post-observation retuning is part of this arm.

## Preflight

Run `--select-only` for seeds 1 and 2 with
`--stop-horizon-neighbors 8`, without old stop-gate flags.

Proceed only if, for each seed:

- the horizon arm retains exactly 1,568 rows: 84 zero-horizon and 1,484
  nonzero-horizon rows;
- true LOO terminal recall is at least 26/84;
- true false stops are at most the 1% nonterminal limit;
- true horizon absolute error is strictly lower than the within-offset
  horizon-permuted control;
- true and shuffled gate hashes differ and predictions change on at least one
  row;
- the selected k is in `{1,2,4,8}` and gate rows equal all training rows; and
- source and all public inputs match.

The preflight output, stderr, resource reading and checksum survive a stop.

## Arms and bar

Run no-gate code, true horizon and within-offset horizon-permuted arms over
complete 16-token replies. Both seeds must pass:

- true training recall is at least **26/84**, with false stops at or below 1%;
- true horizon absolute error is below shuffled;
- at least **9/103** replies stop at BOS, at least **8** stop at or before the
  normalized reference length, and at least **8** exactly match the reference
  and stop at BOS;
- true BOS completion exceeds code and shuffled by
  `candidate_only >= max(2, ceil(3 * sqrt(control_only)))`;
- true top-1 and matching-prefix depth are not below code;
- every fired decision is BOS and every ungated token equals code;
- true zero-horizon predictions beat shuffled by
  `candidate_only >= max(2, ceil(2 * sqrt(control_only)))`;
- gate hashes and per-row stop predictions differ; and
- the seed-1 one-worker repeat matches every non-timing report field and
  complete generated JSON artifact from the 32-worker run.

Report all k candidates' recall, false stops and horizon error; selected k;
true and shuffled full horizon error; BOS class counts; top-1/depth; openings;
distinct/repeated replies; workers; wall, RSS and hashes.

## Instrument gates

- Source and all five public inputs match.
- Horizon derivation is tested against normalized reference length, including
  a negative post-reference horizon.
- Exact-offset neighbour selection, deterministic ties, lower median,
  leave-one-out fitting and malformed-width refusal are tested.
- The within-offset permutation preserves each group's horizon multiset and
  changes the effective mapping.
- Fired decisions are BOS and ungated decisions equal code.
- One-worker and 32-worker generated evidence matches.
- Full CPU release tests, scoped Clippy, rustfmt and evidence manifests pass.

## Stop rules

Stop on source/input/split drift, missing horizons, a preflight bar failure,
vacuous permutation, ungated token mismatch, worker nondeterminism, resource
breach, or any need to change integer serving, CUDA, float training or
Claude-owned source.

## Decision consequence

A pass makes exact-offset continuation-horizon prediction a supported CPU
termination candidate and justifies artifact-bound serving integration. A
fail closes this lower-median same-offset configuration and requires a
different trajectory state, decode objective or content mechanism. Neither
result alone establishes useful chat.

## Outcome

Attempt 1 executed implementation pin
`0a8631d4e033af39025b06c64fc50496782269f4`. The release driver, 254 library
tests, eight driver tests, scoped Clippy and targeted rustfmt passed. The five
public inputs and source pin matched.

| Seed 1 reading | True horizon | Permuted horizon | Registered limit |
|---|---:|---:|---:|
| Total / terminal / nonterminal rows | 1,568 / 84 / 1,484 | 1,568 / 84 / 1,484 | exact |
| Terminal recall | **40/84** | 33/84 | at least 26/84 |
| Nonterminal false stops | **66/1,484** | 62/1,484 | at most 14/1,484 |
| Horizon absolute error | **7,184** | 9,525 | true < shuffled |
| Selected k | 2 | 1 | in `{1,2,4,8}` |
| Changed raw predictions | 806 | 806 | > 0 |

True horizon error was lower than the permuted control's and raw predictions
changed, establishing a non-vacuous control; terminal recall passed. The
false-stop count failed, so the registered stop rule prevented the seed-2
preflight, all generation arms and the one-worker repeat. The limit and
configuration were not changed after observation.

Exact stopped evidence is under
[`evidence-attempt-1/`](../../labs/cpu-training-pilot-continuation-horizon-2026-11-04/evidence-attempt-1/).
Measured compute and cumulative accounting are in the
[resource ledger](../../labs/cpu-training-pilot-continuation-horizon-2026-11-04/resource-ledger.json).

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT and pre-run review | 600 s |
| release build, controls and two-seed preflight | 1,500 s |
| two seeds plus repeat | 900 s |
| delivery and evidence | 300 s |
| total increment | **3,300 s** |

- Build and controls: 16 jobs, 4 GiB RSS, separate 2 GiB and 3 GiB target caps.
- Runs: 32 workers for seeds 1/2, one worker for the repeat, 1 GiB RSS and
  600 s per run.
- Retained evidence: 201,326,592 bytes maximum.
- Previous ledger:
  `cpu-training-pilot-stop-density-2026-11-04/resource-ledger.json` at
  `35,744.068816343 s` measured and `55,327.677487699 s` admitted cumulative.
- Admitted cumulative projection: **58,627.677487699 s**.

## Steps

After FIT and a fresh GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-continuation-horizon
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/continuation-horizon/2026-11-04
BUILD=$SOURCE/target/deepseek/continuation-horizon-143
CONTROL_BUILD=$SOURCE/target/deepseek/continuation-horizon-controls-143
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=0a8631d4e033af39025b06c64fc50496782269f4
MIN_FREE_KIB=31588352
RUNTIME_PATHS=(.cargo Cargo.toml Cargo.lock rust-toolchain.toml crates third_party)
free_kib() { df -Pk "$1" | awk 'NR == 2 { print $4 }'; }
seal_pipeline() {
  test -d "$PIPELINE" || return 0
  (
    cd "$PIPELINE"
    find . -type f ! -name SHA256SUMS ! -name SHA256SUMS.tmp -print0 |
      sort -z | xargs -0 -r sha256sum >SHA256SUMS.tmp
    mv SHA256SUMS.tmp SHA256SUMS
    sha256sum -c SHA256SUMS --quiet
  )
}
test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
test "$(nproc)" -eq 32
test "$(lscpu | awk -F: '/Model name/ { sub(/^[ \t]+/, "", $2); print $2; exit }')" = \
  "AMD Ryzen 9 9950X 16-Core Processor"
test "$(env -u RUSTC -u RUSTUP_TOOLCHAIN rustc --version)" = \
  "rustc 1.97.1 (8bab26f4f 2026-07-14)"
test "$(env -u RUSTUP_TOOLCHAIN cargo --version)" = \
  "cargo 1.97.1 (c980f4866 2026-06-30)"
test "$(awk '/MemAvailable/ { print $2 }' /proc/meminfo)" -ge 8388608
test -z "$(git -C "$SOURCE" status --porcelain -- "${RUNTIME_PATHS[@]}")"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- "${RUNTIME_PATHS[@]}"
test ! -e "$PIPELINE"
test ! -e "$BUILD"
test ! -e "$CONTROL_BUILD"
mkdir -p "$PIPELINE"
trap 'status=$?; trap - EXIT; seal_pipeline || status=$?; exit "$status"' EXIT
BUILD_STARTED=$(date -u +%s)
(
  cd "$SOURCE"
  /usr/bin/time -v -o "$PIPELINE/build.time.txt" \
    env -u RUSTC -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER \
    -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u RUSTDOCFLAGS \
    -u RUSTUP_TOOLCHAIN CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" \
    cargo build --locked --release \
    -p uor-r4-cpu-train --bin cpu-content-readout-driver
)
BUILD_SECONDS=$(( $(date -u +%s) - BUILD_STARTED ))
BUILD_BYTES=$(du -sb "$BUILD" | cut -f1)
BUILD_RSS=$(awk -F': ' '/Maximum resident set size/ { print $2 }' \
  "$PIPELINE/build.time.txt")
test "$BUILD_BYTES" -le 2147483648
test "$BUILD_RSS" -le 4194304
printf '{"jobs":16,"wall_seconds":%s,"peak_rss_kib":%s,"target_bytes":%s}\n' \
  "$BUILD_SECONDS" "$BUILD_RSS" "$BUILD_BYTES" >"$PIPELINE/build-summary.json"
BIN=$BUILD/release/cpu-content-readout-driver
sha256sum "$BIN" | sed 's# .*/#  #' >"$PIPELINE/binary.sha256"
(
  cd "$SOURCE"
  /usr/bin/time -v -o "$PIPELINE/control-lib.time.txt" \
    env -u RUSTC -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER \
    -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u RUSTDOCFLAGS \
    -u RUSTUP_TOOLCHAIN CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$CONTROL_BUILD" \
    timeout 600s cargo test --locked --release \
    -p uor-r4-cpu-train --lib >"$PIPELINE/control-lib.txt"
  /usr/bin/time -v -o "$PIPELINE/control-bin.time.txt" \
    env -u RUSTC -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER \
    -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u RUSTDOCFLAGS \
    -u RUSTUP_TOOLCHAIN CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$CONTROL_BUILD" \
    timeout 600s cargo test --locked --release \
    -p uor-r4-cpu-train --bin cpu-content-readout-driver \
    >"$PIPELINE/control-bin.txt"
  /usr/bin/time -v -o "$PIPELINE/control-clippy.time.txt" \
    env -u RUSTC -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER \
    -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u RUSTDOCFLAGS \
    -u RUSTUP_TOOLCHAIN CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$CONTROL_BUILD" \
    timeout 600s cargo clippy --locked \
    -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings \
    >"$PIPELINE/control-clippy.txt" 2>&1
)
CONTROL_BYTES=$(du -sb "$CONTROL_BUILD" | cut -f1)
CONTROL_RSS=$(awk -F': ' '/Maximum resident set size/ {
  if ($2 > max) max = $2
} END { print max }' "$PIPELINE"/control-*.time.txt)
test "$CONTROL_BYTES" -le 3221225472
test "$CONTROL_RSS" -le 4194304
env -u RUSTUP_TOOLCHAIN cargo fmt -p uor-r4-cpu-train -- --check
git -C "$SOURCE" diff --check
printf '{"jobs":16,"target_bytes":%s,"peak_rss_kib":%s}\n' \
  "$CONTROL_BYTES" "$CONTROL_RSS" >"$PIPELINE/control-summary.json"
run_preflight() {
  seed=$1 artifact=$2
  status=0
  /usr/bin/time -v -o "$PIPELINE/seed-$seed.preflight.time.txt" \
    timeout 600s env RAYON_NUM_THREADS=32 "$BIN" \
      "$DUAL/dual-c131-seed-${seed}-raw_both.uorlut" \
      "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" "$DEV/dev-manifest.json" \
      "$PIPELINE/select-only-unused-$seed" \
      --expected-artifact-sha256 "$artifact" \
      --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
      --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
      --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
      --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
      --reply-cap 16 --stop-horizon-neighbors 8 --seed 1 --select-only \
      >"$PIPELINE/seed-$seed.preflight.json" \
      2>"$PIPELINE/seed-$seed.preflight.stderr.txt" || status=$?
  if test "$status" -ne 0; then
    jq -n --arg seed "$seed" --argjson exit "$status" \
      --arg stderr "$(cat "$PIPELINE/seed-$seed.preflight.stderr.txt")" \
      '{schema:"uor-r4.cpu-content-continuation-preflight-failure/1",
        seed:$seed, exit:$exit, stderr:$stderr}' \
      >"$PIPELINE/seed-$seed.preflight-failure.json"
    return "$status"
  fi
  PREFLIGHT_RSS=$(awk -F': ' '/Maximum resident set size/ { print $2 }' \
    "$PIPELINE/seed-$seed.preflight.time.txt")
  test "$PREFLIGHT_RSS" -le 1048576
  if ! jq -e '.modes.gate_horizon as $h |
    $h.training as $g |
    $g.rows == 1568 and
    $g.terminal_rows == 84 and
    $g.nonterminal_rows == 1484 and
    $g.terminal_stops >= 26 and
    $g.nonterminal_stops <= $g.nonterminal_stop_limit and
    $g.horizon_abs_error < $h.shuffled_training.horizon_abs_error and
    ([1,2,4,8] | index($g.gate.neighbors)) != null and
    ($g.gate.rows | length) == $g.rows and
    $h.true_sha256 != $h.shuffled_sha256 and
    $h.predictions_changed > 0' \
    "$PIPELINE/seed-$seed.preflight.json" >/dev/null; then
    jq -n --arg seed "$seed" \
      --slurpfile preflight "$PIPELINE/seed-$seed.preflight.json" \
      '{schema:"uor-r4.cpu-content-continuation-preflight-bar-failure/1",
        seed:$seed, report:$preflight[0]}' \
      >"$PIPELINE/seed-$seed.preflight-bar-failure.json"
    return 1
  fi
}
run_preflight 1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run_preflight 2 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
run() {
  seed=$1 label=$2 artifact=$3 workers=$4
  out=$PIPELINE/$label
  /usr/bin/time -v -o "$PIPELINE/$label.time.txt" timeout 600s \
    env RAYON_NUM_THREADS="$workers" "$BIN" \
    "$DUAL/dual-c131-seed-${seed}-raw_both.uorlut" \
    "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" "$DEV/dev-manifest.json" \
    "$out" --expected-artifact-sha256 "$artifact" \
    --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
    --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
    --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
    --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
    --reply-cap 16 --stop-horizon-neighbors 8 --seed 1 \
    >"$PIPELINE/$label.stdout.json"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 1048576
  jq -e '.bar.code.structural_pass and (.bar.gate_horizon != null)' \
    "$out/report.json" >/dev/null
  test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
  test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
}
run 1 seed-1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd 32
run 2 seed-2 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 32
run 1 seed-1-repeat a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd 1
SEED_1_PASS=$(jq -r '.bar.gate_horizon.passes == true' \
  "$PIPELINE/seed-1/report.json")
SEED_2_PASS=$(jq -r '.bar.gate_horizon.passes == true' \
  "$PIPELINE/seed-2/report.json")
BAR_PASS=false
if test "$SEED_1_PASS" = true && test "$SEED_2_PASS" = true; then
  BAR_PASS=true
fi
jq 'del(.wall_elapsed_ms, .peak_rss_kib, .rayon_workers)' \
  "$PIPELINE/seed-1/report.json" >"$PIPELINE/seed-1.normalized.json"
jq 'del(.wall_elapsed_ms, .peak_rss_kib, .rayon_workers)' \
  "$PIPELINE/seed-1-repeat/report.json" >"$PIPELINE/seed-1-repeat.normalized.json"
REPEAT_PASS=true
cmp "$PIPELINE/seed-1.normalized.json" "$PIPELINE/seed-1-repeat.normalized.json" \
  || REPEAT_PASS=false
while IFS= read -r -d '' file; do
  cmp "$file" "$PIPELINE/seed-1-repeat/${file#"$PIPELINE/seed-1/"}" \
    || REPEAT_PASS=false
done < <(find "$PIPELINE/seed-1" -maxdepth 1 -type f -name '*.json' \
  ! -name report.json -print0 | sort -z)
printf '{"seed_1_passes":%s,"seed_2_passes":%s,"bar_passes":%s,"repeat_passes":%s}\n' \
  "$SEED_1_PASS" "$SEED_2_PASS" "$BAR_PASS" "$REPEAT_PASS" \
  >"$PIPELINE/bar-summary.json"
seal_pipeline
trap - EXIT
test "$(du -sb "$PIPELINE" | cut -f1)" -le 201326592
test -z "$(git -C "$SOURCE" status --porcelain -- "${RUNTIME_PATHS[@]}")"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- "${RUNTIME_PATHS[@]}"
test "$BAR_PASS" = true
test "$REPEAT_PASS" = true
```

Any source, input, split, horizon, prediction, token identity, worker-order,
repeat or resource mismatch is an instrument fault requiring a new FIT.

## Questions for review

- Is exact-offset matching the right continuation state, rather than a radius
  that could blend a terminal state with nearby nonterminal states?
- Does lower-median prediction express the objective without silently making a
  terminal-majority classifier?
- Is stopping only on predicted zero, while continuing on negative predictions,
  correctly separated from the earlier binary class gates?
- Does the within-offset horizon rotation provide a non-vacuous control at
  every tested offset without changing horizon counts?

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 deepseek-143
sheet: docs/harness/prep/cpu-training-pilot-continuation-horizon-2026-11-04.md
reviewer: codex-fresh-context-fit-attempt-4
note: Exact 1,568/84/1,484 counts, failure sealing, raw Option<i16>
  prediction changes and explicit non-self LOO terminal pairing verified.
date: 2026-11-04

A FIT verdict clears compute for the configuration in that sheet only (D22).

## Pre-run GO

GO - reviewer `codex-fresh-context-go-attempt-1`, 2026-11-04. HEAD was
clean at `d44b905403ddfd70387c8bef1c65770854c01034`; implementation pin
`0a8631d4e033af39025b06c64fc50496782269f4` matched the runtime tree. All
five public inputs, host/toolchain limits and absent output roots passed.
The registered shell syntax, report paths, exact row accounting, raw-horizon
LOO controls and non-self pairing were checked read-only.
