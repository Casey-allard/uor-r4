# Generated-state offset-conditioned density stop gate

**Status:** STOPPED AT PREFLIGHT

## Question

Can all generated terminal rows, searched by an offset-conditioned
class-balanced k-nearest density rule, identify held-out reply endpoints and
complete exact replies where the fixed eight-prototype gate did not?

This is D84 CPU development research on the Ryzen 9 9950X. It does not change
integer serving and is not a language or GPU-qualification claim.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `c8180c1576970069dcff395889460defad2a2d6c` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Data split

Use deepseek-137's 16-key split: 98 training and 103 evaluation samples for
each artifact, with runner seed 1. Seed 1, seed 2 and the seed-1 repeat use
the same prompt data but separate fitted artifacts.

## Minimum test power

The comparison is exact and deterministic over complete generated
trajectories. A token or gate mismatch fails the mechanism. Two trained
artifacts and a separate one-worker repeat prevent a one-seed or worker-order
claim. The result remains local to these public development artifacts.

## Fixed construction

Use the deepseek-137 memory and 2,048-bit modulo-256 content readout. Generate
training replies with the exact content arm and derive every generated state:

- terminal when its offset equals normalized reference length;
- nonterminal otherwise;
- store the complete signature and generated offset.

Fit separate terminal and nonterminal row sets. For each query:

- choose k from 1, 2, 4 or 8;
- use only same-class rows whose offsets differ by at most two;
- add the maximum signature distance for missing neighbours;
- score
  `sum(k nearest nonterminal distances) - sum(k nearest terminal distances)`;
- choose k and margin by leave-one-row-out training;
- require nonterminal false stops at or below 1%;
- maximize terminal recall, then lower false stops, larger margin and smaller
  k in that order.

Emit BOS at or above the fitted margin. The content choice before the gate and
every ungated token remain the positive code arm's decisions. The
class-preserving control rotates terminal labels through the same row order
without changing offsets or class counts.

## Preflight

Build the release driver and run `--select-only` for both seeds using
`--stop-density-neighbors 8 --stop-density-offset-radius 2`.

Proceed only when each seed has:

- at least 84 terminal and 1,484 nonterminal generated rows;
- training terminal recall at least 26/84;
- nonterminal false stops at or below its 1% limit;
- a selected k in `{1,2,4,8}`;
- different true and shuffled gate hashes.

The preflight output, stderr and resource reading are retained even when it
stops the run. A driver-level refusal is wrapped in a structured failure JSON
rather than leaving only an empty stdout file.

## Arms and bar

Run no-gate code, true density, shuffled density and the historical
eight-prototype gate over complete 16-token replies.

Both seeds must pass:

- true training recall at least **26/84**, with nonterminal false stops at or
  below **1%**;
- at least **9/103** BOS completions and at least **8** at or before the
  normalized reference length;
- at least **8/103** exact reference replies that stop at BOS;
- true BOS completion exceeds no-gate and shuffled by
  `candidate_only >= max(2, ceil(3 * sqrt(control_only)))`;
- true top-1 and matching depth are not below no-gate;
- every ungated token equals the code arm and every fired decision is BOS;
- true terminal-row coverage exceeds shuffled by
  `candidate_only >= max(2, ceil(2 * sqrt(control_only)))`;
- gate hashes differ and row counts match the generated training rows; and
- a seed-1 repeat at one worker matches every non-timing report field and
  generated JSON file from the 32-worker seed-1 run.

## Instrument gates

- Source and five public inputs match.
- Generated terminal labels and offsets are exact.
- Density fitting and label shuffle are deterministic.
- Missing-neighbour penalties prevent sparse offsets from creating confidence.
- Fired decisions are BOS and ungated token mismatches are zero.
- Workers 1 and 32 produce identical generated evidence.
- Full release tests, scoped Clippy, rustfmt and manifests pass.

## Stop rules

Stop on source/input/split drift, missing classes, preflight failure, vacuous
shuffle, ungated token mismatch, worker nondeterminism, resource breach or any
need to change serving, integer-session, CUDA, float-training or Claude-owned
source.

## Decision consequence

A pass makes offset-conditioned generated-state density a supported CPU
termination candidate and justifies a later artifact-bound serving unit. A
fail closes this full-row density representation and requires a different
continuation objective or trajectory representation. Neither result alone
establishes useful chat.

## Outcome

Attempt 1 executed implementation pin
`c8180c1576970069dcff395889460defad2a2d6c`. The release driver build, full
`uor-r4-cpu-train` library tests, driver tests, scoped Clippy and targeted
rustfmt passed. All five public inputs matched their registered hashes.

The seed-1 preflight stopped the registered run:

| Reading | True density | Shuffled density | Registered limit |
|---|---:|---:|---:|
| Training terminal rows | 84 | 84 | at least 84 |
| Training nonterminal rows | 1,484 | 1,484 | at least 1,484 |
| Terminal recall | **19/84** | **19/84** | at least 26/84 |
| Nonterminal false stops | 9/1,484 | 11/1,484 | at most 14/1,484 |
| Selected k | 8 | 8 | in `{1,2,4,8}` |
| Fitted margin | 916 | 844 | reported |

The true and shuffled gate hashes differed, the selected row counts matched
the 1,568 generated training rows, and the preflight process completed in
0.62 s at 236,356 KiB peak RSS. Recall missed the registered floor, so the
stop rule prevented the seed-2 preflight, all public generation arms and the
one-worker repeat. The 26/84 floor was not changed after observing the result.

The exact stopped output and its checksums are under
[`evidence-attempt-1/`](../../labs/cpu-training-pilot-stop-density-2026-11-04/evidence-attempt-1/).
Measured compute and the ledger chain are in the
[resource ledger](../../labs/cpu-training-pilot-stop-density-2026-11-04/resource-ledger.json).

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT and pre-run review | 600 s |
| release build, full controls and two-seed preflight | 1,500 s |
| two seeds plus repeat | 900 s |
| delivery and evidence | 300 s |
| total increment | **3,300 s** |

- Build: 16 jobs, 4 GiB RSS, 2 GiB target maximum.
- Full controls: 16 jobs, 4 GiB RSS, separate 3 GiB target maximum.
- Runs: 32 workers for seed 1/2, one worker for the repeat, 1 GiB RSS and
  600 s per run.
- Retained evidence: 192 MiB maximum.
- Previous ledger:
  `cpu-training-pilot-content-serving-2026-11-04/resource-ledger.json` at
  `35,683.988816343 s` measured and `52,027.677487699 s` admitted cumulative.
- Admitted cumulative projection: **55,327.677487699 s**.

## Steps

After FIT and a fresh GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-stop-density
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/stop-density/2026-11-04
BUILD=$SOURCE/target/deepseek/stop-density-142
CONTROL_BUILD=$SOURCE/target/deepseek/stop-density-controls-142
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=c8180c1576970069dcff395889460defad2a2d6c
MIN_FREE_KIB=31588352
RUNTIME_PATHS=(.cargo Cargo.toml Cargo.lock rust-toolchain.toml crates third_party)
free_kib() { df -Pk "$1" | awk 'NR == 2 { print $4 }'; }
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
env -u RUSTUP_TOOLCHAIN rustfmt --edition 2021 --check --config skip_children=true \
  "$SOURCE/crates/uor-r4-cpu-train/src/content_readout.rs" \
  "$SOURCE/crates/uor-r4-cpu-train/src/bin/cpu-content-readout-driver.rs" \
  "$SOURCE/crates/uor-r4-cpu-train/src/lib.rs"
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
      --reply-cap 16 --stop-prototypes 8 --stop-density-neighbors 8 \
      --stop-density-offset-radius 2 --seed 1 --select-only \
      >"$PIPELINE/seed-$seed.preflight.json" \
      2>"$PIPELINE/seed-$seed.preflight.stderr.txt" || status=$?
  if test "$status" -ne 0; then
    jq -n --arg seed "$seed" --argjson exit "$status" \
      --arg stderr "$(cat "$PIPELINE/seed-$seed.preflight.stderr.txt")" \
      '{schema:"uor-r4.cpu-content-stop-density-preflight-failure/1",
        seed:$seed, exit:$exit, stderr:$stderr}' \
      >"$PIPELINE/seed-$seed.preflight-failure.json"
    return "$status"
  fi
  PREFLIGHT_RSS=$(awk -F': ' '/Maximum resident set size/ { print $2 }' \
    "$PIPELINE/seed-$seed.preflight.time.txt")
  test "$PREFLIGHT_RSS" -le 1048576
  jq -e '.modes.gate_density as $d | $d.training as $g |
    $g.terminal_rows >= 84 and
    $g.nonterminal_rows >= 1484 and
    $g.terminal_stops >= 26 and
    $g.nonterminal_stops <= $g.nonterminal_stop_limit and
    ([1,2,4,8] | index($g.leave_one_out_neighbors)) != null and
    $g.gate.terminal_rows != [] and
    $g.gate.nonterminal_rows != [] and
    ($g.gate.terminal_rows | length) == $g.terminal_rows and
    ($g.gate.nonterminal_rows | length) == $g.nonterminal_rows and
    $d.true_sha256 != $d.shuffled_sha256' \
    "$PIPELINE/seed-$seed.preflight.json" >/dev/null
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
    --reply-cap 16 --stop-prototypes 8 --stop-density-neighbors 8 \
    --stop-density-offset-radius 2 --seed 1 \
    >"$PIPELINE/$label.stdout.json"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 1048576
  jq -e '.bar.code.structural_pass and
    (.bar.gate_density != null)' "$out/report.json" >/dev/null
  test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
  test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
}
run 1 seed-1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd 32
run 2 seed-2 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 32
run 1 seed-1-repeat a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd 1
SEED_1_PASS=$(jq -r '.bar.gate_density.passes == true' \
  "$PIPELINE/seed-1/report.json")
SEED_2_PASS=$(jq -r '.bar.gate_density.passes == true' \
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
(cd "$PIPELINE" && find . -type f ! -name SHA256SUMS -print0 |
  sort -z | xargs -0 sha256sum >SHA256SUMS &&
  sha256sum -c SHA256SUMS --quiet)
test "$(du -sb "$PIPELINE" | cut -f1)" -le 201326592
test -z "$(git -C "$SOURCE" status --porcelain -- "${RUNTIME_PATHS[@]}")"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- "${RUNTIME_PATHS[@]}"
test "$BAR_PASS" = true
test "$REPEAT_PASS" = true
```

Any source, input, split, gate, token identity, worker-order, repeat or
resource mismatch is an instrument fault requiring a new FIT.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 offset-conditioned density stop gate:
docs/harness/prep/cpu-training-pilot-stop-density-2026-11-04.md
head: 7255d015371fbeb780c16b4b38f74899777b8d11
implementation pin: c8180c1576970069dcff395889460defad2a2d6c
reviewer: codex-fresh-context-fit-attempt-4
note: Failed-bar evidence is sealed before classification; runtime identity covers
  .cargo and strips compiler/toolchain overrides; source, inputs, resource chain,
  preflight, arms, controls and repeat checks are registered and executable.
date: 2026-11-04

A FIT verdict clears compute for this configuration only (D22).

## Pre-run GO

GO - reviewer `codex-fresh-context-go-attempt-1`, 2026-11-04. HEAD was
clean at `85f2894081f915c1cc370ec5cf4e3be4147340e4`; the design head
`7255d0153` and implementation pin `c8180c157` matched the registered
runtime tree. All five public inputs, the resource-ledger chain, current
host/toolchain state and absent output roots passed. The registered shell
syntax, driver flags and preflight JSON predicates were checked read-only.
The full controls, public preflight and public runs remain post-GO gates.
