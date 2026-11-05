# Hierarchical copy-key depth screen

**Status:** `TEST FITNESS: FIT`

## Question

Does exact copy retrieval improve when the read backs off from the longest
available suffix key, depth four to one, instead of fixing one key for every
position?

Claude-267 measured latest exact matches at copy-key depths one and two on
public DEV. Its `k2>k1` backoff was right most often in that screen. This unit
tests whether extending the same backoff to depths three and four adds a
material exact-match signal. It is a data-only read screen, not a trained-model
or language result.

This is D84 CPU development research on the Ryzen 9 9950X. It changes no float
training, integer serving, CUDA or Claude-owned source.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `70b6445d4` |
| panel-generator source | `4e42ffca5` (`docs/labs/one-model-end-to-end/instrument`) |
| tokenizer | `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377` |
| public M1 DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| public M1 DEV mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| public M1 DEV manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

The public DEV parity inputs and the Claude-267 reference counts were already
opened during implementation debugging. They are instrument controls, not
decision panels for this run.

## Fixed construction

Use 192-token BOS-delimited scoring windows, the same boundaries as
`build_document_windows`. Copy the window into `x` from `win_start` through
the last eligible target. For eligible target position `p`, set
`t = p - 1 - win_start`:

```text
query_d(t) = x[t], x[t-1], ..., x[t-d+1]
key_d(s)   = x[s-1], x[s-2], ..., x[s-d]
```

A candidate `1 <= s < t` predicts `x[s]`; the latest exact slice match wins.
The current query position is not a candidate. Missing key history at the start
of `x` repeats `x[0]`, matching Claude-267's depth-two reference.

The candidate is `longest_4_to_1`: use depth four if it has an exact match,
then three, two, one. Controls are fixed depths one through four,
`k2>k1`, and a deterministic random choice over available depths. The seed is
149. `fixed_depth_1` is the `shortest` control.

## Data split

- Public M1 DEV: 400 conversations, 14,287 eligible targets, instrument only.
- `copy-key-depth-tt-2501`: a new 64-conversation `train,train` panel.
- `copy-key-depth-dd-2502`: a new 64-conversation `dev,dev` panel.

The two decision panels are generated independently at fixed seeds. Neither is
used to fit a parameter. Each must contain at least 200 eligible target
positions.

## Instrument gate

Public DEV must reproduce Claude-267 exactly before the bar is read:

- 8,243 copyable positions;
- depth 1: 6,306 exact matches and 3,859 right;
- depth 2: 4,383 exact matches and 3,506 right;
- `k2>k1`: 4,388 right.

Any mismatch stops the run as an instrument fault. Public DEV is not a decision
panel.

## Decision panels

Draw two new m-world-v2 panels after FIT and GO, with the pinned tokenizer and
generator:

- `copy-key-depth-tt-2501`: `cell=train,train`, 64 conversations, seed `2501`;
- `copy-key-depth-dd-2502`: `cell=dev,dev`, 64 conversations, seed `2502`.

Both are new to this decision. The earlier exploratory readings on
`train-control-v3`, `dev-panel-v3` and `panel-dev-dev` are disclosed in the
REPORT but do not enter the bar.

### Exploratory readings, not decision results

Implementation debugging opened three existing panels before FIT:

- `train-control-v3`: longest 754/1,103 copyable decisions against `k2>k1`
  744/1,103;
- `dev-panel-v3`: longest 167/258 against `k2>k1` 167/258;
- `panel-dev-dev`: longest 691/1,037 against `k2>k1` 682/1,037.

These readings are **NOT A DECISION RESULT**. The pre-registered bar written
before them still required both panels, and `dev-panel-v3` showed no gain. The
small improvements on the other two motivated a properly powered new-panel
screen, not a retrospective pass.

## Bar

The candidate passes only if it beats `k2>k1` by the paired two-sigma rule on
both new decision panels, and beats the random-depth control by the same rule
on both. Random-control novelty is counted only on copyable positions. The paired threshold is
`max(2, ceil(2*sqrt(control-only)))`.

A pass makes depths three and four a supported CPU read candidate for the next
Claude copy-read build. A fail closes the fixed longest-first 1–4 exact suffix
router at this context, not deeper keys generally.

## Minimum test power

Each decision panel must contribute at least 200 eligible positions, with the
paired rule computed per target position. Two independently generated
64-conversation panels guard against a seed-specific ordering. The seeded
random control must change at least one copyable `longest` decision on each
panel, and a fresh process must reproduce the dev-panel report byte for byte.
The result is a read-screen decision, not a model or language claim.

## Stop rules

Stop on a public-DEV parity mismatch, input/hash drift, nondeterminism,
cross-document or cross-window candidate use, a vacuous random control,
resource breach, or any need to change serving or Claude-owned source.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT, GO and review | 600 s |
| release build and controls | 300 s |
| three corpus screens and one repeat | 60 s |
| delivery and evidence | 240 s |
| total increment | **1,200 s** |

- Build jobs: 16; build RSS cap: 4 GiB.
- Screen-run cap: 600 s and 1 GiB RSS per command.
- Evidence cap: 33,554,432 bytes.
- Previous measured cumulative: `36,068.788816343 s`; previous admitted
  cumulative: `69,727.677487699 s`.
- New admitted cumulative: **70,927.677487699 s**.

The named model-time ledger at
`.uor-models/native-joint-learning-2026-09-04/model-time.json` is `ABSENT`.

## Steps

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-key-depth
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-key-depth/2026-11-05-attempt-3
PANELS=$OUT/panels
BUILD=$SOURCE/target/deepseek/copy-key-depth-run
GENERATOR_TARGET=$SOURCE/target/deepseek/copy-key-panel-generator
DEV=$SHARED/local/deepseek/public-m1-parity
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
IMPL_PIN=70b6445d4

test ! -e "$OUT"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- \
  .cargo Cargo.toml Cargo.lock rust-toolchain.toml crates third_party
git -C "$SOURCE" diff --quiet 4e42ffca5 -- \
  docs/labs/one-model-end-to-end/instrument
sha256sum -c <(printf '%s\n' \
  '0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b  '"$DEV"'/dev-tokens.u16' \
  'a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078  '"$DEV"'/dev-response-mask.u8' \
  'ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab  '"$DEV"'/dev-manifest.json' \
  'a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377  '"$TOKENIZER") --quiet

mkdir -p "$OUT"
seal() {
  (cd "$OUT"; find . -type f ! -name SHA256SUMS ! -name SHA256SUMS.tmp -print0 |
    sort -z | xargs -0 -r sha256sum >SHA256SUMS.tmp
    mv SHA256SUMS.tmp SHA256SUMS
    sha256sum -c SHA256SUMS --quiet)
}
trap 'status=$?; trap - EXIT; seal || status=$?; exit "$status"' EXIT

/usr/bin/time -v -o "$OUT/build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" cargo build --locked --release \
  -p uor-r4-cpu-train --bin cpu-copy-key-depth

/usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GENERATOR_TARGET" cargo build --locked \
  --manifest-path "$SOURCE/docs/labs/one-model-end-to-end/instrument/Cargo.toml" \
  --features panel-writer --bin heldout-panel
GEN=$GENERATOR_TARGET/debug/heldout-panel
/usr/bin/time -v -o "$OUT/train-panel-gen.time.txt" \
  timeout 600s "$GEN" out="$PANELS/train-train-2501" tokenizer="$TOKENIZER" \
  cell=train,train conversations=64 seed=2501 protocol=2 \
  >"$OUT/train-panel-gen.stdout.json"
/usr/bin/time -v -o "$OUT/dev-panel-gen.time.txt" \
  timeout 600s "$GEN" out="$PANELS/dev-dev-2502" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=64 seed=2502 protocol=2 \
  >"$OUT/dev-panel-gen.stdout.json"

BIN=$BUILD/release/cpu-copy-key-depth
run() {
  name=$1
  dir=$2
  token_file=$3
  mask_file=$4
  manifest_file=$5
  /usr/bin/time -v -o "$OUT/$name.time.txt" \
    timeout 600s "$BIN" "$dir/$token_file" "$dir/$mask_file" "$dir/$manifest_file" \
      --context 192 --seed 149 >"$OUT/$name.json"
}
run dev "$DEV" dev-tokens.u16 dev-response-mask.u8 dev-manifest.json
run train-train "$PANELS/train-train-2501" tokens.u16 response_mask.u8 manifest.json
run dev-dev "$PANELS/dev-dev-2502" tokens.u16 response_mask.u8 manifest.json
run dev-dev-repeat "$PANELS/dev-dev-2502" tokens.u16 response_mask.u8 manifest.json

test "$(jq -r '.reading.copyable_positions' "$OUT/dev.json")" -eq 8243
test "$(jq -r '.reading.fixed[0].positions_with_match' "$OUT/dev.json")" -eq 6306
test "$(jq -r '.reading.fixed[0].right' "$OUT/dev.json")" -eq 3859
test "$(jq -r '.reading.fixed[1].positions_with_match' "$OUT/dev.json")" -eq 4383
test "$(jq -r '.reading.fixed[1].right' "$OUT/dev.json")" -eq 3506
test "$(jq -r '.reading.k2_backoff_k1.right' "$OUT/dev.json")" -eq 4388
test "$(jq -r '.reading.eligible_positions' "$OUT/train-train.json")" -ge 200
test "$(jq -r '.reading.eligible_positions' "$OUT/dev-dev.json")" -ge 200
test "$(jq -r '.reading.random_changed_from_longest' "$OUT/train-train.json")" -gt 0
test "$(jq -r '.reading.random_changed_from_longest' "$OUT/dev-dev.json")" -gt 0
test "$(awk -F': ' '/Maximum resident set size/ {print $2}' "$OUT/build.time.txt")" -le 4194304
test "$(awk -F': ' '/Maximum resident set size/ {print $2}' "$OUT/generator-build.time.txt")" -le 4194304
for t in "$OUT"/*.time.txt; do
  [ "$t" = "$OUT/build.time.txt" ] || [ "$t" = "$OUT/generator-build.time.txt" ] ||
    test "$(awk -F': ' '/Maximum resident set size/ {print $2}' "$t")" -le 1048576
done
cmp "$OUT/dev-dev.json" "$OUT/dev-dev-repeat.json"
test "$(du -sb "$OUT" | cut -f1)" -le 33554432
```

Read `longest_vs_k2_backoff.passes` and `longest_vs_random.passes` in both
`train-train.json` and `dev-dev.json`. Both must be true for a candidate pass.
`dev.json` is the Claude-267 instrument-parity control only.

## Rollback boundary

The unit adds an isolated CPU module and binary. Reverting it removes only the
screen; no model artifact, serving format, CUDA code or Claude-owned source is
changed.

## TEST FITNESS

The prior FIT at `70b6445d4` is superseded by attempt 1's input-path
instrument fault.

The prior corrected FIT is superseded by attempt 2's public-DEV manifest-path
fault.

TEST FITNESS: FIT - M2 #2030 deepseek-149
sheet: docs/harness/prep/cpu-training-pilot-copy-key-depth-2026-11-05.md
reviewer: codex fresh-context FIT round 6
note: Attempts 1 and 2 are sealed and disclosed; attempt 3 passes the exact
  input, generator, panel, power, control, resource and isolation terms.
date: 2026-11-05

A FIT verdict clears compute for this configuration only (D22).

## Pre-run GO

GO - reviewer `codex fresh-context GO`, 2026-11-05. HEAD was clean at
`694b59e0f`; implementation pin `70b6445d4` and generator pin `4e42ffca5`
matched. Attempts 1 and 2, absent attempt 3, all input filenames and hashes,
panel seeds, resources and failure sealing were checked read-only.

## Outcome

Attempt 1 stopped before the oracle: the public-DEV command used
`tokens.u16`/`response_mask.u8` instead of the actual
`dev-tokens.u16`/`dev-response-mask.u8` names. Its build and generated panels
are preserved under `2026-11-05-attempt-1/`; no target reading was produced.
Attempt 2 fixes only those two input filenames.

Attempt 1 `SHA256SUMS`:
`222d12634ac2c8d9f9bcda3edee332e1503f2d477737a03a4c2a234cd9667572`.

Attempt 2 repeated the run with the corrected token and mask filenames but
still used `manifest.json` for public DEV instead of `dev-manifest.json`. No
target reading occurred. Attempt 2 is preserved under
`2026-11-05-attempt-2/`; its `SHA256SUMS` is
`ed23906414c6420e1162d3497ef60819f029d7cb06da07f124b14b9e4acfe833`.

Attempt 3 completed. `longest_4_to_1` beat `k2>k1` by 9:1 discordant decisions
on the train panel and 8:0 on the dev panel, and beat random depth by 32:7 and
33:5. Public DEV reproduced the Claude-267 parity counts. The candidate passes
the registered screen. Evidence `SHA256SUMS` is
`bec19988e1b161973de1037f444ffc1516093f2bd33f54df4a35ee52a09c3a22`.
Generated corpus bytes remain in the sealed local attempt and are omitted from
Git; committed evidence verifies with `PUBLISHED-SHA256SUMS`.
