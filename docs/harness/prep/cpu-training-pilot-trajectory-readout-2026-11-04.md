# First-order generated trajectory content readout

**Status:** STOPPED AT PREFLIGHT

## Question

Does adding the change between the current and previous generated geometric
states improve exact next-content selection beyond the current modulo-256
content readout?

This is D84 CPU development research on the Ryzen 9 9950X. It does not change
integer serving, `UORLUT01`, CUDA, float training or Claude-owned source.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `99a0687f3a4a7cf87cc5bdaff07ce6330d5d3968` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Data split

Use the deepseek-137 16-key split: 98 training and 103 evaluation samples per
artifact, runner seed 1.

## Fixed construction

Keep the positive modulo-256 current-state code as the first 2,048 bits. Add
its first-order transition as the next 2,048 bits:

```text
current_code(q_t) || (current_code(q_t) XOR current_code(q_(t-1)))
```

At offset zero, the transition half is zero. The previous term is the previous
current-state code, not the previous combined trajectory signature.

Use the existing generated content arm, turn memory and one-based trajectory
history during both fitting and evaluation. Fit a 4,096-bit content readout:

- exact Hamming distance over current and transition halves;
- commonest reference target among the nearest rows, then lower token ID;
- the smallest threshold covering at least 90% of leave-one-out rows;
- exact writer fallback beyond the threshold.

The control rotates targets through the same trajectory rows and must change
the effective mapping.

## Cancel probe

Deepseek-136 tested current-state sign signatures. Deepseek-137 tested
current-state modulo-256 code. Neither used transition history. Deepseek-142
and 143 changed termination labels without improving content. This unit keeps
termination out of the decision and tests only first-order trajectory context
for the next content token.

## Preflight

Build the release driver and run `--select-only --trajectory-readout` for both
seeds.

Proceed only if, for each seed:

- trajectory training has at least 640 rows, at least 16 target values and 90%
  leave-one-out coverage;
- the trajectory signature is exactly 4,096 bits;
- trajectory leave-one-out top-1 is not below code;
- true and shuffled target mappings differ; and
- source and all five inputs match.

The preflight and structured failure, if any, are sealed before exit.

## Arms and bar

Run writer, code, trajectory and trajectory-shuffled over complete 16-token
replies. Both seeds must pass:

- trajectory top-1 and matching-prefix depth are not below code;
- trajectory top-1 beats code and trajectory-shuffled by the prompt-level rule
  `candidate_only >= max(2, ceil(3 * sqrt(control_only)))`;
- trajectory depth beats code by
  `candidate_only >= max(2, ceil(2 * sqrt(control_only)))`;
- every trajectory decision is either its fitted readout target or the exact
  writer fallback; and
- a one-worker seed-1 repeat matches every non-timing report field and every
  generated JSON file from the 32-worker run.

Exact-reference and BOS counts are reported but do not decide this
content-selection unit.

Report LOO top-1, ambiguity, coverage and threshold for code and trajectory;
generated top-1, depth, openings, distinct/repeated replies; mapping hashes;
workers; wall and RSS.

## Instrument gates

- Source and all five public inputs match.
- Transition packing is current code followed by XOR against the previous
  current code; offset zero uses a zero transition.
- The same history rule is used during fit and evaluation.
- Target shuffle preserves trajectory signatures and changes the effective
  mapping.
- Readout fallback and lower-token ties are deterministic.
- One-worker and 32-worker generated evidence match.
- Full CPU release tests, scoped Clippy, rustfmt and evidence manifests pass.

## Minimum test power

The split and model seeds are fixed. Fitting and generation are exact and
deterministic, so a token or mapping mismatch fails the mechanism. Two trained
artifacts and a one-worker seed-1 repeat prevent a one-seed or worker-order
claim. The result applies only to the named public development corpus.

## Stop rules

Stop on source/input/split drift, trajectory width or history mismatch,
preflight failure, vacuous shuffle, worker nondeterminism, resource breach or
any need to change integer serving, CUDA, float training or Claude-owned
source.

## Decision consequence

A pass makes first-order trajectory context a supported CPU content candidate
and justifies artifact-bound serving integration. A fail closes this
current-plus-XOR construction and requires higher-order transition history,
learned transition geometry or a different content objective.

## Outcome

Attempt 1 executed implementation pin
`99a0687f3a4a7cf87cc5bdaff07ce6330d5d3968`. The release driver, 255 library
tests, ten driver tests, scoped Clippy and targeted rustfmt passed. Source and
all five public inputs matched.

| Seed 1 reading | Code | Trajectory | Registered rule |
|---|---:|---:|---:|
| Training rows | 894 | 894 | at least 640 |
| Target values | 207 | 207 | at least 16 |
| Covered rows | 807 | 806 | at least 90% |
| Leave-one-out top-1 | **300** | **297** | trajectory not below code |
| Ambiguous rows | 277 | **254** | reported |
| Fitted radius | 537 | 1,080 | reported |
| Signature bits | 2,048 | 4,096 | trajectory exactly 4,096 |

The 4,096-bit trajectory mapping differed from its shuffled target control on
632 effective assignments. It reduced ambiguity, but its top-1 fell below the
existing code readout, so the preflight stopped before seed 2, generation or
the repeat. The floor and representation were not changed after observation.

Exact output is under
[`evidence-attempt-1/`](../../labs/cpu-training-pilot-trajectory-readout-2026-11-04/evidence-attempt-1/).
Measured compute and cumulative accounting are in the
[resource ledger](../../labs/cpu-training-pilot-trajectory-readout-2026-11-04/resource-ledger.json).

## Repair attempt 2

Fresh exact-head review found that full-generation fallback accounting compared
reference-aligned decisions with counters covering every generated decision.
The repair adds an explicit generated-decision count and a capped-horizon
regression without changing the registered construction, inputs, split,
threshold rule or stop rule.

Attempt 2 rebuilt from source `4388db081b30ee0c554f99b41500c667cf08eb8c` and
reran the seed-1 `--select-only --trajectory-readout` preflight. The preflight
report and structured failure are byte-identical to attempt 1
(`06311414…693bdc` and `17ac1246…b8879`): code top-1 300/894, trajectory
297/894, coverage 807/806, ambiguity 277/254, and radii 537/1,080.

The corrected source therefore stops for the registered mechanism reason, not
an accounting fault. Attempt 1 remains sealed unchanged; attempt 2 is under
`docs/labs/cpu-training-pilot-trajectory-readout-2026-11-04/evidence-attempt-2/`
with `SHA256SUMS` `140cf3f54cf60931d149bd89359444e2caf67629f0e2485dcc4aa4ee5e288ee8`.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT and pre-run review | 600 s |
| release build, controls and two-seed preflight | 1,500 s |
| two seeds plus repeat | 900 s |
| delivery and evidence | 300 s |
| total increment | **3,300 s** |

- Build and controls: 16 jobs, 4 GiB RSS, separate 2 GiB and 3 GiB targets.
- Runs: 32 workers for seeds 1/2 and one worker for the repeat, 1 GiB RSS and
  600 s each.
- Retained evidence limit: 201,326,592 bytes.
- Previous ledger:
  `cpu-training-pilot-continuation-horizon-2026-11-04/resource-ledger.json` at
  `35,821.238816343 s` measured and `58,627.677487699 s` admitted cumulative.
- Admitted cumulative projection: **61,927.677487699 s**.

## Steps

After FIT and a fresh GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-trajectory-readout
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/trajectory-readout/2026-11-04
BUILD=$SOURCE/target/deepseek/trajectory-readout-144
CONTROL_BUILD=$SOURCE/target/deepseek/trajectory-readout-controls-144
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=99a0687f3a4a7cf87cc5bdaff07ce6330d5d3968
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
      --reply-cap 16 --trajectory-readout --seed 1 --select-only \
      >"$PIPELINE/seed-$seed.preflight.json" \
      2>"$PIPELINE/seed-$seed.preflight.stderr.txt" || status=$?
  if test "$status" -ne 0; then
    jq -n --arg seed "$seed" --argjson exit "$status" \
      --arg stderr "$(cat "$PIPELINE/seed-$seed.preflight.stderr.txt")" \
      '{schema:"uor-r4.cpu-content-trajectory-preflight-failure/1",
        seed:$seed, exit:$exit, stderr:$stderr}' \
      >"$PIPELINE/seed-$seed.preflight-failure.json"
    return "$status"
  fi
  PREFLIGHT_RSS=$(awk -F': ' '/Maximum resident set size/ { print $2 }' \
    "$PIPELINE/seed-$seed.preflight.time.txt")
  test "$PREFLIGHT_RSS" -le 1048576
  if ! jq -e '.modes.trajectory as $t | .modes.code.training as $c |
    $t.training.rows >= 640 and
    $t.training.target_values >= 16 and
    $t.training.covered_rows * 10 >= $t.training.rows * 9 and
    $t.training.readout.signature_bits == 4096 and
    $t.training.leave_one_out_top1 >= $c.leave_one_out_top1 and
    $t.mapping.differs == true' \
    "$PIPELINE/seed-$seed.preflight.json" >/dev/null; then
    jq -n --arg seed "$seed" \
      --slurpfile preflight "$PIPELINE/seed-$seed.preflight.json" \
      '{schema:"uor-r4.cpu-content-trajectory-preflight-bar-failure/1",
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
    --reply-cap 16 --trajectory-readout --seed 1 \
    >"$PIPELINE/$label.stdout.json"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 1048576
  jq -e '.bar.code.structural_pass and (.bar.trajectory != null)' \
    "$out/report.json" >/dev/null
  test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
  test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
}
run 1 seed-1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd 32
run 2 seed-2 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 32
run 1 seed-1-repeat a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd 1
SEED_1_PASS=$(jq -r '.bar.trajectory.passes == true' \
  "$PIPELINE/seed-1/report.json")
SEED_2_PASS=$(jq -r '.bar.trajectory.passes == true' \
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

Any source, input, split, trajectory, mapping, worker-order, repeat or
resource mismatch is an instrument fault requiring a new FIT.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 deepseek-144
sheet: docs/harness/prep/cpu-training-pilot-trajectory-readout-2026-11-04.md
reviewer: codex-fresh-context-fit-attempt-2
note: Trajectory history, target shuffle, exact fallback and distinct
  two/three-sigma paired thresholds verified.
date: 2026-11-04

A FIT verdict clears compute for the configuration in that sheet only (D22).

## Pre-run GO

GO - reviewer `codex-fresh-context-go-attempt-2`, 2026-11-04. HEAD was
clean at `ffdb3421782e9fd487941fe201c3f47d8e4adc8e`; implementation pin
`99a0687f3a4a7cf87cc5bdaff07ce6330d5d3968` matched the runtime tree. The
five public inputs, host/toolchain/resource limits and absent output roots
passed. Shell syntax, JSON paths, trajectory history, target shuffle,
two/three-sigma pairing and readout/fallback accounting were checked
read-only.
