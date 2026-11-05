# Pre-registration: exact-copy candidate lookahead

Run ID: `cpu-training-pilot-copy-lookahead-2026-11-05`

## Decision

Can one-step lookahead through the exact model state distinguish exact-copy
candidates better than the rejected no-lookahead learned read? For each unique
depth-1..4 candidate, this unit advances a cloned exact `CpuFbsForward`, packs
the resulting 2,048-bit modulo-256 query state, and uses the train-fitted
nearest-example readout to score that future state.

This is D84 CPU development research, not GPU qualification, complete-reply,
language or serving evidence.

## Fixed construction

The readout is fitted independently for each artifact seed from generated
longest-32 states on train panel 2711. Training keeps the existing 90%
leave-one-out threshold and deterministically retains at most 1,024 generated
rows.

At a lookahead decision:

1. Collect unique exact-copy candidate tokens at depths one through four.
2. Clone the current forward, feed the candidate, and obtain its next writer
   query.
3. Read the nearest train example for that candidate-conditioned next state.
4. Rank by next-state Hamming distance, deeper original copy depth and lower
   token id.
5. If the selected next state is outside the learned threshold, fall back to
   longest-32.
6. Repeat with a deterministic target-shuffled readout.

No candidate outside the exact-copy set can be selected. Model state advances
only after selection; lookahead clones are discarded.

## Controls

- no-lookahead learned and shuffled reads from deepseek-167;
- lookahead with the shuffled target table;
- longest-32, longest-4, `k2>k1`, fixed depth one, random-32 and writer;
- static token-row sign-Hamming arbitration.

## Data split and power

- two public artifact seeds;
- train/fit on panel 2711;
- fresh `cell=dev,dev`, seed 2764, generated only after FIT and GO;
- at least 200 reachable replies per seed and panel;
- at least one lookahead arbitration per fresh seed;
- non-vacuous lookahead-versus-longest, lookahead-versus-no-lookahead,
  shuffled-lookahead and random-32 changes;
- fresh-process repeats of both seeds.

## Bar

For both artifact seeds on the fresh panel, lookahead correct body positions
must beat all controls by the paired two-sigma rule. Lookahead must change at
least one token from longest-32 and from the no-lookahead learned read,
arbitrate at least one candidate, preserve zero premature BOS and ungated
mismatches, and repeat byte-identically after timing fields are removed.

Exact and BOS-complete counts are reported but expected to remain zero because
stopping is unchanged.

## Stop rule

Stop on source, artifact, tokenizer, panel, readout or report drift; failed fit
or lookahead coverage; fewer than 200 reachable replies; vacuous controls;
nondeterminism; or any need to change serving, CUDA, float training,
termination or Claude-owned source. Once seed 2764 is read, do not change the
depth cap, rollout, threshold, controls or bar.

## Base artifacts and source

- implementation commit: `4b05143cb`;
- base: `97c4c5a4a`;
- generator tree:
  `869e65db6d0bb997a5f1094ab0de4efa1ac10c25`;
- seed 1 artifact SHA-256:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2 artifact SHA-256:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- tokenizer SHA-256:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- `learned_copy_reply.rs` SHA-256:
  `dd611569937c21eac5afb1975151b4bad73ceb11a21a9abfcd6b04cd5a47ab91`;
- `cpu-learned-copy-read.rs` SHA-256:
  `5af29377adec20eead10387c07d5377e046111e72dfe04786909c9c64f687660`.

## Output

`local/deepseek/copy-lookahead/2026-11-05`. The fresh root and seed-2764 panel
are absent at registration.

## Steps

Run only after FIT and GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-lookahead
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-lookahead/2026-11-05
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
PANEL=$OUT/panels/dd-2764
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/copy-lookahead
GEN_TARGET=$SOURCE/target/deepseek/copy-lookahead-generator
IMPL=4b05143cb
GEN_TREE=869e65db6d0bb997a5f1094ab0de4efa1ac10c25
BASE=97c4c5a4a

test ! -e "$OUT"
test ! -e "$PANEL"
git -C "$SOURCE" diff --quiet "$IMPL" -- crates/uor-r4-cpu-train
test "$(git -C "$SOURCE" rev-parse "$BASE":docs/labs/one-model-end-to-end/instrument)" = "$GEN_TREE"
printf '%s  %s\n' \
  a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 "$TOKENIZER" \
  a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$SEED1" \
  da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$SEED2" \
  140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91 "$TRAIN/tokens.u16" \
  d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad "$TRAIN/response_mask.u8" \
  d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5 "$TRAIN/manifest.json" |
  sha256sum -c -
printf '%s  %s\n' \
  dd611569937c21eac5afb1975151b4bad73ceb11a21a9abfcd6b04cd5a47ab91 \
  "$SOURCE/crates/uor-r4-cpu-train/src/learned_copy_reply.rs" \
  5af29377adec20eead10387c07d5377e046111e72dfe04786909c9c64f687660 \
  "$SOURCE/crates/uor-r4-cpu-train/src/bin/cpu-learned-copy-read.rs" |
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
  cargo build --locked --release -p uor-r4-cpu-train --bin cpu-learned-copy-read
/usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GEN_TARGET" \
  cargo build --locked \
  --manifest-path "$SOURCE/docs/labs/one-model-end-to-end/instrument/Cargo.toml" \
  --features panel-writer --bin heldout-panel
GEN=$GEN_TARGET/debug/heldout-panel

/usr/bin/time -v -o "$OUT/panel.time.txt" \
  timeout 900s "$GEN" out="$PANEL" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2764 protocol=2 \
  >"$OUT/panel.stdout.json"
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2764 and .dialogue_protocol_version == 2
  and .cell.value == "dev" and .cell.phrasing == "dev"
  and .conversations == 128 and .world == "m-world-v2"
  and .tokenizer.sha256 == $tokenizer_sha
' "$PANEL/manifest.json" >/dev/null
TRAIN_TOKENS_SHA=$(sha256sum "$TRAIN/tokens.u16" | cut -d ' ' -f1)
TRAIN_MASK_SHA=$(sha256sum "$TRAIN/response_mask.u8" | cut -d ' ' -f1)
TRAIN_MANIFEST_SHA=$(sha256sum "$TRAIN/manifest.json" | cut -d ' ' -f1)
PANEL_TOKENS_SHA=$(sha256sum "$PANEL/tokens.u16" | cut -d ' ' -f1)
PANEL_MASK_SHA=$(sha256sum "$PANEL/response_mask.u8" | cut -d ' ' -f1)
PANEL_MANIFEST_SHA=$(sha256sum "$PANEL/manifest.json" | cut -d ' ' -f1)

BIN=$BUILD/release/cpu-learned-copy-read
run() {
  name=$1 artifact=$2
  /usr/bin/time -v -o "$OUT/reports/$name.time.txt" \
    timeout 1800s env RAYON_NUM_THREADS=32 "$BIN" \
    "$TRAIN/tokens.u16" "$TRAIN/response_mask.u8" "$TRAIN/manifest.json" \
    "$PANEL/tokens.u16" "$PANEL/response_mask.u8" "$PANEL/manifest.json" \
    "$artifact" --cap 64 --seed 163 --allow-underpowered \
    >"$OUT/reports/$name.json"
}
run seed1 "$SEED1"
run seed2 "$SEED2"
run seed1-repeat "$SEED1"
run seed2-repeat "$SEED2"

normalize() {
  jq -S 'walk(if type == "object"
    then del(.wall_elapsed_us, .generated_tokens_per_second, .elapsed_us)
    else . end)' "$1"
}
cmp <(normalize "$OUT/reports/seed1.json") \
  <(normalize "$OUT/reports/seed1-repeat.json")
cmp <(normalize "$OUT/reports/seed2.json") \
  <(normalize "$OUT/reports/seed2-repeat.json")

VALID=1
POWER_OK=1
for name in seed1 seed2; do
  report=$OUT/reports/$name.json
  case "$name" in
    seed1) expected=a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd ;;
    seed2) expected=da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 ;;
  esac
  if ! jq -e --arg expected "$expected" \
    --arg train_tokens "$TRAIN_TOKENS_SHA" \
    --arg train_mask "$TRAIN_MASK_SHA" \
    --arg train_manifest "$TRAIN_MANIFEST_SHA" \
    --arg test_tokens "$PANEL_TOKENS_SHA" \
    --arg test_mask "$PANEL_MASK_SHA" \
    --arg test_manifest "$PANEL_MANIFEST_SHA" \
    --arg tokenizer a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
    .schema == "uor-r4.cpu-learned-copy-read/1"
    and .inputs.artifact_sha256 == $expected
    and .inputs.train_tokens_sha256 == $train_tokens
    and .inputs.train_mask_sha256 == $train_mask
    and .inputs.train_manifest_sha256 == $train_manifest
    and .inputs.test_tokens_sha256 == $test_tokens
    and .inputs.test_mask_sha256 == $test_mask
    and .inputs.test_manifest_sha256 == $test_manifest
    and .inputs.tokenizer_sha256 == $tokenizer
    and .inputs.train_split == "train" and .inputs.test_split == "dev"
    and .settings.cap == 64 and .settings.seed == 163
    and .settings.signature_mode == "modulo_256_code"
    and .training.generated_rows >= 640
    and .training.retained_rows <= 1024
    and .training.target_values >= 16
    and (.training.covered_rows * 10 >= .training.retained_rows * 9)
    and .training.mapping_sha256 != .training.shuffled_mapping_sha256
    and .test.changes.lookahead_from_longest_32.non_vacuous == true
    and .test.changes.lookahead_from_no_lookahead.non_vacuous == true
    and .test.changes.shuffled_lookahead_from_lookahead.non_vacuous == true
    and .test.changes.random_32_from_longest_32.non_vacuous == true
    and .test.lookahead.reading.learned_arbitrations > 0
    and .test.lookahead.reading.lookahead_decisions > 0
    and .test.lookahead.reading.premature_bos == 0
    and .test.lookahead_shuffled.reading.premature_bos == 0
    and all(.test.lookahead.steps[];
      (if .lookahead_evaluated
        then
          .lookahead_distance != null
          and (if .learned_arbitrated
            then (.selected_depth != null and .selected_depth <= 4)
            else .selected_token == .longest_token
            end)
        else
          .lookahead_distance == null
          and .selected_token == .longest_token
        end)
      and (if .used_copy then .selected_depth != null
        else .selected_token == .writer_token
        end))
  ' "$report" >/dev/null; then
    printf '%s instrument-or-control-fault\n' "$name" >>"$OUT/instrument-fault.txt"
    VALID=0
  fi
  if ! jq -e '.test.examples.reachable >= 200' "$report" >/dev/null; then
    printf '%s reachable-below-200\n' "$name" >>"$OUT/power-limited.txt"
    POWER_OK=0
  fi
  jq '{
    lookahead_body_positions: .test.lookahead.reading.correct_body_positions,
    no_lookahead_body_positions: .test.learned.reading.correct_body_positions,
    longest_body_positions: .test.controls.longest_32_to_1.reading.correct_body_positions,
    lookahead_arbitrations: .test.lookahead.reading.learned_arbitrations,
    compares: (all(.test.comparisons[]; .correct_body_positions.passes == true)),
    all_control_reachable: (all(.test.controls[]; .reading.reference_reachable >= 200)),
    exact_reference: .test.lookahead.reading.exact_reference,
    bos_complete: .test.lookahead.reading.bos_complete
  }' "$report" >"$OUT/reports/$name-outcome.json"
done

gzip -9 "$OUT"/reports/seed1.json "$OUT"/reports/seed2.json \
  "$OUT"/reports/seed1-repeat.json "$OUT"/reports/seed2-repeat.json
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
  {passes: (all(.[]; .compares == true and .all_control_reachable == true)), outcomes: .}
' "$OUT/reports/seed1-outcome.json" "$OUT/reports/seed2-outcome.json" \
  >"$OUT/reports/comparison-outcome.json"
jq -s -r 'if .[0].passes then "KEEP" else "REJECT" end' \
  "$OUT/reports/comparison-outcome.json" >"$OUT/outcome.txt"
```

## Minimum test power

- two artifact seeds;
- at least 640 generated training rows, 16 target values and 90% fitted
  leave-one-out coverage;
- at most 1,024 retained rows;
- at least 200 reachable replies per seed and panel;
- at least one lookahead arbitration per fresh seed;
- non-vacuous lookahead-versus-longest, lookahead-versus-no-lookahead,
  shuffled-lookahead and random-32 changes;
- fresh-process repeats of both artifact seeds.

## Bar

For both artifact seeds on the fresh panel, lookahead must beat all eleven
controls by the paired two-sigma rule on correct body positions. It must change
at least one token from both longest-32 and the no-lookahead learned read,
arbitrate at least one candidate, preserve zero premature BOS and ungated
mismatches, and repeat byte-identically after timing fields are removed.

## Cost

- host: AMD Ryzen 9 9950X, 16 physical cores / 32 threads, 59 GiB RAM;
- one release build, one generator build and one fresh panel;
- four driver processes at up to 32 Rayon workers and 1,800 s each;
- candidate rollouts are capped at four unique depths per decision;
- local CPU cost $0, no GPU pod, paid API or remote upload;
- fresh output capped at 64 MiB after gzip.

## Outcomes

- **KEEP** if both artifact seeds clear every required comparison and control.
- **REJECT** if any required comparison or control fails; do not retune this
  lookahead on seed 2764.
- **POWER-LIMITED** if fewer than 200 replies are reachable.
- **INSTRUMENT FAULT** if fit, source, input, lookahead or report identity
  fails closed.

## Independent GO

GO - fresh-context Codex subagent `go_copy_lookahead_2` reviewed head
`12e1a3911efd01aba909ab242791e2f89762d6f5` and implementation pin
`4b05143cbcd9f9a7e8959fb1610acfc0bf14c841`. The registered seed-2764 run is
authorized; the fresh panel and output root were absent at review.

## TEST FITNESS

TEST FITNESS: FIT - M1 #2029 candidate-conditioned exact model lookahead
sheet: docs/harness/prep/cpu-training-pilot-copy-lookahead-2026-11-05.md
reviewer: fresh-context Codex subagent fit_copy_lookahead_2
note: Fresh FIT after correcting empty-candidate fallback checks; exact cloned-forward lookahead, candidate-only semantics, no-lookahead/control comparisons, identity checks and deterministic replay verified; seed 2764 remains absent.
date: 2026-11-05
A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone before compute.
