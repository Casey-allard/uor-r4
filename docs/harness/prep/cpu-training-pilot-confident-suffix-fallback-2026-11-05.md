# Pre-registration: confident content-suffix fallback

Run ID: `cpu-training-pilot-confident-suffix-fallback-2026-11-05`

## Status

STOPPED AT PREFLIGHT - INSUFFICIENT FALLBACK COVERAGE. Local exact copy had a
candidate at every one of the 25,202 reachable generated decisions, so the
confident content fallback made zero decisions against the required minimum of
100. Seed 2773 was not generated. This is not a mechanism negative; it shows
that strict local priority made the proposed treatment unreachable on TRAIN.
Record: [deepseek-196](../../loop/tasks/cpu-training-pilot/deepseek-196/REPORT.md).

## Question

Can a reply-only suffix memory improve complete-reply body reconstruction when
it is stripped of stopping targets, gated by strict-majority confidence, and
used only where established local `Longest32To1` has no candidate?

The local exact-copy selector remains first. At an uncovered decision, the
system searches TRAIN reply suffixes from depth 32 down to one and uses the
deepest suffix with at least two observations and a successor count strictly
greater than half its observations. Otherwise it uses the writer. The true
deepseek-191 boundary remains the only stopping path.

This is D84 CPU development research. It is not fresh-wording generalization,
useful chat, serving integration or GPU qualification.

## Why this is new

- deepseek-193's full suffix table included BOS and globally displaced local
  copy. It reached 57/86 body positions against 514/514 for local copy and made
  14/227 premature stops.
- It therefore confounded content with stopping and global replacement with
  local exact memory.
- This unit removes both confounds: no BOS targets, no displacement while a
  local candidate exists, and a strict support/majority gate for every global
  fallback.
- A depth-one fallback isolates suffix length; a successor-multiset shuffle
  isolates learned continuation association.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model. The result cannot substitute for a GPU qualification
bar. Serving remains integer-only and unchanged.

## Base artifact and source

- base: `ebbe084e9`;
- implementation commit: `c2b2fda872f3ba19e74cad80de01a9ba84ca6ec6`;
- `suffix_fallback.rs` SHA-256:
  `3dfd23c11f74af472a94868e2c9cdc21ff37a914b93e7a17bdc87b4a17c4f926`;
- `suffix_memory.rs` SHA-256:
  `a5d8a13a15beae8112b4575e31985a501a7949e34ace71fbe95c20d08f3de7c2`;
- `cpu-copy-suffix-fallback.rs` SHA-256:
  `00c9b41cb1fa235ccc2d21e79c4a0a3d78d86b2c08ee3aeba51243caf6776df2`;
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

`/home/callard/projects/codex-playground/uor-r4/local/deepseek/suffix-fallback/2026-11-05/panels/dd-2773`

- generated only after FIT and GO;
- `cell=dev,dev`, seed 2773, protocol 2, 128 conversations;
- the output root and panel directory are absent at registration;
- no evaluation reply enters table derivation.

## Fixed construction

For every normalized TRAIN reply:

1. Count every suffix of length 1 through 32 whose next reply token exists.
2. Do not add decoder BOS or any terminal successor.
3. Preserve exact integer counts.

At generation:

1. apply the true reply-boundary memory before each token;
2. use local `Longest32To1` whenever it has a candidate;
3. otherwise search generated-reply suffixes from 32 down to one;
4. use the deepest suffix with at least two observations and a plurality
   successor count strictly greater than half of those observations;
5. otherwise use the writer.

No dense matmul, softmax, squashing activation or float serving path is used.

## Arms

For each artifact seed:

1. `local_then_content_32`;
2. `local_then_content_depth_1`;
3. `local_then_shuffled_content_32`;
4. `content_32_only`;
5. `local_longest_32_boundary`;
6. `writer_boundary`;
7. `reference_length_oracle`, reported only as the exact-body ceiling.

The shuffle redistributes successor labels among equal-depth occurrences while
preserving each suffix's observation count and each depth's target multiset.

## Instrument gate

Before a fresh result is read:

- implementation and every input hash match;
- the content table has zero BOS transitions;
- true and shuffled tables preserve depth support and target multisets;
- treatment makes at least 100 reachable confident-suffix fallback decisions;
- treatment changes at least one reachable decision from local longest and
  from shuffled fallback;
- every artifact has at least 200 reference-reachable replies;
- a fresh process reproduces every non-timing field and digest.

Any failure is an instrument or power fault, not a mechanism negative.

## Minimum test power

- two independently trained public artifact seeds;
- one fresh DEV panel with 128 conversations and at least 200 reachable replies
  per artifact seed;
- at least 100 reachable confident fallbacks per artifact seed;
- non-vacuous local and shuffled comparisons;
- fresh-process repeats of both artifact seeds;
- complete fresh sequence bounded to 30 minutes and 2 GiB peak RSS.

This meets D22's at least 200 fresh rows. No evaluation row is used for fitting
or control construction.

## Bar

On both artifact seeds, `local_then_content_32` must beat local longest,
depth-one fallback, shuffled fallback, content-only and writer on correct body
positions by:

```text
candidate sum > control sum
candidate-only >= max(2, ceil(2 * sqrt(control-only)))
```

It must keep premature BOS at or below four per panel, make at least 100
reachable confident fallbacks, and reproduce exactly.

Exact complete replies are the primary capability outcome. If the reference
oracle has fewer than ten exact bodies, exact comparison is
`INSUFFICIENT_POWER`; a body pass is retained only as
`BODY_POSITIVE_INSUFFICIENT_COMPLETION`. With at least ten oracle-exact bodies,
treatment must also beat local longest on exact complete replies by the paired
rule.

## R12 authority

1. **What is measured:** paired complete replies under fixed selectors, scored
   on exact bodies, BOS completion, premature BOS and correct body positions.
2. **Whether it is computable:** yes. The exact CPU FBS forward, public
   artifacts, tokenizer and deterministic panels are present.
3. **Whether the instrument can beat the bar:** yes in principle. The global
   table supplies continuations absent from the current stream, while local
   priority and the confidence gate prevent the observed displacement.
4. **Whether it should run now:** yes. It directly isolates the two confounds
   from deepseek-193 and tests a conservative route to better continuation.

## Cost

- preparation and finite checks: about 90 s;
- release build: about 30 s;
- generator build and panel draw: about 30 s;
- TRAIN preflight: about 15 s;
- four fresh artifact runs and two repeats: about 60 s;
- evidence and delivery: about 10 minutes;
- total bounded increment: about 15 minutes;
- build jobs: 16; build RSS cap: 4 GiB;
- each evaluator run: 32 Rayon workers, 600 s wall cap and 2 GiB peak RSS;
- retained evidence cap: 32 MiB; fresh target cap: 8 GiB;
- placement: local Ryzen 9 9950X under D84; no pod or GPU.

## Steps

Run only after FIT and D67 GO. Use a fresh output root and fail closed on every
identity drift.

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-suffix-fallback
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/suffix-fallback/2026-11-05
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
PANEL=$OUT/panels/dd-2773
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/suffix-fallback
GEN_TARGET=$SOURCE/target/deepseek/suffix-fallback-generator
IMPL=c2b2fda872f3ba19e74cad80de01a9ba84ca6ec6
GEN_TREE=869e65db6d0bb997a5f1094ab0de4efa1ac10c25

test ! -e "$OUT"
test ! -e "$PANEL"
git -C "$SOURCE" diff --quiet "$IMPL" -- crates/uor-r4-cpu-train
test "$(git -C "$SOURCE" rev-parse ebbe084e9:docs/labs/one-model-end-to-end/instrument)" = "$GEN_TREE"
printf '%s  %s\n' \
  a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 "$TOKENIZER" \
  a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$SEED1" \
  da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$SEED2" \
  140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91 "$TRAIN/tokens.u16" \
  d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad "$TRAIN/response_mask.u8" \
  d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5 "$TRAIN/manifest.json" \
  3dfd23c11f74af472a94868e2c9cdc21ff37a914b93e7a17bdc87b4a17c4f926 "$SOURCE/crates/uor-r4-cpu-train/src/suffix_fallback.rs" \
  a5d8a13a15beae8112b4575e31985a501a7949e34ace71fbe95c20d08f3de7c2 "$SOURCE/crates/uor-r4-cpu-train/src/suffix_memory.rs" \
  00c9b41cb1fa235ccc2d21e79c4a0a3d78d86b2c08ee3aeba51243caf6776df2 "$SOURCE/crates/uor-r4-cpu-train/src/bin/cpu-copy-suffix-fallback.rs" |
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
  cargo build --locked --release -p uor-r4-cpu-train --bin cpu-copy-suffix-fallback
check_rss "$OUT/build.time.txt" 4194304
/usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GEN_TARGET" \
  cargo build --locked \
  --manifest-path "$SOURCE/docs/labs/one-model-end-to-end/instrument/Cargo.toml" \
  --features panel-writer --bin heldout-panel
check_rss "$OUT/generator-build.time.txt" 4194304
GEN=$GEN_TARGET/debug/heldout-panel
BIN=$BUILD/release/cpu-copy-suffix-fallback

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
  --cap 64 --seed 196 >"$OUT/reports/train-preflight.json"
check_rss "$OUT/reports/train-preflight.time.txt" 2097152
jq -e '
  .examples.reachable >= 200
  and .table.content.bos_transitions == 0
  and .table.content.transitions > 0
  and .table.changed_targets > 0
  and .changes.from_local.changed_steps > 0
  and .changes.from_shuffled.changed_steps > 0
  and .bar.suffix_decisions >= 100
' "$OUT/reports/train-preflight.json" >/dev/null

/usr/bin/time -v -o "$OUT/panel.time.txt" \
  timeout 900s "$GEN" out="$PANEL" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2773 protocol=2 \
  >"$OUT/panel.stdout.json"
check_rss "$OUT/panel.time.txt" 2097152
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2773 and .dialogue_protocol_version == 2
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
    --cap 64 --seed 196 >"$OUT/reports/$name.json"
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
    and .bar.suffix_decisions >= 100
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

Stop on source, artifact, tokenizer, panel, table, confidence, report,
output-root or repeat drift; fewer than 200 reachable replies; fewer than 100
confident fallback decisions; a vacuous shuffle; nondeterminism; wall time above
30 minutes; peak RSS at or above 2 GiB; or any need to edit serving, CUDA,
float-training, exporter or Claude-owned source. Once seed 2773 is read, do not
change local priority, depth cap, support, majority rule, shuffle or bar.

## Verdict

TEST FITNESS: FIT - M2 #2030

reviewer: suffix_fallback_fit_review_2
date: 2026-11-05
note: Corrected head 67b6ea7e2; accounting, local-priority common-state,
and end-to-end fallback/shuffle controls verified.

## D67 pre-run review

D67 PRE-RUN: GO - implementation `c2b2fda87`, sheet head `949e58dd6`;
source/input pins match; seed 2773 and the output root are absent; every timed
stage enforces its RSS limit; each fresh seed fails before repeats on
reachability, confident-fallback count, local/shuffle non-vacuity, BOS absence
and matched true/shuffled census; the extracted wrapper passes `bash -n`; and
the source remains isolated from Claude's scale work. Reviewer:
`suffix_fallback_go_review_2`, 2026-11-05.
