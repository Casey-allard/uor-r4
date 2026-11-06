# Pre-registration: full-sequence suffix arbitration

Run ID: `cpu-training-pilot-full-sequence-suffix-2026-11-05`

## Status

COMPLETE - REJECT under the combined pre-registered bar, with a positive
content result. Full-sequence depth arbitration raised exact replies from
0/0 to 17/17 and body positions from 578/578 to 847/844 on artifact seeds 1
and 2, while the unchanged boundary made 85/104 premature stops against the
four-stop ceiling. Content is promoted as a promising configuration; the
combined decode is not. Record:
[deepseek-198](../../loop/tasks/cpu-training-pilot/deepseek-198/REPORT.md).

## Question

Can reply-target suffix memory trained over full prompt-plus-reply histories
improve body reconstruction when it reads the same token stream as local exact
copy and replaces local only when its confident suffix is deeper?

The treatment preserves local matches at equal or greater depth. A global
prediction requires at least two observations and a strict-majority successor.
Stopping remains deepseek-191's independent true reply-boundary memory.

This is D84 CPU development research. It is not fresh-wording generalization,
useful chat, serving integration or GPU qualification.

## Why this is new

- deepseek-193's global memory displaced local copy and included stopping
  targets.
- deepseek-196 removed both confounds but trained reply-only keys while local
  copy read the prompt. Local copy then covered 25,202/25,202 reachable
  decisions, leaving the fallback unreachable.
- This unit trains keys over prompt-plus-reply histories, the same surface used
  by local copy, including prompt-conditioned reply starts.
- A same-depth rule compares the two memories directly: deeper confident global
  context may replace shallow local context; ties remain local.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model. The result cannot substitute for a GPU qualification
bar. Serving remains integer-only and unchanged.

## Base artifact and source

- base: `0e3924274`;
- implementation commit: `fb439a0666366971b87b40697416dd50ee9d4de4`;
- `suffix_arbitration.rs` SHA-256:
  `2cd67076025a66801b7fc141b7bd68094b83fad796a38f403928ce307a31e9c5`;
- `suffix_memory.rs` SHA-256:
  `7cff7a71b61cc7c38b3b048124296e6cc64a77441e622e9f34b5ff6531656c37`;
- `cpu-copy-suffix-arbitration.rs` SHA-256:
  `c03976fd9d8079a8323d2e7fd2ba88bf593f846a4fe0de084bf5cc0b1972422f`;
- panel generator tree:
  `869e65db6d0bb997a5f1094ab0de4efa1ac10c25`;
- tokenizer SHA-256:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- public FBS seed 1 SHA-256:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- public FBS seed 2 SHA-256:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- host: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

The public artifacts are unchanged raw dual-c131 control models.

## Data split

Training and table derivation:

`/home/callard/projects/codex-playground/uor-r4/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711`

- tokens SHA-256:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- response mask SHA-256:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- manifest SHA-256:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`;
- `cell=train,train`, seed 2711, 128 conversations.

Fresh evaluation panel:

`/home/callard/projects/codex-playground/uor-r4/local/deepseek/full-sequence-suffix/2026-11-05/panels/dd-2774`

- generated only after FIT and GO;
- `cell=dev,dev`, seed 2774, protocol 2, 128 conversations;
- output root and panel directory are absent at registration;
- no evaluation reply enters table derivation.

## Fixed construction

For every TRAIN conversation:

1. Start history with prompt tokens.
2. For each normalized reply token, count every suffix of history length one
   through 32 to that reply token.
3. Append the reply token to history.
4. Do not add decoder BOS or terminal successors.

At generation:

1. apply the true reply-boundary memory before each token;
2. read local `Longest32To1` on the complete stream;
3. read the content table on that same stream;
4. accept global only with at least two observations and a strict-majority
   successor;
5. use global only when its depth is strictly greater than local depth or no
   local candidate exists;
6. otherwise preserve local, then writer.

No dense matmul, softmax, squashing activation or float serving path is used.

## Arms

1. `deeper_full_sequence_32`;
2. `deeper_full_sequence_shuffled`;
3. `shallow_local_global_4`;
4. `full_sequence_32_only`;
5. `local_longest_32_boundary`;
6. `writer_boundary`;
7. `reference_length_oracle`.

The shuffle redistributes successors among equal-depth occurrences, preserving
each suffix's observation count and each depth's target multiset.

## Instrument gate

Before a fresh result is read:

- implementation and all inputs match;
- content table has zero BOS transitions;
- true and shuffled censuses preserve support, transition and target sets;
- treatment makes at least 100 reachable deeper-global decisions where its
  selected token differs from local;
- treatment changes at least one decision from local and shuffled;
- every arm has at least 200 reachable replies;
- fresh-process repeats match after timing removal.

Any failure is an instrument or power fault, not a mechanism negative.

## Minimum test power

- two independently trained public artifact seeds;
- one fresh DEV panel with 128 conversations and at least 200 reachable replies
  per artifact seed;
- at least 100 reachable deeper-global decisions;
- non-vacuous local and shuffled comparisons;
- fresh-process repeats of both artifact seeds;
- complete sequence bounded to 30 minutes and 2 GiB peak RSS.

This meets D22's at least 200 fresh rows. No evaluation row is used for fitting
or control construction.

## Bar

On both artifact seeds, `deeper_full_sequence_32` must beat local longest,
shallow-local depth-4, shuffled, global-only and writer on correct body
positions by:

```text
candidate sum > control sum
candidate-only >= max(2, ceil(2 * sqrt(control-only)))
```

It must keep premature BOS at or below four, make at least 100 reachable
deeper-global replacements, and repeat exactly.

Exact complete replies are the primary capability outcome. If the reference
oracle has fewer than ten exact bodies, exact comparison is
`INSUFFICIENT_POWER`; a body pass is retained only as
`BODY_POSITIVE_INSUFFICIENT_COMPLETION`. With at least ten oracle-exact bodies,
treatment must also beat local longest on exact complete replies by the paired
rule.

## R12 authority

1. **What is measured:** paired complete replies under fixed selectors.
2. **Whether it is computable:** yes. Exact CPU FBS, public artifacts,
   tokenizer and deterministic panels are present.
3. **Whether the instrument can beat the bar:** yes in principle. Full-sequence
   keys expose prompt-conditioned continuations and a direct depth comparison.
4. **Whether it should run now:** yes. It resolves the structural coverage
   failure in deepseek-196 without abandoning local exact copy.

## Cost

- preparation and finite checks: 90 s;
- release build: 30 s;
- generator build and panel draw: 40 s;
- TRAIN preflight: 15 s;
- four fresh artifact runs and two repeats: 60 s;
- evidence and delivery: 10 minutes;
- total bounded increment: 15 minutes;
- build jobs: 16; build RSS cap: 4 GiB;
- each evaluator run: 32 Rayon workers, 600 s wall cap and 2 GiB peak RSS;
- retained evidence cap: 32 MiB; fresh target cap: 8 GiB;
- placement: local Ryzen 9 9950X under D84; no pod or GPU.

## Steps

Run only after FIT and D67 GO. Use a fresh output root and fail closed.

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-full-sequence-suffix
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/full-sequence-suffix/2026-11-05
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
PANEL=$OUT/panels/dd-2774
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/full-sequence-suffix
GEN_TARGET=$SOURCE/target/deepseek/full-sequence-suffix-generator
IMPL=fb439a0666366971b87b40697416dd50ee9d4de4
GEN_TREE=869e65db6d0bb997a5f1094ab0de4efa1ac10c25

test ! -e "$OUT"
test ! -e "$PANEL"
git -C "$SOURCE" diff --quiet "$IMPL" -- crates/uor-r4-cpu-train
test "$(git -C "$SOURCE" rev-parse 0e3924274:docs/labs/one-model-end-to-end/instrument)" = "$GEN_TREE"
printf '%s  %s\n' \
  a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 "$TOKENIZER" \
  a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$SEED1" \
  da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$SEED2" \
  140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91 "$TRAIN/tokens.u16" \
  d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad "$TRAIN/response_mask.u8" \
  d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5 "$TRAIN/manifest.json" \
  2cd67076025a66801b7fc141b7bd68094b83fad796a38f403928ce307a31e9c5 "$SOURCE/crates/uor-r4-cpu-train/src/suffix_arbitration.rs" \
  7cff7a71b61cc7c38b3b048124296e6cc64a77441e622e9f34b5ff6531656c37 "$SOURCE/crates/uor-r4-cpu-train/src/suffix_memory.rs" \
  c03976fd9d8079a8323d2e7fd2ba88bf593f846a4fe0de084bf5cc0b1972422f "$SOURCE/crates/uor-r4-cpu-train/src/bin/cpu-copy-suffix-arbitration.rs" |
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
check_rss() {
  local log=$1
  local limit_kib=$2
  local rss
  rss=$(awk '/Maximum resident set size/ { print $NF }' "$log")
  test -n "$rss"
  test "$rss" -le "$limit_kib"
}

cd "$SOURCE"
/usr/bin/time -v -o "$OUT/build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" \
  cargo build --locked --release -p uor-r4-cpu-train --bin cpu-copy-suffix-arbitration
check_rss "$OUT/build.time.txt" 4194304
/usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GEN_TARGET" \
  cargo build --locked \
  --manifest-path "$SOURCE/docs/labs/one-model-end-to-end/instrument/Cargo.toml" \
  --features panel-writer --bin heldout-panel
check_rss "$OUT/generator-build.time.txt" 4194304
GEN=$GEN_TARGET/debug/heldout-panel
BIN=$BUILD/release/cpu-copy-suffix-arbitration

TRAIN_TOKENS_SHA=$(sha256sum "$TRAIN/tokens.u16" | cut -d ' ' -f1)
TRAIN_MASK_SHA=$(sha256sum "$TRAIN/response_mask.u8" | cut -d ' ' -f1)
TRAIN_MANIFEST_SHA=$(sha256sum "$TRAIN/manifest.json" | cut -d ' ' -f1)

/usr/bin/time -v -o "$OUT/reports/train-preflight.time.txt" \
  timeout 600s env RAYON_NUM_THREADS=32 "$BIN" \
  "$TRAIN/tokens.u16" "$TRAIN/response_mask.u8" "$TRAIN/manifest.json" \
  "$TRAIN/tokens.u16" "$TRAIN/response_mask.u8" "$TRAIN/manifest.json" \
  "$SEED1" \
  --expected-train-tokens "$TRAIN_TOKENS_SHA" \
  --expected-train-mask "$TRAIN_MASK_SHA" \
  --expected-train-manifest "$TRAIN_MANIFEST_SHA" \
  --expected-panel-tokens "$TRAIN_TOKENS_SHA" \
  --expected-panel-mask "$TRAIN_MASK_SHA" \
  --expected-panel-manifest "$TRAIN_MANIFEST_SHA" \
  --expected-artifact a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd \
  --cap 64 --seed 198 >"$OUT/reports/train-preflight.json"
check_rss "$OUT/reports/train-preflight.time.txt" 2097152
jq -e '
  .examples.reachable >= 200
  and .table.content.bos_transitions == 0
  and .table.content.transitions > 0
  and .table.changed_targets > 0
  and .changes.from_local.changed_steps > 0
  and .changes.from_shuffled.changed_steps > 0
  and .bar.global_decisions >= 100
' "$OUT/reports/train-preflight.json" >/dev/null

/usr/bin/time -v -o "$OUT/panel.time.txt" \
  timeout 900s "$GEN" out="$PANEL" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2774 protocol=2 \
  >"$OUT/panel.stdout.json"
check_rss "$OUT/panel.time.txt" 2097152
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2774 and .dialogue_protocol_version == 2
  and .cell.value == "dev" and .cell.phrasing == "dev"
  and .conversations == 128 and .world == "m-world-v2"
  and .tokenizer.sha256 == $tokenizer_sha
' "$PANEL/manifest.json" >/dev/null
PANEL_TOKENS_SHA=$(sha256sum "$PANEL/tokens.u16" | cut -d ' ' -f1)
PANEL_MASK_SHA=$(sha256sum "$PANEL/response_mask.u8" | cut -d ' ' -f1)
PANEL_MANIFEST_SHA=$(sha256sum "$PANEL/manifest.json" | cut -d ' ' -f1)

run() {
  name=$1 artifact=$2 expected=$3
  /usr/bin/time -v -o "$OUT/reports/$name.time.txt" \
    timeout 600s env RAYON_NUM_THREADS=32 "$BIN" \
    "$TRAIN/tokens.u16" "$TRAIN/response_mask.u8" "$TRAIN/manifest.json" \
    "$PANEL/tokens.u16" "$PANEL/response_mask.u8" "$PANEL/manifest.json" \
    "$artifact" \
    --expected-train-tokens "$TRAIN_TOKENS_SHA" \
    --expected-train-mask "$TRAIN_MASK_SHA" \
    --expected-train-manifest "$TRAIN_MANIFEST_SHA" \
    --expected-panel-tokens "$PANEL_TOKENS_SHA" \
    --expected-panel-mask "$PANEL_MASK_SHA" \
    --expected-panel-manifest "$PANEL_MANIFEST_SHA" \
    --expected-artifact "$expected" \
    --cap 64 --seed 198 >"$OUT/reports/$name.json"
  check_rss "$OUT/reports/$name.time.txt" 2097152
}
run seed1 "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run seed2 "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
for report in "$OUT/reports/seed1.json" "$OUT/reports/seed2.json"; do
  jq -e --arg panel_tokens "$PANEL_TOKENS_SHA" \
    --arg panel_mask "$PANEL_MASK_SHA" \
    --arg panel_manifest "$PANEL_MANIFEST_SHA" '
    .inputs.panel.tokens_sha256 == $panel_tokens
    and .inputs.panel.mask_sha256 == $panel_mask
    and .inputs.panel.manifest_sha256 == $panel_manifest
    and .examples.reachable >= 200
    and .table.content.bos_transitions == 0
    and .table.changed_targets > 0
    and .bar.global_decisions >= 100
    and .changes.from_local.changed_steps > 0
    and .changes.from_shuffled.changed_steps > 0
    and .table.content.suffixes == .table.shuffled.suffixes
    and .table.content.transitions == .table.shuffled.transitions
    and .table.content.suffixes_by_depth == .table.shuffled.suffixes_by_depth
    and .table.content.transitions_by_depth == .table.shuffled.transitions_by_depth
    and .table.content.distinct_targets_by_depth
      == .table.shuffled.distinct_targets_by_depth
  ' "$report" >/dev/null
done
run seed1-repeat "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run seed2-repeat "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1

normalize() {
  jq -S 'walk(if type == "object"
    then del(.wall_elapsed_us, .generated_tokens_per_second)
    else . end)' "$1"
}
cmp <(normalize "$OUT/reports/seed1.json") \
  <(normalize "$OUT/reports/seed1-repeat.json")
cmp <(normalize "$OUT/reports/seed2.json") \
  <(normalize "$OUT/reports/seed2-repeat.json")
```

## Stop rule

Stop on source, artifact, panel, table, confidence, depth rule, report,
output-root or repeat drift; fewer than 200 reachable replies; fewer than 100
deeper-global replacements; a vacuous shuffle; nondeterminism; wall time above
30 minutes; peak RSS at or above 2 GiB; or any need to edit serving, CUDA,
float-training, exporter or Claude-owned source. Once seed 2774 is read, do not
change local priority, depth comparison, confidence, shuffle or bar.

## Verdict

TEST FITNESS: FIT - M2 #2030

reviewer: full_sequence_fit_review_2
date: 2026-11-05
note: Corrected head 767705441; selected-source accounting and
global-selection trajectory test verified.

## D67 pre-run review

D67 PRE-RUN: GO - implementation `fb439a066`, review head `0566e32e9`;
source/input pins match; seed 2774 and the output root are absent; full
prompt-plus-reply keys and same-stream arbitration are tested; the wrapper
enforces RSS, activates TRAIN before the panel, and fails each seed before
repeats on reachability, 100 global selections, local/shuffle non-vacuity, BOS
absence and census equality; and the work remains isolated from Claude's scale
probe. Reviewer: `full_sequence_go_review`, 2026-11-05.
