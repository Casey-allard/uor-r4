# Pre-registration: geometric copy arbitration

Run ID: `cpu-training-pilot-copy-geometric-2026-11-05`

## Decision

Can the shared FBS query geometry choose the correct next token among the
available exact-copy candidates? Deepseek-163 extended exact suffix search to
32 tokens but found no body-position change over longest-4. This unit retains
that candidate set and replaces depth-first selection with sign-Hamming
agreement between the current Q12 writer query and each candidate token's
shared vector-table row.

This is a bounded D84 CPU development screen. It is not GPU qualification,
complete-reply evidence, language evidence or serving evidence.

## Hypothesis and fixed construction

At each generated decision, collect exact suffix candidates at depths one
through 32. For every available candidate token, map the current 256
coordinate Q12 writer query and that token's 256 shared vector-table values to
sign bits, then count XOR bits. Choose the candidate by the total ordering:

1. smallest sign-Hamming distance;
2. deepest exact-suffix depth;
3. lower token id.

The arm replaces only the next token. The selected token is fed through the
same exact `CpuFbsForward`; writer fallback, memory, point-key behavior and
stopping are unchanged. The decision digest binds the selected distance.

Controls are longest-32, longest-4, `k2>k1`, fixed depth one, random
available depth in 1..32, and the unmodified writer. The legacy random 1..4
arm remains present but is diagnostic only.

## Prior evidence

Deepseek-163's fresh seed-2752 panel has 458 reference-reachable replies per
artifact seed. Longest-32 and longest-4 each score 300/4,758 correct body
positions; both score 409/4,661 on train. Extending the exact copy window
therefore did not improve scored body tokens. No prior complete-reply unit
ranks available exact-copy candidates by their geometric agreement with the
writer query.

## Base artifacts and source

- implementation commit: `2378b0414`;
- base: `7018b09cd`;
- generator tree at base:
  `869e65db6d0bb997a5f1094ab0de4efa1ac10c25`;
- seed 1: `dual-c131-seed-1-raw_both.uorlut`, SHA-256
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2: `dual-c131-seed-2-raw_both.uorlut`, SHA-256
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- tokenizer SHA-256
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- `copy_key_reply.rs` SHA-256
  `c2b50b032dbff0061d16fcc11b103915ab5607ab01d29380ea7c5e975d02744a`;
- `cpu-copy-key-reply.rs` SHA-256
  `232be51bcdf088ba89e3861bc18ef3184c6b726dd2c725f59cac22c8afd2d3f6`;
- `cpu-copy-key-reply-oracle.rs` SHA-256
  `41159845363e0946bfc3a253cb4a32a9fd4ded81ba0bcd1a71e77326f07c2a3e`;
- `fbs.rs` SHA-256
  `c45707fdb49f76d03a149934caa06e24c5867a6bbc530356cd77e34480428d90`.

## Data split

- Train diagnostic: existing panel 2711 and sealed train reports.
- Fresh decision: one new `cell=dev,dev` panel, 128 conversations, protocol 2,
  seed 2762, generated only after FIT and GO.
- Output root: `local/deepseek/copy-geometric/2026-11-05`.
- The output root and seed-2762 panel are absent at registration.

## Steps

Run from a fresh process only after FIT and GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-geometric
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-geometric/2026-11-05
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2
PANEL=$OUT/panels/dd-2762
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/copy-geometric
GEN_TARGET=$SOURCE/target/deepseek/copy-geometric-generator
IMPL=2378b0414
GEN_TREE=869e65db6d0bb997a5f1094ab0de4efa1ac10c25
BASE=7018b09cd

test ! -e "$OUT"
test ! -e "$PANEL"
git -C "$SOURCE" diff --quiet "$IMPL" -- crates/uor-r4-cpu-train
test "$(git -C "$SOURCE" rev-parse "$BASE":docs/labs/one-model-end-to-end/instrument)" = "$GEN_TREE"
printf '%s  %s\n' \
  a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 "$TOKENIZER" \
  a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$SEED1" \
  da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$SEED2" \
  37715a5d621c5b8c2269940505491d13d84c37b414fa2eb9646a2de54000e6ee "$TRAIN/seed1-tt.json.gz" \
  4a6a274822f287a806a52704417c3c2bd75bee604484ce610852cfd40e809372 "$TRAIN/seed2-tt.json.gz" \
  140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91 "$TRAIN/panels/tt-2711/tokens.u16" \
  d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad "$TRAIN/panels/tt-2711/response_mask.u8" \
  d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5 "$TRAIN/panels/tt-2711/manifest.json" |
  sha256sum -c -
printf '%s  %s\n' \
  c2b50b032dbff0061d16fcc11b103915ab5607ab01d29380ea7c5e975d02744a \
  "$SOURCE/crates/uor-r4-cpu-train/src/copy_key_reply.rs" \
  232be51bcdf088ba89e3861bc18ef3184c6b726dd2c725f59cac22c8afd2d3f6 \
  "$SOURCE/crates/uor-r4-cpu-train/src/bin/cpu-copy-key-reply.rs" \
  41159845363e0946bfc3a253cb4a32a9fd4ded81ba0bcd1a71e77326f07c2a3e \
  "$SOURCE/crates/uor-r4-cpu-train/src/bin/cpu-copy-key-reply-oracle.rs" \
  c45707fdb49f76d03a149934caa06e24c5867a6bbc530356cd77e34480428d90 \
  "$SOURCE/crates/uor-r4-cpu-train/src/fbs.rs" |
  sha256sum -c -

mkdir -p "$OUT/panels" "$OUT/reports"
seal() {
  (
    cd "$OUT"
    find . -type f ! -name SHA256SUMS ! -name SHA256SUMS.tmp -print0 |
      sort -z | xargs -0 -r sha256sum >SHA256SUMS.tmp
    mv SHA256SUMS.tmp SHA256SUMS
    sha256sum -c SHA256SUMS --quiet
  )
}
trap 'status=$?; trap - EXIT; seal || status=$?; exit "$status"' EXIT

cd "$SOURCE"
/usr/bin/time -v -o "$OUT/build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" \
  cargo build --locked --release -p uor-r4-cpu-train --bin cpu-copy-key-reply
/usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GEN_TARGET" \
  cargo build --locked \
  --manifest-path "$SOURCE/docs/labs/one-model-end-to-end/instrument/Cargo.toml" \
  --features panel-writer --bin heldout-panel
GEN=$GEN_TARGET/debug/heldout-panel

/usr/bin/time -v -o "$OUT/panel.time.txt" \
  timeout 900s "$GEN" out="$PANEL" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2762 protocol=2 \
  >"$OUT/panel.stdout.json"
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2762 and .dialogue_protocol_version == 2
  and .cell.value == "dev" and .cell.phrasing == "dev"
  and .conversations == 128 and .world == "m-world-v2"
  and .tokenizer.sha256 == $tokenizer_sha
' "$PANEL/manifest.json" >/dev/null

gzip -dc "$TRAIN/seed1-tt.json.gz" >"$OUT/reports/seed1-tt-source.json"
gzip -dc "$TRAIN/seed2-tt.json.gz" >"$OUT/reports/seed2-tt-source.json"
printf '%s  %s\n' \
  28a8caaffef6d815da86a6e84207729f3acd41892ced8a205cfe5248d7c2f62a \
  "$OUT/reports/seed1-tt-source.json" \
  466b94e7b4c1ae2d58337c51b07171c6d2229b23edaa366949e7a2ed8a5642e3 \
  "$OUT/reports/seed2-tt-source.json" |
  sha256sum -c -

BIN=$BUILD/release/cpu-copy-key-reply
run() {
  name=$1 panel=$2 artifact=$3
  /usr/bin/time -v -o "$OUT/reports/$name.time.txt" \
    timeout 900s env RAYON_NUM_THREADS=32 "$BIN" \
    "$panel/tokens.u16" "$panel/response_mask.u8" "$panel/manifest.json" \
    "$artifact" --cap 64 --seed 163 --allow-underpowered \
    >"$OUT/reports/$name.json"
}
run seed1-tt "$TRAIN/panels/tt-2711" "$SEED1"
run seed2-tt "$TRAIN/panels/tt-2711" "$SEED2"
run seed1-dd "$PANEL" "$SEED1"
run seed2-dd "$PANEL" "$SEED2"
run seed1-dd-repeat "$PANEL" "$SEED1"
run seed2-dd-repeat "$PANEL" "$SEED2"

normalize() {
  jq -S 'walk(if type == "object"
    then del(.wall_elapsed_us, .generated_tokens_per_second)
    else . end)' "$1"
}
cmp <(normalize "$OUT/reports/seed1-dd.json") \
  <(normalize "$OUT/reports/seed1-dd-repeat.json")
cmp <(normalize "$OUT/reports/seed2-dd.json") \
  <(normalize "$OUT/reports/seed2-dd-repeat.json")

VALID=1
POWER_OK=1
for name in seed1-tt seed2-tt seed1-dd seed2-dd; do
  report=$OUT/reports/$name.json
  case "$name" in
    seed1*) expected=a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd ;;
    seed2*) expected=da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 ;;
  esac
  if ! jq -e '.examples.reachable >= 200' "$report" >/dev/null; then
    printf '%s reachable-below-200\n' "$name" >>"$OUT/power-limited.txt"
    POWER_OK=0
  fi
  if ! jq -e --arg expected "$expected" '
    .settings.maximum_copy_depth == 32
    and .inputs.artifact_sha256 == $expected
    and .random_nonvacuity_from_longest_32.non_vacuous == true
    and .random_nonvacuity.non_vacuous == true
  ' "$report" >/dev/null; then
    printf '%s instrument-or-control-fault\n' "$name" >>"$OUT/instrument-fault.txt"
    VALID=0
  fi
  if ! jq -e 'all(.arms.geometric_copy_32.steps[];
    if .used_copy
    then .selected_sign_hamming != null
    else .selected_token == .writer_token and .selected_sign_hamming == null
    end)' "$report" >/dev/null; then
    printf '%s geometric-decision-fault\n' "$name" >>"$OUT/instrument-fault.txt"
    VALID=0
  fi
done

for seed in seed1 seed2; do
  jq '{
    train_not_below_longest_32: (
      .arms.geometric_copy_32.reading.correct_body_positions
      >= .arms.longest_32_to_1.reading.correct_body_positions
    ),
    geometric_candidate_premature_zero:
      .arms.geometric_copy_32.reading.premature_bos == 0,
    geometric_body_positions:
      .arms.geometric_copy_32.reading.correct_body_positions,
    longest_32_body_positions:
      .arms.longest_32_to_1.reading.correct_body_positions
  }' "$OUT/reports/$seed-tt.json" >"$OUT/reports/$seed-train-outcome.json"
  jq '{
    fresh_passes_longest_32:
      .comparisons.longest_32_to_1.correct_body_positions.passes,
    fresh_passes_longest_4:
      .comparisons.longest_4_to_1.correct_body_positions.passes,
    fresh_passes_k2:
      .comparisons.k2_backoff_k1.correct_body_positions.passes,
    fresh_passes_depth_1:
      .comparisons.fixed_depth_1.correct_body_positions.passes,
    fresh_passes_random:
      .comparisons.random_available_depth_32.correct_body_positions.passes,
    fresh_passes_writer:
      .comparisons.writer.correct_body_positions.passes,
    geometric_change_non_vacuous:
      .geometric_changes_from_longest_32.non_vacuous == true,
    geometric_premature_zero:
      .arms.geometric_copy_32.reading.premature_bos == 0,
    exact_reference: .arms.geometric_copy_32.reading.exact_reference,
    bos_complete: .arms.geometric_copy_32.reading.bos_complete
  }' "$OUT/reports/$seed-dd.json" >"$OUT/reports/$seed-dev-outcome.json"
done

gzip -9 "$OUT"/reports/seed1-tt-source.json "$OUT"/reports/seed2-tt-source.json \
  "$OUT"/reports/seed1-tt.json "$OUT"/reports/seed2-tt.json \
  "$OUT"/reports/seed1-dd.json "$OUT"/reports/seed2-dd.json \
  "$OUT"/reports/seed1-dd-repeat.json "$OUT"/reports/seed2-dd-repeat.json
test "$(du -sb "$OUT" | cut -f1)" -le 67108864
if test "$VALID" -ne 1; then
  printf 'INSTRUMENT_FAULT\n' >"$OUT/outcome.txt"
  exit 1
fi
if test "$POWER_OK" -ne 1; then
  printf 'POWER_LIMITED\n' >"$OUT/outcome.txt"
  exit 0
fi
jq -s '
  {
    passes: (all(.[]; .train_not_below_longest_32 == true
      and .geometric_candidate_premature_zero == true)),
    outcomes: .
  }
' "$OUT/reports/seed1-train-outcome.json" \
  "$OUT/reports/seed2-train-outcome.json" \
  >"$OUT/reports/train-outcome.json"
jq -s '
  {
    passes:
      (all(.[];
        .fresh_passes_longest_32 == true
        and .fresh_passes_longest_4 == true
        and .fresh_passes_k2 == true
        and .fresh_passes_depth_1 == true
        and .fresh_passes_random == true
        and .fresh_passes_writer == true
        and .geometric_change_non_vacuous == true
        and .geometric_premature_zero == true)),
    outcomes: .
  }
' "$OUT/reports/seed1-dev-outcome.json" \
  "$OUT/reports/seed2-dev-outcome.json" \
  >"$OUT/reports/dev-outcome.json"
jq -s -r '
  if .[0].passes and .[1].passes then "KEEP" else "REJECT" end
' "$OUT/reports/train-outcome.json" "$OUT/reports/dev-outcome.json" \
  >"$OUT/outcome.txt"
```

## Minimum test power

- two artifact seeds;
- at least 200 reference-reachable replies per seed and panel;
- a non-vacuous random-depth control in every diagnostic and fresh report;
- at least one geometric decision that differs from longest-32 per fresh seed;
- every copied geometric decision carries a selected sign-Hamming distance;
- fresh-process repeats of both fresh-dev seeds.

## Bar

For both artifact seeds:

- train geometric correct body positions are not below longest-32;
- fresh geometric correct body positions beat longest-32, longest-4, `k2>k1`,
  fixed depth one, random-32 and writer by the paired two-sigma rule;
- geometric selection changes at least one token from longest-32;
- at least 200 replies are reference-reachable;
- the random-32 control changes at least one selected token;
- every copied geometric step has a recorded distance, every fallback preserves
  the writer token, and no reply has a premature BOS;
- normalized fresh-process repeats are byte-identical.

Exact and BOS-complete counts are reported but expected to remain zero because
this arm changes only content selection.

## Stop rule

Stop on source, artifact, tokenizer, panel or report drift; a control or
geometric change that is vacuous; a missing geometric distance; fewer than 200
reachable replies; an ungated token mismatch; nondeterminism; or any need to
change serving, CUDA, float training, termination or Claude-owned source. Once
the fresh panel is read, do not change the ranking, cap, arms or bar.

## Cost

- host: AMD Ryzen 9 9950X, 16 physical cores / 32 threads, 59 GiB RAM;
- six complete-reply processes at up to 32 Rayon workers and 900 s each;
- one panel generation, one release build and one generator build;
- local CPU cost $0, no GPU pod, paid API or remote upload;
- fresh output capped at 64 MiB after gzip; the worktree target is reusable.

## Outcomes

- **KEEP** if both seeds clear the train floor and every fresh comparison and
  control; the next unit tests the winning arbitration as a trained/exported
  read.
- **REJECT** if any required comparison or control fails; do not retune this
  selection rule on the same fresh panel.
- **POWER-LIMITED** if fewer than 200 replies are reachable or the registered
  change control is vacuous.
- **INSTRUMENT FAULT** if source, artifact, tokenizer, panel or report identity
  differs, or any geometric decision lacks its distance.

## Independent GO

GO - fresh-context Codex subagent `go_geometric_review` reviewed head
`8c941102764f61538e4aa4a9beb4669d87a53901` and implementation pin
`2378b0414c674923a2c62de44865260f3b778abc`. The registered seed-2762 run is
authorized; the fresh panel and output root were absent at review.

## TEST FITNESS

TEST FITNESS: FIT - M1 #2029 geometric exact-copy arbitration
sheet: docs/harness/prep/cpu-training-pilot-copy-geometric-2026-11-05.md
reviewer: fresh-context Codex subagent fit_geometric_review_3
note: Independent sign-Hamming oracle, distinct-token nearest-distance selection, geometric replay, and random-32 domain/fallback tests pass; fresh root and seed 2762 remain absent.
date: 2026-11-05
A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone before compute.
