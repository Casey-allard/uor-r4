# Pre-registration: continuation closure

Run ID: `cpu-training-pilot-continuation-closure-2026-11-05`

## Status

COMPLETE - **REJECT**. Whole continuation was non-vacuous but tied tokenwise
content on exact replies and still exceeded the premature-stop ceiling.
Record: [deepseek-202](../../loop/tasks/cpu-training-pilot/deepseek-202/REPORT.md).

This is D84 CPU development research. It is exact-memory evidence, not
fresh-wording generalization, serving integration, useful chat or GPU
qualification.

## Question

Can a confident complete-reply continuation predicted from the full-history
suffix selected by deepseek-198's content arbiter reduce premature stopping by
making content and termination one decision?

deepseek-200 showed that full prompt-plus-reply terminal keys stopped at
exactly the same positions as reply-only keys. Every premature stop followed a
global-content decision. A separate stop table therefore did not change the
outcome.

## Fixed construction

For every TRAIN conversation:

1. Start with the prompt and append the normalized reply.
2. At the prompt boundary and after every generated reply token, consider
   full-history suffixes of depth four through 32.
3. Store the entire remaining normalized reply as the suffix value. The final
   reply token maps to the empty continuation.

At each generated decision:

1. Keep deepseek-198 local `Longest32To1` and confident full-sequence content
   arbitration fixed.
2. Select global content only when its suffix is strictly deeper than local;
   equal-depth ties stay local.
3. For the treatment, accept a whole continuation only when a confident
   closure exists at the same depth as the selected global content suffix.
4. Confidence requires at least two observations and a strict majority of
   identical complete continuations. A nonempty continuation must begin with
   the selected global content token; an empty terminal continuation is
   allowed.
5. Emit the whole continuation as one decision and stop at its natural end
   when it fits the cap and context. Otherwise continue tokenwise.

No dense matmul, softmax, squashing activation or float serving path is used.

## Arms

1. `source_closure`: treatment;
2. `tokenwise_content`: deepseek-198 selection plus the reply-only boundary;
3. `source_closure_shuffled`: equal-depth complete-continuation shuffle;
4. `closure_only`: confident closure without content/local arbitration, then
   writer;
5. `content_no_boundary`;
6. `local_reply_boundary`;
7. `writer_boundary`;
8. `content_reference_oracle`.

The shuffled closure preserves every suffix key, every key's observation count,
and the per-depth complete-continuation token multiset.

## Data and power

- table derivation and TRAIN preflight use panel `tt-2711` only;
- fresh unopened `cell=dev,dev`, seed 2776, protocol 2, 128 conversations;
- both public dual-c131 raw artifact seeds;
- 64-token cap and at least 200 reachable replies per artifact;
- at least 100 closure rows whose complete reply differs from tokenwise
  content or local reply output;
- at least one changed complete reply versus tokenwise, closure-only,
  no-boundary, shuffled and local;
- fresh-process repeats for both artifacts.

## Instrument gate

- source, artifact, tokenizer, input and generator pins match;
- output root and seed panel were absent before release;
- content has zero BOS transitions and nonzero transitions;
- closure has nonzero observations, continuation tokens and empty endpoints;
- shuffle preserves key counts, observation counts and continuation-token
  totals at every depth;
- treatment makes at least 100 behaviorally displaced closure rows versus
  tokenwise content or local output;
- treatment changes at least one complete reply versus tokenwise,
  closure-only, no-boundary, shuffled and local;
- every arm has at least 200 reachable replies;
- both artifact seeds and fresh-process repeats match after timing removal.

Any failure is an instrument, power or drift fault, not a mechanism negative.

## Bar

For both artifacts, `source_closure` must:

- keep premature BOS at or below four;
- produce at least `max(5, ceil(content_oracle_exact / 2))` exact replies;
- beat `tokenwise_content`, `closure_only` and
  `source_closure_shuffled` on exact replies by the paired two-sigma rule;
- keep correct body positions at or above `content_no_boundary` and
  `local_reply_boundary`, and at or above `closure_only`;
- make at least 100 behaviorally displaced closure rows versus tokenwise or
  local output;
- change at least one complete reply versus tokenwise, closure-only,
  no-boundary, shuffled and local;
- repeat exactly after timing fields are removed.

If the oracle has fewer than ten exact bodies, exact comparison is
`INSUFFICIENT_POWER`. `KEEP` requires the exact, premature-stop and body
conditions together. A non-vacuous failure is `REJECT`; a vacuous continuation
or shuffle is `CONTROL_FAULT`.

## Compute class

Local CPU development on the Ryzen 9 9950X. No GPU, pod, paid API, upload or
external model. Serving remains integer-only and unchanged.

## Base, source and inputs

- base: `d56427955`;
- implementation:
  `668fb23504b010767c88d502f73646f73a0e290b`;
- `continuation_closure.rs` SHA-256:
  `8c1b499b5b1db7d5236f89b9278d33ec29be0bd179943e0eb8a7764dfe4f31c4`;
- `cpu-copy-continuation-closure.rs` SHA-256:
  `fe6113182a2aae309ed4d6174f3ddde2bd873a4125f90655cc1cbfc4b0399c0c`;
- `suffix_memory.rs` SHA-256:
  `7cff7a71b61cc7c38b3b048124296e6cc64a77441e622e9f34b5ff6531656c37`;
- `reply_boundary.rs` SHA-256:
  `2138e1b05d833beb9f8b2ff3bb93024df1cb7cf1aa5663aad665a0384a8f119b`;
- `copy_key_reply.rs` SHA-256:
  `b754a4435a68d6d8f315815c05d88f3b4552edd8a1b659b0ab91dc53f4bf12ef`;
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

Fresh panel:

`local/deepseek/continuation-closure/2026-11-05/panels/dd-2776`

## Cost

- preparation and finite tests: 10 minutes;
- release and generator builds: 2 minutes;
- TRAIN preflight: 2 minutes;
- panel and six artifact runs: 3 minutes;
- evidence and delivery: 15 minutes;
- total bounded increment: 32 minutes;
- whole run deadline: 1800 seconds;
- evaluators: 32 Rayon workers and 2 GiB peak RSS;
- retained evidence: 32 MiB; fresh targets: 8 GiB.

## Steps

Run only after FIT and GO. Use a fresh output root and fail closed.

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-continuation-closure
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/continuation-closure/2026-11-05
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
PANEL=$OUT/panels/dd-2776
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/continuation-closure
GEN_TARGET=$SOURCE/target/deepseek/continuation-closure-generator
IMPL=668fb23504b010767c88d502f73646f73a0e290b
GEN_TREE=869e65db6d0bb997a5f1094ab0de4efa1ac10c25
RUN_DEADLINE=$((SECONDS + 1800))

test ! -e "$OUT"
test ! -e "$PANEL"
git -C "$SOURCE" diff --quiet "$IMPL" -- crates/uor-r4-cpu-train
git -C "$SOURCE" diff --quiet "$IMPL" -- docs/labs/one-model-end-to-end/instrument
test "$(git -C "$SOURCE" rev-parse "$IMPL:docs/labs/one-model-end-to-end/instrument")" = "$GEN_TREE"
printf '%s  %s\n' \
  a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 "$TOKENIZER" \
  a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$SEED1" \
  da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$SEED2" \
  140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91 "$TRAIN/tokens.u16" \
  d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad "$TRAIN/response_mask.u8" \
  d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5 "$TRAIN/manifest.json" \
  8c1b499b5b1db7d5236f89b9278d33ec29be0bd179943e0eb8a7764dfe4f31c4 "$SOURCE/crates/uor-r4-cpu-train/src/continuation_closure.rs" \
  fe6113182a2aae309ed4d6174f3ddde2bd873a4125f90655cc1cbfc4b0399c0c "$SOURCE/crates/uor-r4-cpu-train/src/bin/cpu-copy-continuation-closure.rs" \
  7cff7a71b61cc7c38b3b048124296e6cc64a77441e622e9f34b5ff6531656c37 "$SOURCE/crates/uor-r4-cpu-train/src/suffix_memory.rs" \
  2138e1b05d833beb9f8b2ff3bb93024df1cb7cf1aa5663aad665a0384a8f119b "$SOURCE/crates/uor-r4-cpu-train/src/reply_boundary.rs" \
  b754a4435a68d6d8f315815c05d88f3b4552edd8a1b659b0ab91dc53f4bf12ef "$SOURCE/crates/uor-r4-cpu-train/src/copy_key_reply.rs" |
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
finish() {
  local status=$?
  trap - EXIT
  if test -d "$OUT/reports"; then
    find "$OUT/reports" -type f -name '*.json' -print0 |
      xargs -0 -r gzip -9 || status=$?
  fi
  seal || status=$?
  if test "$(du -sb "$OUT" | cut -f1)" -gt 33554432; then
    echo "retained evidence exceeds 32 MiB" >&2
    status=1
  fi
  exit "$status"
}
trap finish EXIT
run_limited() {
  local remaining=$((RUN_DEADLINE - SECONDS))
  test "$remaining" -gt 0
  timeout "${remaining}s" "$@"
}
check_rss() {
  local log=$1
  local limit_kib=$2
  local rss
  rss=$(awk '/Maximum resident set size/ { print $NF }' "$log")
  test -n "$rss"
  test "$rss" -le "$limit_kib"
}

cd "$SOURCE"
run_limited /usr/bin/time -v -o "$OUT/build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" \
  cargo build --locked --release -p uor-r4-cpu-train --bin cpu-copy-continuation-closure
check_rss "$OUT/build.time.txt" 4194304
run_limited /usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GEN_TARGET" \
  cargo build --locked --manifest-path "$SOURCE/docs/labs/one-model-end-to-end/instrument/Cargo.toml" \
  --features panel-writer --bin heldout-panel
check_rss "$OUT/generator-build.time.txt" 4194304
GEN=$GEN_TARGET/debug/heldout-panel
BIN=$BUILD/release/cpu-copy-continuation-closure
TRAIN_TOKENS_SHA=$(sha256sum "$TRAIN/tokens.u16" | cut -d ' ' -f1)
TRAIN_MASK_SHA=$(sha256sum "$TRAIN/response_mask.u8" | cut -d ' ' -f1)
TRAIN_MANIFEST_SHA=$(sha256sum "$TRAIN/manifest.json" | cut -d ' ' -f1)

run_limited /usr/bin/time -v -o "$OUT/reports/train-preflight.time.txt" \
  env RAYON_NUM_THREADS=32 "$BIN" \
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
  --cap 64 --seed 2776 >"$OUT/reports/train-preflight.json"
check_rss "$OUT/reports/train-preflight.time.txt" 2097152
jq -e '
  .examples.reachable >= 200
  and .tables.content.bos_transitions == 0
  and .tables.content.transitions > 0
  and .tables.closure.observations > 0
  and .tables.closure.continuation_tokens > 0
  and .tables.closure.empty_continuations > 0
  and .tables.closure.suffixes_by_depth == .tables.shuffled_closure.suffixes_by_depth
  and .tables.closure.observations_by_depth == .tables.shuffled_closure.observations_by_depth
  and .tables.closure.continuation_tokens_by_depth
    == .tables.shuffled_closure.continuation_tokens_by_depth
  and .tables.changed_continuations > 0
  and .changes.vs_tokenwise > 0
  and .changes.vs_closure_only > 0
  and .changes.vs_no_boundary > 0
  and .changes.vs_shuffled > 0
  and .changes.vs_local > 0
  and .changes.closure_rows_vs_tokenwise_or_local >= 100
  and .arms.source_closure.reading.closure_decisions > 0
  and .bar.control_non_vacuous
' "$OUT/reports/train-preflight.json" >/dev/null

run_limited /usr/bin/time -v -o "$OUT/panel.time.txt" \
  "$GEN" out="$PANEL" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2776 protocol=2 >"$OUT/panel.stdout.json"
check_rss "$OUT/panel.time.txt" 2097152
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2776 and .dialogue_protocol_version == 2
  and .cell.value == "dev" and .cell.phrasing == "dev"
  and .conversations == 128 and .world == "m-world-v2"
  and .tokenizer.sha256 == $tokenizer_sha
' "$PANEL/manifest.json" >/dev/null
PANEL_TOKENS_SHA=$(sha256sum "$PANEL/tokens.u16" | cut -d ' ' -f1)
PANEL_MASK_SHA=$(sha256sum "$PANEL/response_mask.u8" | cut -d ' ' -f1)
PANEL_MANIFEST_SHA=$(sha256sum "$PANEL/manifest.json" | cut -d ' ' -f1)

run() {
  name=$1 artifact=$2 expected=$3
  run_limited /usr/bin/time -v -o "$OUT/reports/$name.time.txt" \
    env RAYON_NUM_THREADS=32 "$BIN" \
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
    --cap 64 --seed 2776 >"$OUT/reports/$name.json"
  check_rss "$OUT/reports/$name.time.txt" 2097152
}
run seed1 "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run seed2 "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
for report in "$OUT/reports/seed1.json" "$OUT/reports/seed2.json"; do
  jq -e --arg panel_tokens "$PANEL_TOKENS_SHA" \
    --arg panel_mask "$PANEL_MASK_SHA" --arg panel_manifest "$PANEL_MANIFEST_SHA" '
    .inputs.panel.tokens_sha256 == $panel_tokens
    and .inputs.panel.mask_sha256 == $panel_mask
    and .inputs.panel.manifest_sha256 == $panel_manifest
    and .examples.reachable >= 200
    and .tables.content.bos_transitions == 0
    and .tables.closure.suffixes_by_depth == .tables.shuffled_closure.suffixes_by_depth
    and .tables.closure.observations_by_depth == .tables.shuffled_closure.observations_by_depth
    and .tables.closure.continuation_tokens_by_depth
      == .tables.shuffled_closure.continuation_tokens_by_depth
    and .tables.changed_continuations > 0
    and .changes.vs_tokenwise > 0
    and .changes.vs_closure_only > 0
    and .changes.vs_no_boundary > 0
    and .changes.vs_shuffled > 0
    and .changes.vs_local > 0
    and .changes.closure_rows_vs_tokenwise_or_local >= 100
    and .bar.control_non_vacuous
    and (.arms | length) == 8
    and all(.arms[]; .reading.reference_reachable >= 200)
  ' "$report" >/dev/null
done
run seed1-repeat "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run seed2-repeat "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1

normalize() {
  jq -S 'walk(if type == "object"
    then del(.wall_elapsed_us, .generated_tokens_per_second)
    else . end)' "$1"
}
cmp <(normalize "$OUT/reports/seed1.json") <(normalize "$OUT/reports/seed1-repeat.json")
cmp <(normalize "$OUT/reports/seed2.json") <(normalize "$OUT/reports/seed2-repeat.json")
find "$OUT/reports" -type f -name '*.json' -print0 | xargs -0 -r gzip -9
```

## Stop rule

Stop on source, artifact, panel, table, continuation, parity, determinism,
seed or output-root drift; fewer than 200 reachable replies; fewer than 100
behaviorally changed closure rows; a vacuous shuffle; wall time above 30
minutes; peak RSS above 2 GiB; retained evidence above 32 MiB; or any need to
change serving, CUDA, float-training, exporter or Claude-owned source. Once
seed 2776 is read, do not change closure depth, confidence, shuffle, selectors
or bars.

## Result

- fresh panel seed 2776: 128 `dev,dev` protocol-2 conversations;
- 449 reference-reachable replies per artifact;
- table: 80,728 suffixes, 111,656 observations and 715,428 continuation
  tokens;
- shuffle: 79,384 changed continuation assignments with preserved census;
- source closure: **17/17** exact, **71/64** premature, **682/678** body;
- tokenwise content: **17/17** exact, **74/98** premature, **682/678** body;
- closure only: **12/12** exact, **78/19** premature, **238/243** body;
- shuffled closure: **0/0** exact, **18/18** premature, **602/598** body;
- content no-boundary: **0/0** exact, **682/678** body;
- content oracle: **30/30** exact, **682/678** body;
- 187 closure rows were behaviorally displaced versus tokenwise or local;
- 8/39 complete replies changed versus tokenwise;
- all other-control change gates passed;
- repeats matched after timing removal;
- retained evidence is **10,045,094** bytes and sealed by `SHA256SUMS`
  SHA-256
  `899bbca44564d63705496103846096ec74f2698a2aaf859865afe9e7372d6669`.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 continuation closure

reviewer: `continuation_closure_fit_414_final`
date: 2026-11-05
head: `414ec5f0aa81c51a522143486665d6af1f07e65c`
note: Whole-continuation confidence, source-depth and first-token binding,
local/content isolation, matched displacement thresholds, cap/context
fallback, stop-marker evidence, failure-path compression/sealing, repeats and
resource gates verified.

## D67 pre-run review

D67 PRE-RUN: GO - registration head
`607f67655620987238d02e04cdef8f7c406e083e`.

reviewer: `continuation_closure_go`
date: 2026-11-05
note: Source remained at FIT pin `668fb2350`; continuation, control, power,
repeat, deadline, RSS, failure-path sealing and isolation gates were verified.
Seed 2776 and the output root were absent.
