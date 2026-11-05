# Combined trajectory-density termination readout

**Status:** `TEST FITNESS: FIT`

## Question

Does the current modulo-256 content code plus a phase-aligned first-order
transition separate generated reply endpoints from ordinary continuation under
the existing termination-density rule?

Deepseek-142's current-code density stopped at 19/84 terminal recall.
Deepseek-146's phase-only transition stopped at 14/84. This sheet tests the
combined termination state while retaining the code-only content readout.

This is D84 CPU development research on the Ryzen 9 9950X. It uses the existing
serving-parity controls but does not change integer serving, `UORLUT01`, CUDA,
float training or Claude-owned source.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `1bf1a72fa` |
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

At generated offset `t`, retain the 2,048-bit current modulo-256 code and derive
a 2,048-bit relative transition:

```text
phase_t = popcount(previous_code(q_(t-1))) mod 64
transition_t = rotate_left_64(current_code(q_t), phase_t)
               XOR previous_code(q_(t-1))
termination_t = current_code(q_t) || transition_t
```

Offset zero uses an all-zero transition because there is no predecessor.
Content selection continues to use only the current code. Fit the existing
class-balanced density rule over 4,096-bit combined termination states with the
unchanged maximum neighbour set `{1,2,4,8}` and generated-offset radius `2`.
Fit thresholds under the existing one-percent nonterminal false-stop limit.

The matched control permutes terminal labels through the same combined rows and
must change the fitted gate and at least one leave-one-out firing decision.

## Cancel probe

Deepseek-142 closed current-code density at 19/84 terminal recall against the
26/84 floor. Deepseek-143 closed exact-offset horizon at 66/1,484 false stops
against the 14-row limit. Deepseek-144 and deepseek-145 closed current-plus-XOR
content readout, not termination. Deepseek-146 closed phase-only transition
density at 14/84. None combined the surviving current state with the
phase-aligned transition for termination.

## Preflight

Run seed 1 first with `--stop-trajectory-neighbors 8`. Proceed to seed 2 only if:

- the combined rows contain at least 84 terminal and 1,484 nonterminal
  generated decisions;
- true combined density recalls at least 26/84 terminal rows;
- true combined density emits at most 14/1,484 false stops;
- the shuffled-label gate changes at least one leave-one-out firing decision;
- the code-prefix audit reports at least one decision and zero mismatches;
- both pinned FBS artifact files and the three shared corpus files match; and
- source and all five public inputs match.

Preserve and seal the preflight even when it stops.

## Arms and bar

If the preflight passes, run code, combined density and shuffled combined
density over complete 16-token replies on both seeds.

Both seeds must:

- recall at least 26/84 terminal rows and emit at most 14/1,484 false stops;
- beat the shuffled-label terminal arm and the code arm by the prompt-level
  paired two-sigma stopping rule;
- keep every ungated content decision equal to code;
- produce at least 9 exact references and 9 BOS-complete replies, including at
  least 8 stops within the reference length;
- keep generated top-1 and matching depth not below code; and
- reproduce every non-timing field in a one-worker seed-1 repeat.

Report terminal/false counts, threshold, selected neighbours, gate hashes,
leave-one-out prediction changes, exact-reference/BOS counts, top-1/depth,
openings, distinct/repeated replies, workers, wall and RSS.

## Instrument gates

- Combined packing places the current code first and the phase transition
  second.
- The phase uses only the previous current-state code; offset zero is all zero.
- Fitting, evaluation and generation use the same combined history.
- The density fitter accepts exactly 2,048- or 4,096-bit signatures and rejects
  mixed row widths.
- Label permutation changes the gate and at least one firing decision.
- Ungated content remains byte-identical.
- Worker-count repeat matches all non-timing outputs.
- Full CPU release tests, scoped Clippy, rustfmt and evidence manifests pass.

## Minimum test power

The split and model seeds are fixed. Fitting and generation are exact and
deterministic. Two trained artifacts and a one-worker seed-1 repeat prevent a
one-seed or worker-order claim. The result applies only to this public
development corpus and combined trajectory construction.

## Stop rules

Stop on source/input/split drift, combined-history mismatch, degenerate or
vacuous transitions, preflight failure, vacuous label control, worker
nondeterminism, resource breach, or any need to change integer serving, CUDA,
float training or Claude-owned source.

## Decision consequence

A pass makes combined trajectory-density termination a supported CPU candidate
and justifies a separate artifact-bound serving integration unit. A fail closes
this exact current-code-plus-phase-transition construction, not all learned
transition geometry.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT, GO and exact-head rechecks | 900 s |
| release build, controls and seed-1 preflight | 1,200 s |
| seed 2 and one-worker repeat when admitted | 900 s |
| delivery and evidence | 300 s |
| total increment | **3,300 s** |

- Build and controls: 16 jobs, 4 GiB RSS, separate 2 GiB and 3 GiB targets.
- Runs: 32 workers for seeds 1/2 and one worker for the repeat, 1 GiB RSS and
  600 s each.
- Retained evidence limit: 201,326,592 bytes.
- Previous measured cumulative is `36,008.048816343 s`; the admitted
  cumulative is `66,427.677487699 s`.
- New admitted cumulative: **69,727.677487699 s**.

The named model-time ledger at
`.uor-models/native-joint-learning-2026-09-04/model-time.json` is `ABSENT` and
is not treated as zero.

## Steps

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-trajectory-density
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/trajectory-density/2026-11-04
BUILD=$SOURCE/target/deepseek/trajectory-density-147
CONTROL_BUILD=$SOURCE/target/deepseek/trajectory-density-controls-147
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=1bf1a72fa
MIN_FREE_KIB=31588352
free_kib() { df -Pk "$1" | awk 'NR == 2 { print $4 }'; }

test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
test ! -e "$PIPELINE"
test ! -e "$BUILD"
test ! -e "$CONTROL_BUILD"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- \
  .cargo Cargo.toml Cargo.lock rust-toolchain.toml crates third_party
sha256sum -c <(printf '%s\n' \
  'a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd  '"$DUAL"'/dual-c131-seed-1-raw_both.uorlut' \
  'da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1  '"$DUAL"'/dual-c131-seed-2-raw_both.uorlut' \
  '0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b  '"$DEV"'/dev-tokens.u16' \
  'a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078  '"$DEV"'/dev-response-mask.u8' \
  'ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab  '"$DEV"'/dev-manifest.json') --quiet
mkdir -p "$PIPELINE"
seal_pipeline() {
  (
    cd "$PIPELINE"
    find . -type f ! -name SHA256SUMS ! -name SHA256SUMS.tmp -print0 |
      sort -z | xargs -0 -r sha256sum >SHA256SUMS.tmp
    mv SHA256SUMS.tmp SHA256SUMS
    sha256sum -c SHA256SUMS --quiet
  )
}
trap 'status=$?; trap - EXIT; seal_pipeline || status=$?; exit "$status"' EXIT

/usr/bin/time -v -o "$PIPELINE/build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" cargo build --locked --release \
  -p uor-r4-cpu-train --bin cpu-content-readout-driver
/usr/bin/time -v -o "$PIPELINE/control-lib.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$CONTROL_BUILD" cargo test --locked --release \
  -p uor-r4-cpu-train --lib
/usr/bin/time -v -o "$PIPELINE/control-bin.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$CONTROL_BUILD" cargo test --locked --release \
  -p uor-r4-cpu-train --bin cpu-content-readout-driver
/usr/bin/time -v -o "$PIPELINE/control-clippy.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$CONTROL_BUILD" cargo clippy --locked \
  -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings
cargo fmt -p uor-r4-cpu-train -- --check
test "$(awk -F': ' '/Maximum resident set size/ {print $2}' "$PIPELINE/build.time.txt")" -le 4194304
test "$(du -sb "$BUILD" | cut -f1)" -le 2147483648
test "$(awk -F': ' '/Maximum resident set size/ {if ($2 > max) max=$2} END {print max}' "$PIPELINE"/control-*.time.txt)" -le 4194304
test "$(du -sb "$CONTROL_BUILD" | cut -f1)" -le 3221225472

BIN=$BUILD/release/cpu-content-readout-driver
run_preflight() {
  seed=$1
  artifact=$2
  /usr/bin/time -v -o "$PIPELINE/seed-$seed.preflight.time.txt" \
    timeout 600s env RAYON_NUM_THREADS=32 "$BIN" \
      "$DUAL/dual-c131-seed-$seed-raw_both.uorlut" \
      "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" "$DEV/dev-manifest.json" \
      "$PIPELINE/select-only-unused-$seed" \
      --expected-artifact-sha256 "$artifact" \
      --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
      --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
      --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
      --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
      --reply-cap 16 --stop-trajectory-neighbors 8 --seed 1 --select-only \
      >"$PIPELINE/seed-$seed.preflight.json"
  test "$(awk -F': ' '/Maximum resident set size/ {print $2}' \
    "$PIPELINE/seed-$seed.preflight.time.txt")" -le 1048576
  jq -e '.modes.gate_density as $g |
    $g.rows == 1568 and
    $g.signature_mode == "current_code_phase_transition" and
    $g.training.gate.signature_bits == 4096 and
    $g.training.terminal_rows >= 84 and
    $g.training.nonterminal_rows >= 1484 and
    $g.training.terminal_stops >= 26 and
    $g.training.nonterminal_stops <= $g.training.nonterminal_stop_limit and
    $g.training.gate.neighbors <= 8 and
    $g.training.gate.offset_radius == 2 and
    $g.predictions_changed > 0 and
    $g.training_content_audit.decisions > 0 and
    $g.training_content_audit.mismatches == 0' \
    "$PIPELINE/seed-$seed.preflight.json" >/dev/null
}
run_preflight 1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run_preflight 2 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1

run_arm() {
  seed=$1
  label=$2
  workers=$3
  artifact=$4
  /usr/bin/time -v -o "$PIPELINE/$label.time.txt" \
    timeout 600s env RAYON_NUM_THREADS="$workers" "$BIN" \
      "$DUAL/dual-c131-seed-$seed-raw_both.uorlut" \
      "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" "$DEV/dev-manifest.json" \
      "$PIPELINE/$label" --expected-artifact-sha256 "$artifact" \
      --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
      --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
      --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
      --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
      --reply-cap 16 --stop-trajectory-neighbors 8 --seed 1 \
      >"$PIPELINE/$label.stdout.json"
  test "$(jq -r '.peak_rss_kib' "$PIPELINE/$label/report.json")" -le 1048576
}
run_arm 1 seed-1 32 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run_arm 2 seed-2 32 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
run_arm 1 seed-1-repeat 1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd

SEED_1_PASS=$(jq -r '.bar.gate_density.passes == true' "$PIPELINE/seed-1/report.json")
SEED_2_PASS=$(jq -r '.bar.gate_density.passes == true' "$PIPELINE/seed-2/report.json")
printf '{"seed_1_passes":%s,"seed_2_passes":%s}\n' \
  "$SEED_1_PASS" "$SEED_2_PASS" >"$PIPELINE/bar-summary.json"

jq 'del(.wall_elapsed_ms, .peak_rss_kib, .rayon_workers)' \
  "$PIPELINE/seed-1/report.json" >"$PIPELINE/seed-1.normalized.json"
jq 'del(.wall_elapsed_ms, .peak_rss_kib, .rayon_workers)' \
  "$PIPELINE/seed-1-repeat/report.json" >"$PIPELINE/seed-1-repeat.normalized.json"
cmp "$PIPELINE/seed-1.normalized.json" "$PIPELINE/seed-1-repeat.normalized.json"
while IFS= read -r file; do
  cmp "$file" "$PIPELINE/seed-1-repeat/${file#"$PIPELINE/seed-1/"}"
done < <(find "$PIPELINE/seed-1" -maxdepth 1 -type f -name '*.json' \
  ! -name report.json | sort)

test "$(du -sb "$PIPELINE" | cut -f1)" -le 201326592
test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
test "$SEED_1_PASS" = true
test "$SEED_2_PASS" = true
```

Stop before seed 2 when any preflight condition fails. Seal every generated
file in the attempt directory without editing it afterward.

## Rollback boundary

The unit changes only `uor-r4-cpu-train` and research records. Reverting it
restores the separate code and phase termination modes and leaves integer
serving, `UORLUT01`, CUDA and Claude-owned paths untouched.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 deepseek-147
sheet: docs/harness/prep/cpu-training-pilot-trajectory-density-2026-11-04.md
reviewer: codex fresh-context FIT round 2
note: Combined 4,096-bit signature, previous-state history, select-only
  content-prefix audit, fixed density controls, public inputs, pinned 16-job
  build controls, resource limits and full-run recipe verified.
date: 2026-11-04

A FIT verdict clears compute for the configuration in this sheet only (D22).

## Pre-run GO

GO - reviewer `codex fresh-context GO`, 2026-11-04. HEAD was clean at
`42ff65e1c`; implementation pin `1bf1a72fa` matched the runtime paths. The five
public inputs, host/toolchain/resource limits, absent output roots, failure-seal
trap, seed ordering, full-run commands and repeat comparison were checked
read-only.

## Outcome

Seed 1 of the registered preflight stopped before seed 2 or generation. The
combined trajectory-density gate retained the exact 1,568 generated rows,
recalled **16/84** terminal endpoints against the **26/84** floor, and emitted
**11/1,484** false stops inside the 14-row limit. Its shuffled-label control
changed **58** leave-one-out firing decisions, recalled 18/84 and emitted
13/1,484 false stops. The ungated code-prefix audit covered 1,145 decisions
with zero mismatches.

The stopped evidence is in
`docs/labs/cpu-training-pilot-trajectory-density-2026-11-04/evidence-attempt-1/`
with `SHA256SUMS`
`f56a5dc8be880189fb71776cac9f9979c6a7a0a6daa4a1c249230bcfa6c90467`.
No threshold or representation was changed after observing the result.
