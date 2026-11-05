# Pre-registration: geometric writer/copy candidate union

Run ID: `cpu-training-pilot-geometric-candidate-union-2026-11-05`

## Decision

Does adding the artifact writer's top geometric candidates to exact-copy
history create enough teacher-forced supervision to train an integer ranker
that improves fresh body-token selection over the writer, fixed geometry,
shuffled-label and longest-32 controls?

This is D84 CPU-development research. It is not GPU qualification, complete
reply, language or serving evidence.

## Fixed construction

1. Start every train-panel 2711 sample at its prompt and reply start.
2. At each reference-prefix decision, form one unique-token candidate union
   from:
   - the existing writer's highest-scoring 128 non-BOS candidates under its
     unchanged cell admission and integer L1 geometry;
   - the unique exact-copy candidates at depths one through 32, excluding BOS;
   - the writer or point-fallback token for the current state when it is
     non-BOS.
3. Keep the deepest exact-copy source for a token. Keep its writer rank when
   the writer also returned it.
4. Compute eight bounded integer features per candidate:
   - bias;
   - exact-copy depth, or zero;
   - query/token sign agreement;
   - best sign agreement over eight cyclic row rotations;
   - source recency;
   - within-window occurrence frequency;
   - writer-token agreement;
   - writer-rank bucket, with the first writer candidate highest.
5. Supervise a reference-prefix position only when its next reference token is
   in the union. Feed the reference token to reach the next training state.
6. Fit one pairwise integer perceptron for eight passes with learning rate four
   and weights clipped to `[-4096, 4096]`.
7. At generated inference, start from the prompt and generated tokens only.
   Select the highest-scoring candidate, breaking exact ties by lower token id.
   BOS is excluded from every ranker candidate set.
8. Fit a second ranker after replacing each training target with a deterministic
   different candidate.
9. Compare the trained ranker with fixed geometry-only, longest-32 and writer
   controls on complete generated replies.

The new path uses integer table lookup, sign-Hamming, arithmetic and branch
logic only. It changes no serving artifact, CUDA, float inference, termination
or Claude-owned source.

## Controls

- writer token, including its point fallback;
- longest-32 exact-copy token;
- fixed `rotated_sign_agreement` scorer over the same candidate union;
- shuffled-label pairwise ranker over the same candidate union;
- depth-only weights retained in the report as a diagnostic arm.

## Data split and power

- fit on train panel 2711 reference prefixes;
- fresh `cell=dev,dev`, seed 2768, generated only after FIT and GO;
- 128 conversations and two public artifact seeds;
- at least 200 reachable replies per artifact seed and control;
- at least 2,048 supervised candidate groups and 64 distinct target tokens;
- non-vacuous changes versus writer, geometry-only, shuffled ranker and
  longest-32;
- fresh-process repeats of both artifact seeds.

## Bar

For both artifact seeds on the fresh panel, the trained candidate-union ranker
must beat writer, fixed geometry-only, shuffled-label and longest-32 on correct
body positions by the paired two-sigma rule. It must change at least one token
against each, emit zero premature BOS tokens, and repeat byte-identically after
timing fields are removed.

## Stop rule

Stop on source, artifact, tokenizer, panel, feature, label, weight or report
drift; fewer than 2,048 supervised groups; fewer than 64 target tokens; fewer
than 200 reachable replies; vacuous controls; nondeterminism; or any need to
change serving, CUDA, float inference, termination or Claude-owned source. Once
seed 2768 is read, do not retune this construction.

## Base artifacts and source

- implementation commit: `dc17b847a2ff0fc6c9ffeafe0d8ed5abdf86c61f`;
- base: `04fae9eaf`;
- generator tree:
  `869e65db6d0bb997a5f1094ab0de4efa1ac10c25`;
- seed 1 artifact SHA-256:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2 artifact SHA-256:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- tokenizer SHA-256:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- `geometric_candidate_rank.rs` SHA-256:
  `8278dcbf179f6ef6d42b2f4a91c6640a671bd5dcbad5498aef145a5e112366bd`;
- `cpu-learned-copy-read.rs` SHA-256:
  `89261efe09d05e214b3d1617379409be6e43ab066d3567370f4315ce06f241c2`;
- `fbs.rs` SHA-256:
  `6bcf3d606ec6f44ff9dba828c900132362fe6bff94f9d1543ee4bf898304080f`;
- `lib.rs` SHA-256:
  `a7cb22b6a5990f93c4d282a1d64b1082c4cb38c03b13ad4dd7d16ea6ed05b6eb`;

## Output

`local/deepseek/geometric-candidate-union/2026-11-05`. The fresh root and
seed-2768 panel are absent at registration.

## Steps

Run only after FIT and GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-geometric-candidate-union
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/geometric-candidate-union/2026-11-05
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
PANEL=$OUT/panels/dd-2768
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/geometric-candidate-union
GEN_TARGET=$SOURCE/target/deepseek/geometric-candidate-union-generator
IMPL=dc17b847a2ff0fc6c9ffeafe0d8ed5abdf86c61f
GEN_TREE=869e65db6d0bb997a5f1094ab0de4efa1ac10c25
BASE=04fae9eaf

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
  8278dcbf179f6ef6d42b2f4a91c6640a671bd5dcbad5498aef145a5e112366bd \
  "$SOURCE/crates/uor-r4-cpu-train/src/geometric_candidate_rank.rs" \
  89261efe09d05e214b3d1617379409be6e43ab066d3567370f4315ce06f241c2 \
  "$SOURCE/crates/uor-r4-cpu-train/src/bin/cpu-learned-copy-read.rs" \
  6bcf3d606ec6f44ff9dba828c900132362fe6bff94f9d1543ee4bf898304080f \
  "$SOURCE/crates/uor-r4-cpu-train/src/fbs.rs" \
  a7cb22b6a5990f93c4d282a1d64b1082c4cb38c03b13ad4dd7d16ea6ed05b6eb \
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
  cell=dev,dev conversations=128 seed=2768 protocol=2 \
  >"$OUT/panel.stdout.json"
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2768 and .dialogue_protocol_version == 2
  and .cell.value == "dev" and .cell.phrasing == "dev"
  and .conversations == 128 and .world == "m-world-v2"
  and .tokenizer.sha256 == $tokenizer_sha
' "$PANEL/manifest.json" >/dev/null

BIN=$BUILD/release/cpu-learned-copy-read
run() {
  name=$1 artifact=$2
  /usr/bin/time -v -o "$OUT/reports/$name.time.txt" \
    timeout 1800s env RAYON_NUM_THREADS=32 "$BIN" \
    "$TRAIN/tokens.u16" "$TRAIN/response_mask.u8" "$TRAIN/manifest.json" \
    "$PANEL/tokens.u16" "$PANEL/response_mask.u8" "$PANEL/manifest.json" \
    "$artifact" --cap 64 --seed 163 \
    --ranker-trajectory reference --ranker-candidates geometric-union \
    >"$OUT/reports/$name.json" 2>"$OUT/reports/$name.stderr"
}

attempt() {
  name=$1 artifact=$2
  if run "$name" "$artifact"; then
    return 0
  fi
  stderr=$OUT/reports/$name.stderr
  if rg -q 'fewer than 2048 supervised groups|fewer than 64 target tokens' "$stderr"; then
    printf 'INSUFFICIENT_TRAINING_DATA\n' >"$OUT/outcome.txt"
    exit 0
  fi
  if rg -q 'fewer than 200 reachable replies' "$stderr"; then
    printf 'POWER_LIMITED\n' >"$OUT/outcome.txt"
    exit 0
  fi
  printf 'INSTRUMENT_FAULT\n' >"$OUT/outcome.txt"
  exit 1
}

attempt seed1 "$SEED1"
attempt seed2 "$SEED2"
attempt seed1-repeat "$SEED1"
attempt seed2-repeat "$SEED2"

normalize() {
  jq -S 'walk(if type == "object"
    then del(.wall_elapsed_us, .generated_tokens_per_second, .elapsed_us)
    else . end)' "$1"
}
cmp <(normalize "$OUT/reports/seed1.json") \
  <(normalize "$OUT/reports/seed1-repeat.json") || {
    printf 'INSTRUMENT_FAULT\n' >"$OUT/outcome.txt"
    exit 1
  }
cmp <(normalize "$OUT/reports/seed2.json") \
  <(normalize "$OUT/reports/seed2-repeat.json") || {
    printf 'INSTRUMENT_FAULT\n' >"$OUT/outcome.txt"
    exit 1
  }

VALID=1
POWER_OK=1
BAR_OK=1
for name in seed1 seed2; do
  report=$OUT/reports/$name.json
  case "$name" in
    seed1) expected=a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd ;;
    seed2) expected=da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 ;;
  esac
  if ! jq -e --arg expected "$expected" '
    .schema == "uor-r4.cpu-geometric-candidate-ranker/1"
    and .inputs.artifact_sha256 == $expected
    and .inputs.train_split == "train" and .inputs.test_split == "dev"
    and .settings.cap == 64 and .settings.seed == 163
    and .settings.candidate_set == "writer_top128_nonbos_exact32_fallback"
    and .training.trajectory == "reference_prefix"
    and .training.supervised_groups >= 2048
    and .training.target_values >= 64
    and .training.shuffled_target_changes > 0
    and .candidate_ranker.depth_only_weights == [0,1,0,0,0,0,0,0]
    and .test.trained.premature_bos == 0
    and .test.shuffled.premature_bos == 0
    and .test.geometry_only.premature_bos == 0
    and .test.longest_32.premature_bos == 0
    and .test.writer.premature_bos == 0
    and .test.changes.writer.non_vacuous == true
    and .test.changes.geometry_only.non_vacuous == true
    and .test.changes.shuffled.non_vacuous == true
    and .test.changes.longest_32.non_vacuous == true
    and (.test.signatures.trained.reply_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.trained.decision_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.shuffled.reply_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.shuffled.decision_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.geometry_only.reply_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.geometry_only.decision_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.longest_32.reply_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.longest_32.decision_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.writer.reply_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.writer.decision_sha256 | test("^[0-9a-f]{64}$"))
  ' "$report" >/dev/null; then
    printf '%s instrument-or-control-fault\n' "$name" >>"$OUT/instrument-fault.txt"
    VALID=0
  fi
  if ! jq -e '
    .test.trained.reference_reachable >= 200
    and .test.shuffled.reference_reachable >= 200
    and .test.geometry_only.reference_reachable >= 200
    and .test.longest_32.reference_reachable >= 200
    and .test.writer.reference_reachable >= 200
  ' "$report" >/dev/null; then
    printf '%s reachable-below-200\n' "$name" >>"$OUT/power-limited.txt"
    POWER_OK=0
  fi
  if ! jq -e '
    .test.comparisons.writer.correct_body_positions.passes == true
    and .test.comparisons.geometry_only.correct_body_positions.passes == true
    and .test.comparisons.shuffled.correct_body_positions.passes == true
    and .test.comparisons.longest_32.correct_body_positions.passes == true
  ' "$report" >/dev/null; then
    BAR_OK=0
  fi
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
if test "$BAR_OK" -eq 1; then
  printf 'KEEP\n' >"$OUT/outcome.txt"
else
  printf 'REJECT\n' >"$OUT/outcome.txt"
fi
```

## Minimum test power

- two public artifact seeds;
- at least 2,048 supervised candidate groups from train panel 2711;
- at least 64 distinct target tokens;
- at least 200 reachable replies per artifact seed and control;
- at least one non-vacuous change against each registered control;
- fresh-process repeats of both artifact seeds.

## Cost

- host: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads, 59 GiB RAM;
- one release build, one generator build and one fresh panel;
- four driver processes at up to 32 Rayon workers and 1,800 s each;
- at most 128 non-BOS writer candidates, 32 exact-copy candidates and one
  fallback per state;
- local CPU cost $0, no GPU pod, paid API or remote upload;
- fresh output capped at 64 MiB after gzip.

## Outcomes

- **KEEP** if both artifact seeds beat writer, fixed geometry-only,
  shuffled-label and longest-32 by the paired rule and preserve all controls.
- **REJECT** if any required comparison or control fails; do not retune this
  construction on seed 2768.
- **INSUFFICIENT_TRAINING_DATA** if fewer than 2,048 candidate groups or 64
  target tokens remain available.
- **POWER_LIMITED** if fewer than 200 replies are reachable.
- **INSTRUMENT_FAULT** if fitting, features, source, inputs, controls or report
  identity fails closed.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 geometric writer/copy candidate union
sheet: `docs/harness/prep/cpu-training-pilot-geometric-candidate-union-2026-11-05.md`
implementation pin: `dc17b847a2ff0fc6c9ffeafe0d8ed5abdf86c61f`
reviewed head: `7a326d4162e952108721dd716e38c51790271ab3`
reviewer: deepseek fresh-context subagent `geometric_candidate_fit_review_2`
note: 128 non-BOS writer candidates, exact depths through 32 and deepest source,
  fallback, eight integer features, reference-free inference, all controls,
  exact floors and five-arm replay hashes were verified; source and input pins
  match and seed 2768 remained absent.
date: 2026-11-05

A FIT verdict clears compute for the configuration in this sheet only (D22).

## Pre-run GO

GO - reviewer `deepseek fresh-context subagent geometric_candidate_pre_run_go_2`,
2026-11-05. HEAD was clean at
`2183f2f69958690ece84da309de6cf21d7b1c092`; `prep-check` reported the sheet
complete FIT. The implementation source pin, generator, tokenizer, two
artifacts, train panel hashes, registered commands and jq predicates all
matched or parsed; seed 2768, its panel and the output root remained absent.
The four registered fresh-process runs, fail-closed classifications, local
resource plan and lane isolation were checked read-only. No seed-2768
generation, build or driver run occurred.

## Outcome

**INSUFFICIENT_TRAINING_DATA.** Artifact seed 1 passed the floors with 2,442
supervised groups and 332 target tokens. Artifact seed 2 stopped with 1,847
groups against the 2,048-group floor, before fit, reply evaluation or any
repeat. The raw result is preserved at
`local/deepseek/geometric-candidate-union/2026-11-05`.

The registered shell automation emitted `INSTRUMENT_FAULT` because its stop
regex expected `fewer than 2048 supervised groups` while the preserved stderr
reads `has 1847 supervised groups, fewer than 2048`. The post-run correction
applies the registered stop rule to that preserved output. Seed 2768 was not
rerun, no construction was retuned, and no two-seed bar is claimed.
