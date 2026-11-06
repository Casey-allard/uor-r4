# Pre-registration: full-sequence reply boundary, attempt 2

Run ID: `cpu-training-pilot-full-sequence-boundary-2026-11-05`

## Status

COMPLETE - `CONTROL_FAULT`. Attempt 2 ran the fresh seed-2775 panel on both
artifacts. The full-history and reply-only boundaries made the same observable
stop decisions on both, so the registered per-artifact non-vacuity gate
stopped the sequence before repeats. Record:
[deepseek-200](../../loop/tasks/cpu-training-pilot/deepseek-200/REPORT.md).

Attempt 1 built the pinned source and generator, then stopped at TRAIN
preflight before generating seed 2775. Full and reply-only boundaries made the
same 241 stops on exact TRAIN replay, so the preflight observed zero
full-versus-reply stop changes. The full boundary was active, differed from
the shuffled boundary at 226 reachable rows, and had identical per-depth
terminal and nonterminal censuses with shuffled. Exact replay equality between
the two learned boundaries is expected because both replay a known reference
endpoint; requiring a difference there made the preflight vacuous.

Attempt 2 changes only the preflight gate: it retains full activity,
full-versus-shuffled change, reachability and table invariants, but leaves
full-versus-reply non-vacuity as a per-artifact evaluation gate. The seed,
panel shape, source, selectors, arms, artifact seeds and scientific bar are
unchanged. No seed-2775 panel or result exists.

This is D84 CPU development research in `uor-r4-cpu-train`. It is exact-memory
recall evidence, not fresh-phrasing generalization, useful chat, serving
integration or GPU qualification.

## Question

Can a boundary trained over the same full prompt-plus-reply history as
deepseek-198's content selector keep its exact and body-position gains while
preventing premature BOS decisions?

deepseek-198 produced 17/17 exact replies versus 0/0 for local longest copy and
847/844 correct body positions versus 578/578, but its inherited reply-only
boundary made 85/104 premature stops against a four-stop ceiling. This unit
holds that content selector fixed and changes only how the full-history stop
is learned.

## Why this is new

- The 198 content table reads prompt-plus-reply histories.
- The inherited 191 boundary reads generated reply suffixes only and was
  trained on the old local-copy trajectory.
- This boundary assigns nonterminal observations to every internal reply
  position and terminal observations only to the final reply position, keyed
  by the full prompt-plus-generated stream at depths four through 32.
- At generation, the longest known full-stream suffix decides. Any observed
  nonterminal continuation under that suffix blocks BOS.
- A matched shuffle moves terminal events among suffixes of the same depth
  while preserving every suffix key, every nonterminal count, and every
  per-depth terminal and nonterminal total.
- Attempt 1 showed that exact TRAIN replay can make full-history and
  reply-only stopping behavior identical. Attempt 2 records instead of
  rejects that expected identity; the treatment still must differ from both
  reply-only and shuffled on each fresh evaluation artifact.

No dense matmul, softmax, squashing activation or float serving path is used.

## Compute class

Local CPU development research on the existing Ryzen 9 9950X host. No GPU,
pod, paid API, upload or external model. GPU remains the qualification
authority. Serving, CUDA, float training, exporters and Claude-owned paths are
unchanged.

## Base, source and inputs

- base: `5ed64d3ca`;
- implementation commit:
  `89ed3d2138143f520ddc22e80c871e5d7f522484`;
- `Cargo.toml` SHA-256:
  `92386d60bd97535fa65d1e702ab106c206ce4dfff79917d7a92be72b450d4f3f`;
- `lib.rs` SHA-256:
  `2de49bbe965a16e3c96832e224ba4ec77512b5f0cf769755562e24cf9850aff3`;
- `full_sequence_boundary.rs` SHA-256:
  `1751a85ae21f5f6ba74fd4918aad714726fe570631296720a5cb0cff69e7be1a`;
- `cpu-copy-full-boundary.rs` SHA-256:
  `74271a97fb09ad17e2c0fd07bc1b66515399d7578bf2e071b0741377b3bf4a0f`;
- `suffix_memory.rs` SHA-256:
  `7cff7a71b61cc7c38b3b048124296e6cc64a77441e622e9f34b5ff6531656c37`;
- `reply_boundary.rs` SHA-256:
  `2138e1b05d833beb9f8b2ff3bb93024df1cb7cf1aa5663aad665a0384a8f119b`;
- `copy_key_reply.rs` SHA-256:
  `b754a4435a68d6d8f315815c05d88f3b4552edd8a1b659b0ab91dc53f4bf12ef`;
- panel generator tree:
  `869e65db6d0bb997a5f1094ab0de4efa1ac10c25`;
- tokenizer SHA-256:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- public FBS seed 1 SHA-256:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- public FBS seed 2 SHA-256:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- host: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

The two public artifacts are unchanged raw dual-c131 control models.

## Data split

Training and all table derivations:

`/home/callard/projects/codex-playground/uor-r4/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711`

- tokens SHA-256:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- response-mask SHA-256:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- manifest SHA-256:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`;
- `cell=train,train`, seed 2711, 128 conversations.

Fresh evaluation panel:

`/home/callard/projects/codex-playground/uor-r4/local/deepseek/full-sequence-boundary/2026-11-05-attempt-2/panels/dd-2775`

- `cell=dev,dev`, seed 2775, protocol 2, 128 conversations;
- attempt-2 output root and panel directory are absent at registration;
- seed 2775 remained unused across: attempt 1 did not open the panel;
- generated only after FIT and GO;
- no evaluation row enters the content, boundary or shuffled tables.

## Fixed construction

For every TRAIN conversation:

1. Start history with prompt tokens and append normalized reply tokens.
2. At every internal reply position, count each full-history suffix at depths
   four through 32 as nonterminal.
3. At the final reply position, count those suffixes as terminal.
4. Add no decoder BOS successor.

At generation:

1. inspect the complete prompt-plus-generated stream before each next token;
2. choose the longest known suffix at depth four through 32;
3. emit BOS only when that suffix has terminal observations and zero
   nonterminal observations;
4. otherwise continue with deepseek-198's fixed content/local selector.

Content selection remains:

- full prompt-plus-reply suffix memory;
- strict-majority confidence with at least two observations;
- global replaces local only when its suffix is deeper;
- ties stay local;
- writer is used when neither content nor local has a candidate.

## Arms

Every artifact seed evaluates all eight arms:

1. `content_full_boundary`: treatment;
2. `content_reply_boundary`: 198 content with 191 reply-only stopping;
3. `content_shuffled_full_boundary`: 198 content fixed, terminal associations
   shuffled only in the full-history boundary;
4. `content_no_boundary`;
5. `local_full_boundary`;
6. `local_reply_boundary`;
7. `writer_full_boundary`;
8. `content_reference_oracle`: content body stopped at reference length.

The two no-boundary arms isolate whether an apparent gain comes from stopping.
Local and writer arms isolate content selection. The reply-boundary and
shuffled-full arms isolate the new history-keyed terminal signal. The shuffled
arm does not reshuffle content targets.

## Instrument gate

Before reading a result, all of these must hold:

- source, artifact, panel, manifest and tokenizer pins match;
- the fresh output root and panel were absent before the run;
- TRAIN preflight has at least 200 reachable replies;
- content has zero BOS transitions and nonzero transitions;
- full boundary has nonzero terminal and nonterminal occurrences;
- terminal-only and ambiguous full-boundary suffixes both exist;
- the shuffle preserves key sets, nonterminal counts, and per-depth suffix,
  terminal, and nonterminal totals while changing terminal assignments;
- treatment makes at least one full-boundary stop;
- TRAIN preflight changes at least one observable stop from shuffled;
- each fresh evaluation artifact changes at least one observable stop from
  reply-only and one from shuffled;
- all eight arms have at least 200 reachable replies;
- both artifact seeds and fresh-process repeats match after timing removal.

Any failure is an instrument, power or drift fault, not a mechanism negative.

## Minimum test power

- two independently trained public artifact seeds;
- one fresh DEV panel with 128 conversations;
- at least 200 reference-reachable replies per artifact seed;
- at least one full stop and nonzero full-vs-reply and full-vs-shuffled change
  per fresh artifact seed;
- at least ten content-oracle exact bodies for the paired exact comparison;
- fresh-process repeats of both artifact seeds;
- 64-token cap, 32 Rayon workers, 30-minute wall cap and 2 GiB peak RSS.

This meets D22's fresh-row minimum. No evaluation row is used for fitting or
control construction.

## Bar

For both artifact seeds, treatment must:

- make at most four premature BOS stops;
- produce at least `max(5, ceil(content_oracle_exact / 2))` exact replies;
- beat `content_reply_boundary` and `content_shuffled_full_boundary` on exact
  complete replies by the paired two-sigma rule:

```text
candidate sum > control sum
candidate-only >= max(2, ceil(2 * sqrt(control-only)))
```

- keep correct body positions at or above `content_no_boundary` and
  `local_reply_boundary`;
- make at least one boundary stop;
- change at least one observable stop from reply-only and one from shuffled;
- repeat exactly after timing fields are removed.

If the content oracle has fewer than ten exact bodies, the run is
`INSUFFICIENT_POWER`; stopping state is still reported. `KEEP` requires every
exact, premature-stop and body condition. A failed condition is `REJECT`.
Vacuous controls are `CONTROL_FAULT`.

No threshold, arm, confidence rule, boundary depth or exact comparison changes
after opening seed 2775.

## What each outcome decides

- **KEEP:** full-history termination repairs the content trajectory and is the
  configuration carried into the next fresh-transfer check.
- **REJECT with exact power:** candidate termination is not reliable; the
  report separates stopping-time error, content loss and table non-vacuity and
  names the next configuration.
- **INSUFFICIENT_POWER:** no mechanism claim is made; report the observed stop
  state and require a separately registered powered panel before another claim.
- **CONTROL_FAULT:** stop before interpreting behavior; repair table, arm
  isolation or gates.

## Attempt 1 record

- `TEST FITNESS: FIT` at implementation head `ce180de32`, reviewer
  `full_boundary_ce180_fitness`; source `APPROVE` from
  `full_boundary_ce180_source`.
- `D67 PRE-RUN: GO` at registration head `92220d7d1`, reviewer
  `full_boundary_d67_go`.
- execution stopped at TRAIN preflight with outcome `CONTROL_FAULT`;
- 453 reachable TRAIN replies;
- treatment: 241 full-boundary stops, 56 premature BOS, 95 exact replies,
  1,454 correct body positions;
- reply-only: the same 241 stops, 56 premature BOS, 95 exact replies and 1,454
  body positions;
- full-versus-reply observable stop changes: 0;
- full-versus-shuffled observable stop changes: 226;
- shuffled terminal assignments: 16,470;
- full and shuffled per-depth suffix, terminal and nonterminal censuses match;
- attempt-1 output root
  `local/deepseek/full-sequence-boundary/2026-11-05` is sealed by
  `SHA256SUMS` SHA-256
  `9a01e2e30db0c4f21b47ef6f6e06e7bd9725925bff7898112be494b074710292`;
- raw preflight report SHA-256 before compaction:
  `3a6af893939ea43240f733fc99a742e61f9b24db5907466f469db5f4e0f29dc6`;
- gzip-compressed preflight report SHA-256:
  `29f8df161bcc21475d7f7a3e15312a118c0f441549d99381bf4252ccf9920469`;
- seed 2775 and `panels/dd-2775` were not created.

## Attempt 2 invocation fault

- source, input, artifact, tokenizer and generator pins passed;
- `/usr/bin/time` was invoked outside the shell helper and could not execute
  `run_limited`; the command exited 127 before any build, preflight or panel;
- no seed-2775 file was created;
- the empty scaffold was preserved at
  `local/deepseek/full-sequence-boundary/2026-11-05-attempt-2-invocation-fault`,
  whose `SHA256SUMS` has SHA-256
  `b4a9e236e87f26d2b5ee2bf196f788259f13661fe319f42bf5491d8bc8f89674`;
- attempt 2 now invokes `run_limited /usr/bin/time ...`, so the global timeout
  wraps the measured command.

## R12 authority

1. **What is measured:** paired complete replies under fixed selectors.
2. **Whether it is computable:** yes. Exact CPU FBS, public artifacts,
   tokenizer, TRAIN panel and deterministic panel generator are present.
3. **Whether the instrument can beat the bar:** yes in principle. The treatment
   separates terminal and internal observations on the same full-history
   surface used by the content selector, with reply-only and no-boundary
   controls.
4. **Whether it should run now:** yes, after fresh FIT and D67 GO. It directly
   addresses the only combined-bar failure in deepseek-198 without changing
   its positive content mechanism.

## Cost

- source checks and finite tests: 5 minutes;
- release build: 1 minute;
- generator build and fresh-panel draw: 2 minutes;
- TRAIN preflight: 1 minute;
- four fresh artifact runs and two repeats: 2 minutes;
- sealing, evidence and delivery: 10 minutes;
- total bounded increment: 21 minutes;
- build jobs: 16; build RSS cap: 4 GiB;
- whole run: one 1800 s global deadline, with every external stage limited to
  the remaining time;
- each evaluator run: 32 Rayon workers and 2 GiB peak RSS;
- retained evidence cap: 32 MiB; fresh target cap: 8 GiB;
- placement: local Ryzen 9 9950X under D84; no pod or GPU.

## Steps

Run only after FIT and D67 GO. Use a fresh output root and fail closed.

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-full-sequence-boundary
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/full-sequence-boundary/2026-11-05-attempt-2
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
PANEL=$OUT/panels/dd-2775
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/full-sequence-boundary-attempt-2
GEN_TARGET=$SOURCE/target/deepseek/full-sequence-boundary-generator-attempt-2
IMPL=89ed3d2138143f520ddc22e80c871e5d7f522484
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
  92386d60bd97535fa65d1e702ab106c206ce4dfff79917d7a92be72b450d4f3f "$SOURCE/crates/uor-r4-cpu-train/Cargo.toml" \
  2de49bbe965a16e3c96832e224ba4ec77512b5f0cf769755562e24cf9850aff3 "$SOURCE/crates/uor-r4-cpu-train/src/lib.rs" \
  1751a85ae21f5f6ba74fd4918aad714726fe570631296720a5cb0cff69e7be1a "$SOURCE/crates/uor-r4-cpu-train/src/full_sequence_boundary.rs" \
  74271a97fb09ad17e2c0fd07bc1b66515399d7578bf2e071b0741377b3bf4a0f "$SOURCE/crates/uor-r4-cpu-train/src/bin/cpu-copy-full-boundary.rs" \
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
trap 'status=$?; trap - EXIT; if test -d "$OUT/reports"; then find "$OUT/reports" -type f -name "*.json" -print0 | xargs -0 -r gzip -9 || status=$?; fi; seal || status=$?; exit "$status"' EXIT
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
  cargo build --locked --release -p uor-r4-cpu-train --bin cpu-copy-full-boundary
check_rss "$OUT/build.time.txt" 4194304
run_limited /usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GEN_TARGET" \
  cargo build --locked \
  --manifest-path "$SOURCE/docs/labs/one-model-end-to-end/instrument/Cargo.toml" \
  --features panel-writer --bin heldout-panel
check_rss "$OUT/generator-build.time.txt" 4194304
GEN=$GEN_TARGET/debug/heldout-panel
BIN=$BUILD/release/cpu-copy-full-boundary

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
  --cap 64 --seed 2775 >"$OUT/reports/train-preflight.json"
check_rss "$OUT/reports/train-preflight.time.txt" 2097152
jq -e '
  .examples.reachable >= 200
  and .tables.content.bos_transitions == 0
  and .tables.content.transitions > 0
  and .tables.full_boundary.terminal_occurrences > 0
  and .tables.full_boundary.nonterminal_occurrences > 0
  and .tables.full_boundary.terminal_only_suffixes > 0
  and .tables.full_boundary.ambiguous_suffixes > 0
  and .tables.full_boundary.suffixes_by_depth
    == .tables.shuffled_boundary.suffixes_by_depth
  and .tables.full_boundary.terminal_occurrences
    == .tables.shuffled_boundary.terminal_occurrences
  and .tables.full_boundary.nonterminal_occurrences
    == .tables.shuffled_boundary.nonterminal_occurrences
  and .tables.full_boundary.terminal_by_depth
    == .tables.shuffled_boundary.terminal_by_depth
  and .tables.full_boundary.nonterminal_by_depth
    == .tables.shuffled_boundary.nonterminal_by_depth
  and .tables.changed_boundary_terminals > 0
  and .changes.stop_vs_shuffled > 0
  and .arms.content_full_boundary.reading.full_boundary_stops > 0
' "$OUT/reports/train-preflight.json" >/dev/null

run_limited /usr/bin/time -v -o "$OUT/panel.time.txt" \
  "$GEN" out="$PANEL" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2775 protocol=2 \
  >"$OUT/panel.stdout.json"
check_rss "$OUT/panel.time.txt" 2097152
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2775 and .dialogue_protocol_version == 2
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
    --cap 64 --seed 2775 >"$OUT/reports/$name.json"
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
    and .tables.content.bos_transitions == 0
    and .tables.full_boundary.ambiguous_suffixes > 0
    and .tables.full_boundary.suffixes_by_depth
      == .tables.shuffled_boundary.suffixes_by_depth
    and .tables.full_boundary.terminal_occurrences
      == .tables.shuffled_boundary.terminal_occurrences
    and .tables.full_boundary.nonterminal_occurrences
      == .tables.shuffled_boundary.nonterminal_occurrences
    and .tables.full_boundary.terminal_by_depth
      == .tables.shuffled_boundary.terminal_by_depth
    and .tables.full_boundary.nonterminal_by_depth
      == .tables.shuffled_boundary.nonterminal_by_depth
    and .tables.changed_boundary_terminals > 0
    and .changes.stop_vs_reply > 0
    and .changes.stop_vs_shuffled > 0
    and .arms.content_full_boundary.reading.full_boundary_stops > 0
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
cmp <(normalize "$OUT/reports/seed1.json") \
  <(normalize "$OUT/reports/seed1-repeat.json")
cmp <(normalize "$OUT/reports/seed2.json") \
  <(normalize "$OUT/reports/seed2-repeat.json")
find "$OUT/reports" -type f -name '*.json' -print0 |
  xargs -0 -r gzip -9
test "$(du -sb "$OUT" | cut -f1)" -le 33554432
```

## Stop rule

Stop on source, artifact, panel, table, boundary, parity, determinism, seed or
output-root drift; fewer than 200 reachable replies; a vacuous boundary or
shuffle; fewer than ten oracle-exact bodies for exact comparison; wall time
above 30 minutes; peak RSS above 2 GiB; retained evidence above 32 MiB; or any
need to edit serving, CUDA, float-training, exporter or Claude-owned source.
Once seed 2775 is read, do not change boundary depth, confidence, shuffle,
selectors or bars.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 full-sequence reply boundary attempt 2

reviewer: `full_boundary_attempt2_shell_fit`
date: 2026-11-05
head: `71d9d2d9d7e4973b376cc5ae59760171da1a4f26`
note: Executable timeout/time wrapper, all-exit report compression, unchanged
science and source, preserved invocation fault, and absent seed were verified.

## D67 pre-run review

D67 PRE-RUN: GO - registration head
`945f677f554f87865dcb0aa37d448f9b91a5368d`.

reviewer: `full_boundary_attempt2_shell_go`
date: 2026-11-05
note: FIT at `71d9d2d9d`, executable measured-stage timeout wrapper, all-exit
compression, source/input pins, compact fault evidence and absent seed were
verified.

## Attempt 2 result

- fresh panel seed 2775: 128 `dev,dev` protocol-2 conversations;
- 461 reference-reachable replies on each artifact;
- tokens SHA-256:
  `63b6163e84d5f10a1d87e2c2c38c9038c42548b61e5a55ea36d687c583272501`;
- mask SHA-256:
  `bdd2ae86fc1e7279c29145e4d5d8b83925f83265cd55308dc8b3ae1993082f08`;
- manifest SHA-256:
  `39762c8fe2fa09581a239a0e0d497c02674e3b168107db281ea7f7d361ba0d9c`;
- seed 1 treatment/reply: 15/15 exact, 66/66 premature, 662/662 body;
- seed 2 treatment/reply: 15/15 exact, 94/94 premature, 656/656 body;
- observable stop changes versus reply-only: **0** and **0**;
- observable stop changes versus shuffled-full: **179** and **225**;
- shuffled-full exact: **0** and **0**;
- content no-boundary exact: **0** and **0**;
- content oracle exact: **25** and **25**;
- the control gate stopped before repeats;
- retained attempt-2 evidence is **5,268,949** bytes and sealed by
  `SHA256SUMS` SHA-256
  `310dca229a6889c4a4e228a7fd0dad60ce919589a25c89ff9b995685ba6fafe5`.
