# Pre-registration: pairwise exact-copy candidate ranker

Run ID: `cpu-training-pilot-copy-ranker-2026-11-05`

## Decision

Can a deterministic integer pairwise ranker improve exact-copy arbitration
using candidate-specific geometric and occurrence features? The unit trains on
the exact served query and exact-copy candidate set and never emits a token
outside that set.

This is D84 CPU-development research. It is not GPU qualification, complete
reply, language or serving evidence.

## Fixed construction

1. Generate train-panel 2711 trajectories with longest-32 content selection.
2. At each generated decision, collect unique exact-copy candidates at depths
   one through eight, excluding BOS.
3. Supervise a group only when at least two unique candidates exist and the
   reference next token is present. Retain the deepest occurrence of each
   candidate token.
4. Compute seven bounded integer features per candidate:
   - bias;
   - source depth;
   - query/token sign agreement;
   - best sign agreement over eight cyclic rotations of the token row;
   - source recency;
   - within-window occurrence frequency;
   - agreement with the writer token.
5. Fit a pairwise integer perceptron for eight passes with learning rate four
   and weights clipped to `[-4096, 4096]`.
6. Select the highest-scoring candidate, breaking exact ties by lower token id.
7. Repeat fitting with targets rotated to a different candidate in each group.
8. Evaluate fixed depth-only, shuffled-label ranker, longest-32,
   learned-nearest, geometric, random-depth, fixed-depth and writer controls.

The inference path uses integer addition/multiplication only. It changes no
serving artifact, CUDA, float inference, termination or Claude-owned source.

## Controls

- shuffled-label ranker trained by the same feature and perceptron rule;
- fixed depth-only ranker;
- no-lookahead learned and lookahead readouts;
- longest-32 and longest-4 exact copy;
- `k2>k1`, fixed depth one, random-depth-4, random-depth-32 and writer;
- geometric token-row sign-Hamming arbitration.

## Data split and power

- fit on train panel 2711;
- fresh `cell=dev,dev`, seed 2766, generated only after FIT and GO;
- 128 conversations and two public artifact seeds;
- at least 200 reachable replies per seed and at least 640 supervised groups;
- non-vacuous ranker changes versus longest-32, shuffled ranker and depth-only;
- fresh-process repeats of both artifact seeds.

## Bar

For both artifact seeds on the fresh panel, the trained ranker must beat
longest-32, the shuffled-label ranker and the fixed depth-only ranker on correct
body positions by the paired two-sigma rule. It must change at least one token
against each, preserve zero premature BOS and ungated mismatches, and repeat
byte-identically after timing fields are removed.

## Stop rule

Stop on source, artifact, tokenizer, panel, feature, label, weight or report
drift; fewer than 640 supervised groups; fewer than 200 reachable replies;
vacuous controls; nondeterminism; or any need to change serving, CUDA, float
inference, termination or Claude-owned source. Once seed 2766 is read, do not
retune features, weights, learning rule, controls or bar.

## Base artifacts and source

- implementation commit: `f54ab142c30330cc9aa0212b715b5010fd57ef26`;
- base: `1f56a8e7b`;
- generator tree:
  `869e65db6d0bb997a5f1094ab0de4efa1ac10c25`;
- seed 1 artifact SHA-256:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2 artifact SHA-256:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- tokenizer SHA-256:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- `copy_candidate_rank.rs` SHA-256:
  `8b9f266d72cc77877d63dcafc15a1e5017f642fe12540f856ece39bb7a676ee0`;
- `cpu-learned-copy-read.rs` SHA-256:
  `36a7d5cbd38e8064dc6d868c4a64c14b441e94005b599e747eb93a2e165fc6e0`;
- `lib.rs` SHA-256:
  `91e64959d146cc054e62ce8c187ba935dab4e7e38795d3a77a551b831253c351`.

## Output

`local/deepseek/copy-ranker/2026-11-05`. The fresh root and seed-2766 panel
are absent at registration.

## Steps

Run only after FIT and GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-ranker
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-ranker/2026-11-05
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
PANEL=$OUT/panels/dd-2766
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/copy-ranker
GEN_TARGET=$SOURCE/target/deepseek/copy-ranker-generator
IMPL=f54ab142c30330cc9aa0212b715b5010fd57ef26
GEN_TREE=869e65db6d0bb997a5f1094ab0de4efa1ac10c25
BASE=1f56a8e7b

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
  8b9f266d72cc77877d63dcafc15a1e5017f642fe12540f856ece39bb7a676ee0 \
  "$SOURCE/crates/uor-r4-cpu-train/src/copy_candidate_rank.rs" \
  36a7d5cbd38e8064dc6d868c4a64c14b441e94005b599e747eb93a2e165fc6e0 \
  "$SOURCE/crates/uor-r4-cpu-train/src/bin/cpu-learned-copy-read.rs" \
  91e64959d146cc054e62ce8c187ba935dab4e7e38795d3a77a551b831253c351 \
  "$SOURCE/crates/uor-r4-cpu-train/src/lib.rs" |
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
  cell=dev,dev conversations=128 seed=2766 protocol=2 \
  >"$OUT/panel.stdout.json"
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2766 and .dialogue_protocol_version == 2
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
    and .candidate_ranker.features ==
      ["bias","depth","sign_agreement","rotated_sign_agreement","recency","frequency","writer_match"]
    and .candidate_ranker.epochs == 8
    and .candidate_ranker.max_depth == 8
    and .candidate_ranker.supervised_groups >= 640
    and .candidate_ranker.shuffled_target_changes > 0
    and .test.ranked.reading.reference_reachable >= 200
    and .test.ranked.reading.copy_decisions > 0
    and .test.ranked.reading.premature_bos == 0
    and all(.test.controls[]; .reading.reference_reachable >= 200)
    and .test.changes.ranker_from_longest_32.non_vacuous == true
    and .test.changes.ranker_from_shuffled_ranker.non_vacuous == true
    and .test.changes.ranker_from_depth_only.non_vacuous == true
    and all(.test.ranked.steps[];
      (if .used_copy
        then (.candidate_count > 0 and .selected_depth != null
          and .selected_depth <= 8 and .selected_score != null)
        else (.candidate_count == 0 and .selected_token == .longest_token)
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
    ranker_body_positions: .test.ranked.reading.correct_body_positions,
    longest_body_positions: .test.controls.longest_32_to_1.reading.correct_body_positions,
    shuffled_ranker_body_positions: .test.ranked_shuffled.reading.correct_body_positions,
    depth_only_body_positions: .test.ranked_depth_only.reading.correct_body_positions,
    supervised_groups: .candidate_ranker.supervised_groups,
    compares:
      (.test.ranker_comparisons.longest_32_to_1.correct_body_positions.passes == true
       and .test.ranker_comparisons.shuffled_ranker.correct_body_positions.passes == true
       and .test.ranker_comparisons.depth_only_ranker.correct_body_positions.passes == true),
    all_control_reachable:
      (all(.test.controls[]; .reading.reference_reachable >= 200)),
    exact_reference: .test.ranked.reading.exact_reference,
    bos_complete: .test.ranked.reading.bos_complete
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
- at least 640 supervised candidate groups from train panel 2711;
- at least 200 reachable replies per artifact seed;
- at least one copy decision and one non-vacuous change against longest,
  shuffled and depth-only;
- fresh-process repeats of both artifact seeds.

## Cost

- host: AMD Ryzen 9 9950X, 16 physical cores / 32 threads, 59 GiB RAM;
- one release build, one generator build and one fresh panel;
- four driver processes at up to 32 Rayon workers and 1,800 s each;
- exact-copy candidates are capped at eight, with seven integer features each;
- local CPU cost $0, no GPU pod, paid API or remote upload;
- fresh output capped at 64 MiB after gzip.

## Outcomes

- **KEEP** if both artifact seeds beat longest-32, shuffled and depth-only by
  the paired rule and preserve all controls.
- **REJECT** if any required comparison or control fails; do not retune this
  ranker on seed 2766.
- **POWER-LIMITED** if fewer than 200 replies are reachable.
- **INSTRUMENT FAULT** if fitting, features, source, inputs, controls or report
  identity fails closed.

## TEST FITNESS

TEST FITNESS: FIT - M1 #2029 pairwise exact-copy candidate ranker
sheet: docs/harness/prep/cpu-training-pilot-copy-ranker-2026-11-05.md
reviewer: fresh-context Codex subagent fit_copy_ranker
note: Independent review at `fa74475e121082acff6297f68cbea70efcb23cc2` verified candidate-only semantics, deepest duplicate source, all seven bounded features, pairwise integer updates, shuffled and depth-only controls, train/fresh separation, fail-closed gates, determinism and source pins.
date: 2026-11-05
A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone before compute.

## Independent GO

GO - fresh-context Codex subagent `go_copy_ranker` reviewed head
`05d639782cb04a14a1c8fe6f752f053c59a93032`. The registered seed-2766 run is
authorized; the fresh panel and output root were absent at review. The
implementation pin, source and input hashes, candidate-only feature path,
shuffled and depth-only controls, power floors, deterministic replay and
fail-closed gates passed. No experiment was executed during review.
