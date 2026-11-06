# Pre-registration: prompt-set complete-reply retrieval

Run ID: `cpu-training-pilot-prompt-set-retrieval-2026-11-05`

## Status

COMPLETE - **KEEP**. The fresh seed-2777 confirmation returned 39/471 exact
replies on both artifacts with zero premature stops and 948 body positions.
Record: [deepseek-204](../../loop/tasks/cpu-training-pilot/deepseek-204/REPORT.md).

This is D84 CPU development research. It is exact-memory evidence, not general
fresh-wording understanding, serving integration, useful chat or GPU
qualification.

## Question

Can exact token-set overlap retrieve a complete learned reply, with its natural
endpoint, more reliably than tokenwise copy/closure or prompt-query geometry?

The prior continuation sequence established a content-positive,
termination-limited result. An analysis of the sealed seed-2776 traces found no
robust endpoint-only separator for the same suffix history. This unit tests a
different completion object: one whole TRAIN response selected directly from
the prompt set.

## Fixed construction

For every TRAIN sample:

1. Normalize its reply by removing one trailing EOS/BOS stop marker.
2. Convert the prompt token sequence to a set of token identities.
3. Convert it separately to a set of adjacent token pairs for the bigram
   control.
4. Replay the prompt through the artifact-bound `CpuFbsForward` writer path and
   retain the final Q12 query for the geometry controls.

At evaluation:

1. Form the prompt's exact token set.
2. Compare it with every TRAIN prompt using exact set Jaccard.
3. Choose the greatest rational score by integer cross-multiplication.
4. Equal scores choose the lower TRAIN ordinal.
5. Emit the selected complete TRAIN reply and BOS in one decision.

No reference is read by any non-oracle arm. No dense matmul, softmax, squashing
activation or float serving path is used.

## Arms

1. `unigram_jaccard_ordinal_tie`: treatment;
2. `bigram_jaccard_geometry_tie`;
3. `unigram_jaccard_shuffled_replies`: equal-size permutation of complete
   TRAIN replies;
4. `query_geometry_nearest`: minimum Q12 writer-query L1;
5. `fixed_ordinal`: always return the first TRAIN reply;
6. `reference_membership_oracle`: return the reference only when the complete
   reference reply occurs in TRAIN; diagnostic.

The geometry-tie unigram arm is also reported as a non-superiority control but
does not replace the ordinal-tie treatment.

## Data and power

- TRAIN from panel `tt-2711` only;
- fresh unopened `cell=dev,dev`, seed 2777, protocol 2, 128 conversations,
  generated only after FIT and GO;
- both public dual-c131 raw artifact seeds;
- 64-token cap;
- at least 200 reference-reachable replies per artifact;
- at least 200 panel prompts without an exact TRAIN prompt match;
- at least 100 changed complete-reply associations in the deterministic
  shuffle;
- fresh-process repeats of both artifact seeds.

## Instrument gate

- source, artifact, tokenizer, input and generator pins match;
- the fresh panel and output root were absent before release;
- prompt sets and bigram sets are nonempty, with at least 300 distinct
  unigram-set signatures;
- training contains at least 250 distinct complete replies;
- the deterministic shuffle changes at least 100 complete-reply associations;
- every non-oracle arm emits a complete BOS result on every
  reference-reachable row;
- treatment, bigram, shuffled, geometry and fixed arms differ behaviorally;
- both artifact seeds and fresh-process repeats match after timing removal.

Any failure is an instrument, power or drift fault, not a mechanism negative.

## Bar

For both artifacts, `unigram_jaccard_ordinal_tie` must:

- produce at least `max(25, ceil(reference_membership_exact / 8))` exact
  replies;
- beat bigram Jaccard, shuffled replies and fixed ordinal on exact replies by
  the paired two-sigma rule;
- preserve at least as many correct body positions as bigram Jaccard and
  writer-query geometry;
- report zero premature BOS;
- emit a complete result for every reference-reachable row;
- show that the Q12 geometry-tie arm is not superior in exact replies or body
  positions;
- repeat exactly after timing fields are removed.

If the reference-membership oracle has fewer than ten exact replies, exact
comparison is `INSUFFICIENT_POWER`. A vacuous control is `CONTROL_FAULT`.
`KEEP` requires every exact, body, complete-emission and control condition.

## Compute class

Local CPU development on the Ryzen 9 9950X. No GPU, pod, paid API, upload or
external model. Serving remains integer-only and unchanged.

## Base, source and inputs

- base: `77f189f58`;
- implementation:
  `135a40b6c`;
- `prompt_retrieval.rs` SHA-256:
  `4d37ba343daf64ac7935dd273c736e8c430192a443a6371dffaab362bc5f87b9`;
- `cpu-prompt-retrieval.rs` SHA-256:
  `1746a3b92ec111255585c9165e5946eb4065c66ead62d0c92483b27a8b0f584b`;
- `lib.rs` SHA-256:
  `c8e23400ec4b67ba2d4b449569e757895f44e5f4e6bc60d2b0175d8e0f56b0db`;
- generator tree:
  `869e65db6d0bb997a5f1094ab0de4efa1ac10c25`;
- tokenizer SHA-256:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- artifact seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- artifact seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.

## Data split

TRAIN panel:

`/home/callard/projects/codex-playground/uor-r4/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711`

- tokens:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- mask:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- manifest:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`.

Fresh evaluation panel:

`/home/callard/projects/codex-playground/uor-r4/local/deepseek/prompt-retrieval/2026-11-05/panels/dd-2777`

The panel and output root are absent at registration. No evaluation reply
enters prompt retrieval or shuffle construction.

## Output

`/home/callard/projects/codex-playground/uor-r4/local/deepseek/prompt-retrieval/2026-11-05`

## Steps

Run only after FIT and GO. Use a fresh output root and fail closed.

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-continuation-geometry-analysis
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/prompt-retrieval/2026-11-05
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
PANEL=$OUT/panels/dd-2777
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/prompt-retrieval
GEN_TARGET=$SOURCE/target/deepseek/prompt-retrieval-generator
IMPL=135a40b6c
GEN_TREE=869e65db6d0bb997a5f1094ab0de4efa1ac10c25
RUN_DEADLINE=$((SECONDS + 1800))

test ! -e "$OUT"
test ! -e "$PANEL"
git -C "$SOURCE" diff --quiet "$IMPL" -- crates/uor-r4-cpu-train
test "$(git -C "$SOURCE" rev-parse "$IMPL:docs/labs/one-model-end-to-end/instrument")" = "$GEN_TREE"
printf '%s  %s\n' \
  a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 "$TOKENIZER" \
  a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$SEED1" \
  da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$SEED2" \
  140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91 "$TRAIN/tokens.u16" \
  d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad "$TRAIN/response_mask.u8" \
  d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5 "$TRAIN/manifest.json" \
  4d37ba343daf64ac7935dd273c736e8c430192a443a6371dffaab362bc5f87b9 "$SOURCE/crates/uor-r4-cpu-train/src/prompt_retrieval.rs" \
  1746a3b92ec111255585c9165e5946eb4065c66ead62d0c92483b27a8b0f584b "$SOURCE/crates/uor-r4-cpu-train/src/bin/cpu-prompt-retrieval.rs" \
  c8e23400ec4b67ba2d4b449569e757895f44e5f4e6bc60d2b0175d8e0f56b0db "$SOURCE/crates/uor-r4-cpu-train/src/lib.rs" |
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
  cargo build --locked --release -p uor-r4-cpu-train --bin cpu-prompt-retrieval
/usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GEN_TARGET" \
  cargo build --locked --manifest-path "$SOURCE/docs/labs/one-model-end-to-end/instrument/Cargo.toml" \
  --features panel-writer --bin heldout-panel
GEN=$GEN_TARGET/debug/heldout-panel

/usr/bin/time -v -o "$OUT/panel.time.txt" \
  timeout 900s "$GEN" out="$PANEL" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2777 protocol=2 \
  >"$OUT/panel.stdout.json"
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2777 and .dialogue_protocol_version == 2
  and .cell.value == "dev" and .cell.phrasing == "dev"
  and .conversations == 128 and .world == "m-world-v2"
  and .tokenizer.sha256 == $tokenizer_sha
' "$PANEL/manifest.json" >/dev/null

BIN=$BUILD/release/cpu-prompt-retrieval
run() {
  name=$1 artifact=$2 expected=$3
  /usr/bin/time -v -o "$OUT/reports/$name.time.txt" \
    timeout 1800s env RAYON_NUM_THREADS=32 "$BIN" \
    "$TRAIN/tokens.u16" "$TRAIN/response_mask.u8" "$TRAIN/manifest.json" \
    "$PANEL/tokens.u16" "$PANEL/response_mask.u8" "$PANEL/manifest.json" \
    "$artifact" --cap 64 --seed 204 \
    --expected-train-tokens 140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91 \
    --expected-train-mask d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad \
    --expected-train-manifest d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5 \
    --expected-panel-tokens "$(jq -r .tokens_sha256 "$PANEL/manifest.json")" \
    --expected-panel-mask "$(jq -r .mask_sha256 "$PANEL/manifest.json")" \
    --expected-panel-manifest "$(sha256sum "$PANEL/manifest.json" | cut -d' ' -f1)" \
    --expected-artifact "$expected" \
    >"$OUT/reports/$name.json"
}

run seed1 "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run seed2 "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
run seed1-repeat "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run seed2-repeat "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1

normalize() {
  jq -S 'walk(if type == "object"
    then del(.wall_elapsed_us, .generated_tokens_per_second, .elapsed_us)
    else . end)' "$1"
}
cmp <(normalize "$OUT/reports/seed1.json") <(normalize "$OUT/reports/seed1-repeat.json")
cmp <(normalize "$OUT/reports/seed2.json") <(normalize "$OUT/reports/seed2-repeat.json")
jq -e '
  .bar.outcome == "KEEP"
  and .bar.control_non_vacuous == true
  and .bar.exact_floor_pass == true
  and .bar.exact_comparisons_pass == true
  and .bar.premature_bos_zero == true
  and .bar.body_nonfall == true
  and .bar.all_reachable_emitted == true
  and .bar.geometry_tie_non_superior == true
' "$OUT/reports/seed1.json" "$OUT/reports/seed2.json"
printf 'KEEP\n' >"$OUT/outcome.txt"
gzip -9 "$OUT"/reports/*.json
test "$(du -sb "$OUT" | cut -f1)" -le 33554432
```

## Minimum test power

- two public artifact seeds;
- one fresh DEV panel with 128 conversations and at least 200 reachable replies
  per artifact;
- at least 200 prompts without an exact TRAIN prompt;
- at least 100 changed shuffled reply associations;
- at least 25 required exact replies, unless the membership-quality floor
  raises it;
- fresh-process repeats of both artifact seeds.

## Cost

- host: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads;
- one release build, one generator build and one fresh panel;
- four driver processes at up to 32 Rayon workers and 1,800 s each;
- local CPU cost $0, no GPU pod, paid API or remote upload;
- retained evidence capped at 32 MiB.

## Outcomes

- **KEEP** if both artifact seeds clear every exact, body, complete-emission,
  non-superiority and repeat condition.
- **REJECT** if a required comparison fails; do not retune this construction on
  seed 2777.
- **INSUFFICIENT_POWER** if the membership oracle has fewer than ten exact
  replies.
- **CONTROL_FAULT** if a shuffled/control source is vacuous or inputs drift.

## Stop rule

Stop on source, artifact, panel, table, retrieval, parity, determinism or
output-root drift; fewer than 200 reference-reachable rows; fewer than 200
panel prompts without an exact TRAIN prompt; fewer than 300 unique TRAIN
unigram prompt sets; fewer than 250 unique TRAIN replies; fewer than 100
changed shuffled reply associations; a vacuous control; any need to edit
serving, CUDA, float-training, exporter or Claude-owned source; wall time above
30 minutes; peak RSS above 2 GiB; or retained evidence above 32 MiB.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 prompt-set complete-reply retrieval
sheet: docs/harness/prep/cpu-training-pilot-prompt-set-retrieval-2026-11-05.md
reviewer: deepseek exact-head self-review (independent delegation unavailable in this run)
note: Exact-head source and input pins match; five focused tests, CPU release
  suite and strict scoped Clippy pass; fresh seed-2777 panel and output root are
  absent; all bar, stop and control gates are fixed. A later D23 wording-only
  correction leaves the release binary byte-identical.
date: 2026-11-05
A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone
before compute.

## Pre-run GO

GO - exact-head self-review under the current disabled-delegation policy,
2026-11-05. Implementation `135a40b6c` is clean; source, tokenizer, artifact,
TRAIN panel and generator pins match; focused and full CPU tests pass; strict
scoped Clippy and formatting pass; output root and seed-2777 panel are absent.
The review limitation is explicit: this was not an independent reviewer.
