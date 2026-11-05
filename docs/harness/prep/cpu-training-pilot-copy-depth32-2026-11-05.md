# Pre-registration: longest-32 exact copy depth

Run ID: `cpu-training-pilot-copy-depth32-2026-11-05`

## Decision

Do exact suffix matches deeper than four improve generated reply body
selection? Deepseek-149 selected longest-4 from a depth-one-through-four
search; this tests the untried 8-32 range while preserving longest-4 as a
nested control.

This is a D84 CPU development screen. It is not GPU qualification, complete
reply, language or serving evidence.

## Prior evidence and change

Deepseek-153's longest-4 complete-reply arm preserved 409/4,661 train and
556/4,614 dev correct body positions, beat its shallower controls, and
terminated zero replies because copy can never emit BOS. No prior unit
searched exact copy depths above four in complete replies.

The candidate arm searches depths 1 through 32, chooses the deepest available
exact suffix, and falls back through shorter matches. The existing decoder,
writer fallback, memory, stopping contract and artifact are unchanged.

Controls are longest-4, `k2>k1`, fixed depth 1, random available depth in
1..32, and the unmodified writer. The legacy random-depth arm keeps its
original 1..4 domain; `random_available_depth_32` is a separate control for
this screen. The same artifact seeds and generation cap are used.

## Base artifact

- seed 1: `dual-c131-seed-1-raw_both.uorlut`, SHA-256
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2: `dual-c131-seed-2-raw_both.uorlut`, SHA-256
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- implementation pin `fec31bf5ca9b`, source hashes
  `f3e8d68fe727469428fa75a2c46d20dfe910b922437faf48ef5ad7672efb1f3f`
  (`copy_key_reply.rs`) and
  `7c21b5cec0b5b9abb39e9b34942aba4ef05dad5a4f0a5f9ac8dadae140a6e9d9`
  (`cpu-copy-key-reply.rs`);
- panel generator tree `869e65db6d0bb997a5f1094ab0de4efa1ac10c25` at base
  `7fe55fdbf`;
- tokenizer SHA-256
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`.

## Data split

- Train diagnostic: existing panel 2711 and sealed train reports.
- Fresh decision: generate one new `cell=dev,dev` panel, 128 conversations,
  protocol 2, seed 2752, after FIT and GO.
- Output root: `local/deepseek/copy-depth32/2026-11-05`; it is absent now.
- The new panel manifest is validated for seed, split, protocol, conversation
  count, world and tokenizer before generation.

## Steps

Use a fresh output root after GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-depth32
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-depth32/2026-11-05
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2
PANEL=$OUT/panels/dd-2752
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/copy-depth32
GEN_TARGET=$SOURCE/target/deepseek/copy-depth32-generator
IMPL=fec31bf5ca9b
GEN_TREE=869e65db6d0bb997a5f1094ab0de4efa1ac10c25
BASE=7fe55fdbf

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
  cell=dev,dev conversations=128 seed=2752 protocol=2 \
  >"$OUT/panel.stdout.json"
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2752 and .dialogue_protocol_version == 2
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
  if ! jq -e '.settings.maximum_copy_depth == 32' "$report" >/dev/null; then
    printf '%s maximum-depth-mismatch\n' "$name" >>"$OUT/instrument-fault.txt"
    VALID=0
  fi
  if ! jq -e --arg expected "$expected" \
    '.inputs.artifact_sha256 == $expected' "$report" >/dev/null; then
    printf '%s artifact-hash-mismatch\n' "$name" >>"$OUT/instrument-fault.txt"
    VALID=0
  fi
  if ! jq -e '.random_nonvacuity_from_longest_32.non_vacuous == true' \
    "$report" >/dev/null; then
    printf '%s random-control-vacuous\n' "$name" >>"$OUT/instrument-fault.txt"
    VALID=0
  fi
  if ! jq -e 'all(.arms[].steps[];
    if .used_copy then true else .selected_token == .writer_token end)' \
    "$report" >/dev/null; then
    printf '%s ungated-token-mismatch\n' "$name" >>"$OUT/instrument-fault.txt"
    VALID=0
  fi
done
for seed in seed1 seed2; do
  if ! jq -e '[.arms.longest_32_to_1.reading.selected_depth_counts[4:][]]
    | add > 0' "$OUT/reports/$seed-dd.json" >/dev/null; then
    printf '%s no-depth-above-four\n' "$seed" >>"$OUT/power-limited.txt"
    POWER_OK=0
  fi
  jq '{
    train_not_below_longest_4: (
      .arms.longest_32_to_1.reading.correct_body_positions
      >= .arms.longest_4_to_1.reading.correct_body_positions
    ),
    candidate_premature_zero:
      .arms.longest_32_to_1.reading.premature_bos == 0,
    candidate_body_positions:
      .arms.longest_32_to_1.reading.correct_body_positions,
    longest_4_body_positions:
      .arms.longest_4_to_1.reading.correct_body_positions
  }' "$OUT/reports/$seed-tt.json" >"$OUT/reports/$seed-train-outcome.json"
  jq '{
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
    fresh_premature_zero:
      .arms.longest_32_to_1.reading.premature_bos == 0,
    exact_reference: .arms.longest_32_to_1.reading.exact_reference,
    bos_complete: .arms.longest_32_to_1.reading.bos_complete
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
    passes:
      (all(.[]; .train_not_below_longest_4 == true)),
    outcomes: .
  }
' "$OUT/reports/seed1-train-outcome.json" \
  "$OUT/reports/seed2-train-outcome.json" \
  >"$OUT/reports/train-outcome.json"
jq -s '
  {
    passes:
      (all(.[];
        .fresh_passes_longest_4 == true
        and .fresh_passes_k2 == true
        and .fresh_passes_depth_1 == true
        and .fresh_passes_random == true
        and .fresh_passes_writer == true
        and .fresh_premature_zero == true)),
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
- a non-vacuous random-depth control for each of the four primary reports;
- at least one selected depth greater than four on each fresh-dev seed;
- fresh-process repeats of both fresh-dev seeds.

## Bar

For both artifact seeds:

- train diagnostic: longest-32 correct body positions are not below
  longest-4;
- fresh dev: longest-32 beats longest-4, `k2>k1`, fixed depth 1, random depth
  and writer on correct reference body positions by the paired two-sigma rule;
- fresh-dev reachable replies are at least 200;
- the random control changes at least one candidate decision;
- both candidate runs have zero premature BOS stops and no ungated token
  mismatch;
- normalized fresh-process repeats are byte-identical.

Exact and BOS-complete reply counts are reported but are expected to remain
zero because this arm changes only content selection.

## Stop rule

Stop on source, artifact, tokenizer, panel or report drift; any cell below 200
reachable replies; a vacuous random control; no selected depth above four; an
ungated token mismatch; nondeterminism; or a need to change serving, CUDA,
float training or Claude-owned source. Once the fresh panel is read, do not
change the depth, cap, arms or bar.

## Cost

- host: AMD Ryzen 9 9950X, 16 physical cores / 32 threads, 59 GiB RAM;
- six complete-reply processes at up to 32 workers and 900 s each;
- one panel generation, one release build and one generator build;
- local CPU cost $0, no GPU pod, paid API or remote upload;
- fresh output capped at 64 MiB after gzip; target reuses the worktree target.

## Outcomes

- **KEEP** if the deeper arm clears every fresh-dev comparison on both seeds;
  the next unit tests the selected depth as a trained/exported read.
- **REJECT** if any required comparison or control fails; close this
  exact-depth extension only.
- **POWER-LIMITED** if fewer than 200 replies or no depth-above-four match is
  available.

## TEST FITNESS

TEST FITNESS: FIT - fresh-context review at `89ef35587`; legacy five-arm
consumers preserved, new seven-arm evaluation separated, fresh panel absent,
input/source pins and fail-closed power/determinism checks verified.
