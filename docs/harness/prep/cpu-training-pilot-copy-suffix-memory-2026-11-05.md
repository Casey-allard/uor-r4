# Pre-registration: trained reply-suffix continuation memory

Run ID: `cpu-training-pilot-copy-suffix-memory-2026-11-05`

## Status

COMPLETE - REJECT. The fresh seed-2772 panel cleared the exact-body power floor
with 14 oracle-complete replies. Trained suffix-32 reached only 57 and 86
correct body positions on artifact seeds 1 and 2, against 514 for local
longest-32, and made 14 and 227 premature stops against the four-stop ceiling.
The table and shuffled control were active and deterministic. This closes the
tested global suffix plurality plus full-history BOS construction. Record:
[deepseek-193](../../loop/tasks/cpu-training-pilot/deepseek-193/REPORT.md).

## Question

Can a discrete suffix-to-next-token memory trained only from TRAIN replies
improve complete-reply body reconstruction over the current local
`Longest32To1` exact-copy selector?

At each generated step, the treatment searches suffix depths 32 down to one in
the TRAIN reply table and uses the first known suffix's most frequent successor.
Decoder BOS is an ordinary terminal target. Equal successor counts tie to the
smaller token id. An unknown suffix falls back to the artifact writer and
deepseek-191's true reply-boundary memory.

This is exact-memory recurrence evidence under D84. It is not fresh-wording
generalization, useful chat, serving integration or GPU qualification.

## Why this is new

- deepseek-191 exercised stopping, but only seven fresh bodies were
  oracle-complete, so body reconstruction is the measured next limitation.
- `Longest32To1` copies only from the current stream and retains the latest
  source occurrence at the deepest exact depth. It does not aggregate a global
  TRAIN suffix distribution.
- Existing content readouts operate on generated geometric states and can
  arbitrate only a narrow candidate set. Several lose to `Longest32To1` or lack
  training power.
- No prior CPU unit trains a global discrete reply-suffix table and compares
  longest-context plurality with a successor-multiset-preserving shuffle, a
  depth-one table, local exact copy and the writer.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model. The result cannot substitute for a GPU qualification
bar. Serving remains integer-only and unchanged.

## Base artifact and source

- base: `1f1e3a756`;
- implementation commit: `e3970c1e8`;
- `suffix_memory.rs` SHA-256:
  `11a252a46e94cb15f5dc809720926f019ad9a006a380efe6916f8226a91924a1`;
- `cpu-copy-suffix-memory.rs` SHA-256:
  `dc78a4080a8a7e8ee7bac83e57e813225f6817fe9babcffdc3c085475e04997b`;
- panel generator tree:
  `869e65db6d0bb997a5f1094ab0de4efa1ac10c25`;
- tokenizer SHA-256:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- public FBS seed 1 SHA-256:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- public FBS seed 2 SHA-256:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- host: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

The artifacts are unchanged public dual-c131 raw FBS control models. The
experiment changes no model parameter or serving artifact.

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

`/home/callard/projects/codex-playground/uor-r4/local/deepseek/copy-suffix-memory/2026-11-05/panels/dd-2772`

- generated only after FIT and GO;
- `cell=dev,dev`, seed 2772, protocol 2, 128 conversations;
- the output root and panel directory are absent at registration;
- no evaluation reply enters table derivation.

The initial registration named seed 2771. The first fresh review found that
deepseek's completed `integer-long-context` unit had already used that seed, so
the panel identity was corrected to unused seed 2772 before FIT or compute.

## Fixed construction

For every normalized TRAIN reply:

1. For each reply position, count the transition from every suffix ending
   there, at lengths `1..=min(32, position + 1)`, to the next reply token.
2. At the final position, count the same suffixes to decoder BOS.
3. Preserve exact integer counts.

At generation:

1. inspect only generated reply tokens, never the prompt;
2. search suffix depths from 32 down to one;
3. at the first known depth, select the most frequent successor, ties to the
   smaller token id;
4. stop on decoder BOS, otherwise feed the selected token through the exact CPU
   FBS forward;
5. fall back to the writer and true boundary when no suffix is known.

No dense matmul, softmax, squashing activation or float serving path is used.

## Arms

For each artifact seed:

1. `trained_suffix_32`;
2. `trained_suffix_depth_1`;
3. `shuffled_suffix_32`;
4. `local_longest_32_boundary`;
5. `writer_boundary`;
6. `reference_length_oracle`, reported only as the exact-body ceiling.

The shuffle redistributes successor labels among equal-depth occurrences. It
preserves each suffix's observation count and each depth's complete successor
multiset, while changing key-to-successor associations.

## Instrument gate

Before any fresh result is read:

- implementation and every input hash match;
- the table contains observations at every depth represented by TRAIN;
- the true and shuffled tables preserve depth support and target multisets;
- the shuffle changes at least one association;
- treatment changes at least one decision from local longest and from shuffled;
- each artifact has at least 200 reference-reachable replies and at least 100
  known-suffix treatment decisions;
- a fresh process reproduces every non-timing field and both digests.

Any failure is an instrument or power fault, not a mechanism negative.

## Minimum test power

- two independently trained public artifact seeds;
- one fresh DEV panel with 128 conversations and at least 200 reachable replies
  per artifact seed;
- every reachable prompt/reply is paired by row;
- at least one changed treatment decision against local longest and shuffled
  per artifact seed;
- fresh-process repeats of both artifact seeds;
- the complete fresh sequence is bounded to 30 minutes and 2 GiB peak RSS.

This meets D22's at least 200 fresh rows. The model table is trained from
TRAIN; no evaluation row is used for fitting or control construction.

## Bar

For both artifact seeds on the fresh panel, `trained_suffix_32` must beat each
of `local_longest_32_boundary`, `trained_suffix_depth_1`,
`shuffled_suffix_32` and `writer_boundary` on correct body positions by:

```text
candidate sum > control sum
candidate-only >= max(2, ceil(2 * sqrt(control-only)))
```

It must also keep premature BOS at or below four per panel and reproduce
exactly.

Exact complete replies are the primary capability outcome. If the
reference-length oracle has fewer than ten exact bodies, the exact comparison
is `INSUFFICIENT_POWER`; a body-position pass is then retained only as
`BODY_POSITIVE_INSUFFICIENT_COMPLETION`. With at least ten oracle-exact bodies,
the treatment must also beat local longest on exact complete replies by the
paired rule.

## R12 authority

1. **What is measured:** paired complete replies under six fixed selectors,
   scored on exact bodies, BOS completion, premature BOS and correct body
   positions.
2. **Whether it is computable:** yes. The exact CPU FBS curve, public
   artifacts, panel tokenizer and deterministic token/mask panels are present.
3. **Whether the instrument can beat the bar:** yes in principle. TRAIN suffix
   counts supply continuations not visible in the current prompt stream, and
   the depth-one and successor-shuffle controls isolate context length and
   learned association.
4. **Whether it should run now:** yes. Body reconstruction is the recorded
   blocker after deepseek-191, and this recurrence is materially different from
   the closed geometric-state readouts and one-step lookahead configurations.

## Cost

- preparation and finite checks: about 60 s;
- release build: about 30 s;
- generator build and one panel draw: about 30 s;
- TRAIN preflight: about 15 s;
- four artifact runs at two seeds plus two repeats: about 60 s;
- evidence and delivery: about 10 minutes.
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
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-suffix-memory
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-suffix-memory/2026-11-05
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
PANEL=$OUT/panels/dd-2772
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/copy-suffix-memory
GEN_TARGET=$SOURCE/target/deepseek/copy-suffix-memory-generator
IMPL=e3970c1e8
GEN_TREE=869e65db6d0bb997a5f1094ab0de4efa1ac10c25

test ! -e "$OUT"
test ! -e "$PANEL"
git -C "$SOURCE" diff --quiet "$IMPL" -- crates/uor-r4-cpu-train
test "$(git -C "$SOURCE" rev-parse 1f1e3a756:docs/labs/one-model-end-to-end/instrument)" = "$GEN_TREE"
printf '%s  %s\n' \
  a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 "$TOKENIZER" \
  a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$SEED1" \
  da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$SEED2" \
  140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91 "$TRAIN/tokens.u16" \
  d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad "$TRAIN/response_mask.u8" \
  d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5 "$TRAIN/manifest.json" |
  sha256sum -c -
printf '%s  %s\n' \
  11a252a46e94cb15f5dc809720926f019ad9a006a380efe6916f8226a91924a1 \
  "$SOURCE/crates/uor-r4-cpu-train/src/suffix_memory.rs" \
  dc78a4080a8a7e8ee7bac83e57e813225f6817fe9babcffdc3c085475e04997b \
  "$SOURCE/crates/uor-r4-cpu-train/src/bin/cpu-copy-suffix-memory.rs" |
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
  cargo build --locked --release -p uor-r4-cpu-train --bin cpu-copy-suffix-memory
/usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GEN_TARGET" \
  cargo build --locked \
  --manifest-path "$SOURCE/docs/labs/one-model-end-to-end/instrument/Cargo.toml" \
  --features panel-writer --bin heldout-panel
GEN=$GEN_TARGET/debug/heldout-panel

TRAIN_TOKENS_SHA=$(sha256sum "$TRAIN/tokens.u16" | cut -d ' ' -f1)
TRAIN_MASK_SHA=$(sha256sum "$TRAIN/response_mask.u8" | cut -d ' ' -f1)
TRAIN_MANIFEST_SHA=$(sha256sum "$TRAIN/manifest.json" | cut -d ' ' -f1)
BIN=$BUILD/release/cpu-copy-suffix-memory

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
  --cap 64 --seed 193 >"$OUT/reports/train-preflight.json"
jq -e '
  .examples.reachable >= 200
  and .table.true.transitions > 0
  and .table.true.bos_transitions > 0
  and .table.changed_targets > 0
  and .changes.from_local.changed_steps > 0
  and .changes.from_shuffled.changed_steps > 0
  and .bar.suffix_decisions >= 100
' "$OUT/reports/train-preflight.json" >/dev/null

/usr/bin/time -v -o "$OUT/panel.time.txt" \
  timeout 900s "$GEN" out="$PANEL" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2772 protocol=2 \
  >"$OUT/panel.stdout.json"
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2772 and .dialogue_protocol_version == 2
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
    --cap 64 --seed 193 >"$OUT/reports/$name.json"
}
run seed1 "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run seed2 "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
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

Stop on source, artifact, tokenizer, panel, table, shuffle, report, output-root
or repeat drift; fewer than 200 reachable replies; fewer than 100 known-suffix
treatment decisions; a vacuous shuffle; nondeterminism; wall time above 30
minutes; peak RSS at or above 2 GiB; or any need to edit serving, CUDA,
float-training, exporter or Claude-owned source. Once seed 2772 is read, do not
change the depth cap, target tie, fallback, shuffle or bar.

## D67 pre-run review

D67 PRE-RUN: GO - seed 2772 and its output root are absent; source, panel,
artifact and generator pins match; the CPU source is bounded under D84; the
TRAIN preflight precedes fresh-panel generation; and both pass and fail change
the next decision. Reviewer: `suffix_memory_go_review`, 2026-11-05.

## Verdict

TEST FITNESS: FIT - M2 #2030 spiralcore-fbs

reviewer: suffix_memory_fit_review_2
date: 2026-11-05
note: Fresh review found seed 2772 unused; end-to-end
override/BOS/fallback/shuffle controls present; per-depth target multiset and
reachable-only decision accounting verified.
