# Pre-registration: expanded-cell longest-32-anchored candidate ranker

Run ID: `cpu-training-pilot-expanded-candidate-anchor-2026-11-05`

## Decision

Can a CPU-only 12-cell geometric candidate source supply enough supervision
for a longest-32-anchored integer correction ranker to beat longest-32 itself,
the writer, fixed geometry, shuffled targets and an unanchored ranker on a
fresh prompt panel?

This is D84 CPU-development research. It is not GPU qualification, complete
reply, language or serving evidence.

## Why 12 cells

On the closed train panel 2711, the served eight-cell writer exposes all 128
candidate tokens available from its cells:

| Cells | Writer cap | Seed-1 groups | Seed-1 baseline wrong | Seed-2 groups | Seed-2 baseline wrong |
|---:|---:|---:|---:|---:|---:|
| 8 | 128 | 2,442 | 554 | 1,847 | 263 |
| 12 | 192 | 2,666 | 778 | 2,043 | 459 |
| 16 | 256 | 2,753 | 865 | 2,219 | 635 |
| 24 | 384 | 3,005 | 1,117 | 2,444 | 860 |
| 32 | 512 | 3,181 | 1,293 | 2,732 | 1,148 |

Both artifacts have at least 330 distinct target tokens at 12 cells. Twelve is the
smallest setting that clears 2,048 target-present groups, 64 target tokens and
256 baseline-wrong groups on both artifacts. The 192-candidate value is a
non-BOS cap: observed writer-candidate counts are 191-192 per supervised group.

## Fixed construction

1. Candidate cells are selected by the writer's existing two eight-coordinate
   ports, their integer L1 scores against the 16 SpiralCore roots, and the
   existing lower-cell tie order. The CPU development path reads 12 cells; the
   served writer remains at 8.
2. At each reference-prefix decision, form one unique-token union from:
   - up to 192 non-BOS candidates from those 12 cells, ranked by the existing
     integer L1 token score; BOS is excluded before counting;
   - exact-copy candidates at depths one through 32;
   - the writer/point fallback;
   - the longest-32 token explicitly.
3. Keep the deepest exact-copy source and writer rank.
4. Compute twelve bounded integer features:
   - bias;
   - exact-copy depth;
   - query/token sign agreement;
   - best agreement over eight cyclic row rotations;
   - source recency;
   - within-window frequency;
   - writer-token match;
   - writer-rank bucket;
   - is-longest-32;
   - signed depth gap from longest-32;
   - sign-agreement gain over longest-32;
   - rotated-agreement gain over longest-32.
5. Fit an anchored perceptron for eight passes at learning rate four with
   weights clipped to `[-4096, 4096]`. Baseline-correct groups make no update.
   Baseline-wrong groups compare their reference target directly with the
   longest-32 candidate.
6. Fit these matched arms over the identical candidate union:
   - anchor trained ranker;
   - anchor shuffled-target ranker;
   - anchor fixed geometry-only ranker;
   - unanchored trained ranker.
7. At generated inference, start from prompt and generated tokens only. Keep
   longest-32 unless another candidate has a strictly greater learned score.
   Longest-32 wins every tie. BOS is excluded.
8. Compare the anchored trained arm with anchored shuffled-target, anchored
   geometry-only, unanchored trained, longest-32 and writer.

No serving, CUDA, float inference, termination or Claude-owned source changes.

## Calibration evidence

- seed-1 calibration JSON SHA-256:
  `05c5c621d238a0896022c39840ed0e91e5ec84e2faba2deb4d7917ef4c1f6607`;
- seed-2 calibration JSON SHA-256:
  `5fccaab34c9b2175782a6e0c9b8f5599de7de32ae29bee895c445d9865d6533b`;
- calibration build wall: 7.63 s; seed 1/2 walls: 4.69/4.57 s;
- seed 1/2 peak RSS: 227,544/219,652 KiB;
- fresh panel read during calibration: false.

## Data split and power

- fit on train panel 2711;
- fresh `cell=dev,dev`, seed 2769, generated only after FIT and GO;
- 128 conversations and both public artifact seeds;
- at least 2,048 target-present groups and 64 target tokens per artifact;
- at least 256 baseline-wrong groups per artifact;
- at least 200 reachable replies per artifact and control;
- fresh-process repeats of both artifacts.

## Bar

On seed 2769 and both artifact seeds, the anchored trained ranker must:

- beat longest-32, writer, anchored geometry-only, anchored shuffled-target
  and unanchored trained on correct body positions by the existing paired
  two-sigma rule;
- change at least one token against every control;
- emit zero premature BOS tokens;
- repeat byte-identically after timing fields are removed.

## Stop rule

Stop on source, candidate, feature, label, weight or report drift; failure of
any training floor; fewer than 200 reachable replies; a vacuous control;
nondeterminism; or any need to edit serving, CUDA, float-inference,
termination or Claude-owned source. Once seed 2769 is read, do not change the
selected cell count, features, anchor, threshold or bar.

## Base artifacts and source

- implementation commit: `57ef4cdc1d6122cfb84d506a000a82ef554dbe88`;
- base: `d416181c1`;
- seed-1 artifact SHA-256:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed-2 artifact SHA-256:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- tokenizer SHA-256:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- train tokens/mask/manifest SHA-256:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`,
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`,
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`;
- `geometric_candidate_anchor.rs` SHA-256:
  `0bd0ae0de443afb55530de291cf9d700b4d21b33fec852cd36aa26e8a38202b0`;
- `cpu-learned-copy-read.rs` SHA-256:
  `66646c06cf11315c8544b915eeddd44628bbbaf78e344198ebfd944bd59c0d7d`;
- `fbs.rs` SHA-256:
  `a8bfc9a40a651688d0c0e625f31cd379a15fbaed37dd8a5a1df2d3050da1647d`;
- `lib.rs` SHA-256:
  `099b484cb3f9ce80f2a78a298b4a429b713a05181076ebbd41cfc7d9a4596c85`.

## Output

`local/deepseek/expanded-candidate-anchor/2026-11-05`. The fresh seed-2769
panel and output root are absent at registration.

## Steps

Run only after FIT and GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-expanded-candidate-anchor
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/expanded-candidate-anchor/2026-11-05
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
PANEL=$OUT/panels/dd-2769
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/expanded-candidate-anchor
GEN_TARGET=$SOURCE/target/deepseek/expanded-candidate-anchor-generator
IMPL=57ef4cdc1d6122cfb84d506a000a82ef554dbe88
BASE=d416181c1

test ! -e "$OUT"
git -C "$SOURCE" diff --quiet "$IMPL" -- crates/uor-r4-cpu-train
printf '%s  %s\n' \
  a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 "$TOKENIZER" \
  a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$SEED1" \
  da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$SEED2" \
  140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91 "$TRAIN/tokens.u16" \
  d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad "$TRAIN/response_mask.u8" \
  d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5 "$TRAIN/manifest.json" |
  sha256sum -c -
printf '%s  %s\n' \
  0bd0ae0de443afb55530de291cf9d700b4d21b33fec852cd36aa26e8a38202b0 \
  "$SOURCE/crates/uor-r4-cpu-train/src/geometric_candidate_anchor.rs" \
  66646c06cf11315c8544b915eeddd44628bbbaf78e344198ebfd944bd59c0d7d \
  "$SOURCE/crates/uor-r4-cpu-train/src/bin/cpu-learned-copy-read.rs" \
  a8bfc9a40a651688d0c0e625f31cd379a15fbaed37dd8a5a1df2d3050da1647d \
  "$SOURCE/crates/uor-r4-cpu-train/src/fbs.rs" \
  099b484cb3f9ce80f2a78a298b4a429b713a05181076ebbd41cfc7d9a4596c85 \
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
  cell=dev,dev conversations=128 seed=2769 protocol=2 \
  >"$OUT/panel.stdout.json"
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2769 and .dialogue_protocol_version == 2
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
    --ranker-trajectory reference --ranker-candidates anchored-expanded \
    --candidate-cells 12 \
    >"$OUT/reports/$name.json" 2>"$OUT/reports/$name.stderr"
}

attempt() {
  name=$1 artifact=$2
  if run "$name" "$artifact"; then
    return 0
  fi
  stderr=$OUT/reports/$name.stderr
  if rg -q 'anchored candidate calibration has' "$stderr"; then
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
    .schema == "uor-r4.cpu-anchored-expanded-candidate-ranker/1"
    and .inputs.artifact_sha256 == $expected
    and .inputs.train_split == "train" and .inputs.test_split == "dev"
    and .settings.cap == 64 and .settings.seed == 163
    and .settings.candidate_set == "expanded_writer_cells_exact32_anchored"
    and .settings.cells == 12 and .settings.writer_candidate_limit == 192
    and .settings.baseline == "longest_32"
    and .settings.override_rule == "strict_positive_score"
    and .settings.served_writer_cells == 8
    and .training.supervised_groups >= 2048
    and .training.target_values >= 64
    and .training.baseline_wrong_groups >= 256
    and .training.writer_candidate_rows > 0
    and .training.min_writer_candidates_per_group > 0
    and .training.max_writer_candidates_per_group <= 192
    and .training.shuffled_target_changes > 0
    and .candidate_ranker.features == ["bias","exact_depth","sign_agreement","rotated_sign_agreement","recency","frequency","writer_match","writer_rank","is_longest_32","depth_gap","sign_agreement_gain","rotated_agreement_gain"]
    and .test.trained.premature_bos == 0
    and .test.shuffled.premature_bos == 0
    and .test.geometry_only.premature_bos == 0
    and .test.unanchored.premature_bos == 0
    and .test.longest_32.premature_bos == 0
    and .test.writer.premature_bos == 0
    and .test.changes.longest_32.non_vacuous == true
    and .test.changes.writer.non_vacuous == true
    and .test.changes.geometry_only.non_vacuous == true
    and .test.changes.shuffled.non_vacuous == true
    and .test.changes.unanchored.non_vacuous == true
    and .test.control_changes_vs_longest.geometry_only.non_vacuous == true
    and .test.control_changes_vs_longest.shuffled.non_vacuous == true
    and .test.control_changes_vs_longest.unanchored.non_vacuous == true
    and (.test.signatures.trained.reply_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.trained.decision_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.shuffled.reply_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.shuffled.decision_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.geometry_only.reply_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.geometry_only.decision_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.unanchored.reply_sha256 | test("^[0-9a-f]{64}$"))
    and (.test.signatures.unanchored.decision_sha256 | test("^[0-9a-f]{64}$"))
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
    and .test.unanchored.reference_reachable >= 200
    and .test.longest_32.reference_reachable >= 200
    and .test.writer.reference_reachable >= 200
  ' "$report" >/dev/null; then
    printf '%s reachable-below-200\n' "$name" >>"$OUT/power-limited.txt"
    POWER_OK=0
  fi
  if ! jq -e '
    .test.comparisons.longest_32.correct_body_positions.passes == true
    and .test.comparisons.writer.correct_body_positions.passes == true
    and .test.comparisons.geometry_only.correct_body_positions.passes == true
    and .test.comparisons.shuffled.correct_body_positions.passes == true
    and .test.comparisons.unanchored.correct_body_positions.passes == true
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
- train-only cell sweep on both artifacts before the fresh panel;
- at least 2,048 target-present groups, 64 target tokens and 256
  baseline-wrong groups per artifact;
- at least 200 reachable replies per artifact and control;
- at least one non-vacuous change against each control;
- fresh-process repeats of both artifact seeds.

## Cost

- host: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads, 59 GiB RAM;
- measured train-only calibration: 7.63 s build, 4.69 + 4.57 s sweeps,
  227,544 KiB peak RSS;
- one release build, one generator build and one fresh panel;
- four driver processes at up to 32 Rayon workers and 1,800 s each;
- at most 192 non-BOS writer candidates, 32 exact-copy candidates and one
  fallback per state;
- local CPU cost $0, no GPU pod, paid API or remote upload;
- fresh output capped at 64 MiB after gzip.

## Outcomes

- **KEEP** if both artifact seeds beat all five controls by the paired rule and
  preserve every control.
- **REJECT** if any required comparison or control fails; do not retune this
  construction on seed 2769.
- **INSUFFICIENT_TRAINING_DATA** if either artifact misses the declared floors.
- **POWER_LIMITED** if fewer than 200 replies are reachable.
- **INSTRUMENT_FAULT** if fitting, features, sources, inputs, controls or
  report identity fails closed.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 expanded-cell longest-32-anchored candidate ranker
sheet: `docs/harness/prep/cpu-training-pilot-expanded-candidate-anchor-2026-11-05.md`
implementation pin: `57ef4cdc1d6122cfb84d506a000a82ef554dbe88`
reviewed head: `db3e92142163f3c523bedbc83045924eddf6aed6`
reviewer: deepseek fresh-context subagent `expanded_anchor_fit_review_3`
note: K=12 is the smallest cell count clearing both artifacts' group, target and
  baseline-wrong floors; observed writer counts are bounded; depth-32, deepest
  source, fallback, explicit anchor, BOS stop, strict-positive override,
  shuffled non-vacuity and all control-vs-longest checks were verified.
date: 2026-11-05

A FIT verdict clears compute for the configuration in this sheet only (D22).

## Pre-run GO

GO - reviewer `deepseek fresh-context subagent expanded_anchor_pre_run_go`,
2026-11-05. HEAD was clean at
`304818effe1560f1d61f4ac75d107dab3c123d76`; `prep-check` reported the sheet
complete FIT. Implementation source, calibration, tokenizer, artifacts, train
panel, registered shell and jq predicates matched or parsed; seed 2769 and its
output root remained absent. The six matched arms, K=12 settings, fail-closed
floors, control-vs-longest non-vacuity, BOS direct stop, deterministic replay,
local resource plan and lane isolation were checked read-only. No seed-2769
generation or evaluation occurred.

## Outcome

**INSTRUMENT_FAULT / K12_SELECTION_ERROR.** The registration selected 12
cells although seed 2 had only 2,043 target-present groups, five below the
2,048 selection floor. The driver correctly stopped before evaluation with:

```text
anchored candidate calibration has 2043 groups, 330 target tokens and 459 baseline-wrong groups; requires 2048, 64 and 256
```

The raw automation outcome is `INSUFFICIENT_TRAINING_DATA`. This is preserved
unchanged in the local run. The registration's arithmetic selection was
invalid, so no anchored-ranker comparison is claimed. Seed 2769 is closed and
must not be reused. K=16 was already in the predeclared train-only sweep and is
the smallest setting that clears all three floors on both artifacts.
