# Pre-registration: compressed prompt VSA

Run ID: `cpu-training-pilot-prompt-vsa-compression-2026-11-06`

## Status

COMPLETE - **REJECT** for the selected 1024-bit modulo signature. It was
**3.67x** faster inside the selector on both artifacts, but lost one exact
reply, five openings and 50 body positions on fresh seed 2778. Exact 4,096-bit
serving remains the default. Record:
[deepseek-211](../../loop/tasks/cpu-training-pilot/deepseek-211/REPORT.md).

This is D84 CPU serving research. It tests a deterministic lossy compressed
representation of the exact deepseek-209 prompt selector. It is not a new
training-capability experiment or a serving-default change.

## Question

Can a compressed binary prompt signature selected on TRAIN alone preserve or
improve exact-reply, opening and body quality while reducing selection work?

## Fixed construction

The persisted `uor-r4.stack-prompt-reply-readout/1` schema remains unchanged.
The default selector remains the exact 4,096-bit vocabulary vector. Research
construction accepts a width divisible by 64 and one deterministic mapping:

```text
modulo(t, width)    = t % width
splitmix64(t, width) = splitmix64(t) % width
```

The SplitMix64 seed is the token id itself and the standard four mixing
constants are applied. Query and entry signatures use the same mapping. The
rational Jaccard ordering and lower-ordinal tie remain unchanged. Signature
bits are packed little-endian within each `u64` word.

Compression is lossy because distinct token ids can map to the same bit and
therefore change intersection and union cardinalities. No claim of exact
parity is made for the compressed arm.

## Calibration

Calibration uses only normalized TRAIN `tt-2711` prompt/reply entries. For each
prompt, compute:

```text
BLAKE3("deepseek-211-inner-v1" || each prompt token as u32 little-endian)
```

A prompt is in the inner held-out set when the digest's first byte is less than
51; all other prompts form the inner library. This split is fixed before
execution and does not read the evaluation panel.

Compare each candidate with the exact 4,096-bit selector over the same inner
library and held-out prompts. Rank candidates by:

1. selected-ordinal agreement;
2. selected-reply agreement;
3. smaller signature width;
4. fixed order: 256-modulo, 512-modulo, 1024-modulo, 1024-SplitMix64.

The winner is frozen before the fresh panel is opened.

## Arms

1. `compressed`: selected candidate, loaded from the unchanged sidecar;
2. `exact`: exact 4,096-bit vocabulary signature;
3. `cpu_oracle`: in-memory CPU `unigram_jaccard_ordinal_tie`.

## Data split

- TRAIN `tt-2711`;
- fresh `cell=dev,dev`, seed 2778, protocol 2, 128 conversations, generated
  only after FIT and GO;
- both public dual-c131 raw artifacts;
- 64-token reply cap;
- at least 200 reference-reachable panel rows per artifact;
- three alternating fresh-process runs per artifact and arm;
- unchanged deepseek-207 attempt-4 sidecars.

Fresh and repeat processes remove timing fields before equality checks. The
exact arm is the quality control; the CPU arm is the decision oracle only for
the exact representation.

## Instrument gate

- source, binary, generator, tokenizer, artifacts, TRAIN and sidecars match
  their pins;
- the seed-2778 panel and output root are absent before FIT and GO;
- the output signature width and mapping equal the recorded calibration
  winner on every run;
- exact emits every reachable row and matches CPU ordinals and reply SHA on
  every row;
- compressed emits every reachable row;
- the sidecar serializes and reloads byte-identically;
- API complete-hit, cap-fallback and identity tests pass.

Any failure is an instrument, power or drift fault, not a mechanism negative.

## Bar

For both artifacts:

- exact has zero premature BOS and 471/471 CPU ordinal/reply parity;
- compressed has zero premature BOS and emits all reachable rows;
- compressed `exact_reference`, `opening_correct` and
  `correct_body_positions` are each no lower than exact;
- compressed median selection throughput is at least 1.25x exact;
- compressed median selection throughput is at least 25,000 prompts/s;
- three compressed runs are identical after timing removal;
- sidecars remain byte-identical.

## Outcomes

- **KEEP** if every quality, throughput, identity and determinism condition
  passes on both artifacts.
- **REJECT** if the selected compressed representation loses any quality
  aggregate or misses either throughput floor. This closes only the selected
  lossy signature configuration.
- **CONTROL_FAULT** if the exact arm, inputs, sidecars or deterministic mapping
  drift.

## Stop rule

Stop before panel generation or publication on source, input, generator,
artifact, sidecar or output-root drift; a panel below 200 reachable rows; an
exact-arm CPU mismatch; compressed premature BOS; any compressed quality
regression; throughput below either floor; or nondeterministic calibration,
decisions or repeats.

## Cost

Local CPU only on the Ryzen 9 9950X. The fresh-panel run is capped at 30
minutes, 2 GiB peak RSS and 32 MiB retained evidence. Calibration and focused
tests are capped at five minutes.

## Base and inputs

- base: `4d1e705aa`;
- implementation: `16f35277a`;
- prompt-reply source:
  `bb472b9bc27ce22b607be7073e5ba31c33d4dab489a2a44eeb3374e9b302b7a3`;
- stack export source:
  `d40d3683ce4ab681aaefe3ee990203071bb541da33c37faf6eb7ba3bdb59c28c`;
- serving-check source:
  `b454260ad0789902d9c76698c30a3b92be20cc4b2bd7d7126a8ba450353be6c2`;
- serving-check binary:
  `95b75f18a4d13201b8d8db9df6708b40400e2de8d49f12892850c2d18579ab1d`;
- TRAIN tokens:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- TRAIN mask:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- TRAIN manifest:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`;
- generator tree:
  `869e65db6d0bb997a5f1094ab0de4efa1ac10c25`;
- generator source hash:
  `df24dd629b3606ef43f452173622cac634404ad1acaa33137ef768812be296c3`;
- tokenizer:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- artifact seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- artifact seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- sidecar seed 1:
  `2a298b44ea34d1a8e2d40191951d3da9b3300e2ac7de65e6c2ec13950a0b6424`;
- sidecar seed 2:
  `6504dcb047ce7157dd5e968c29441505cf48d993f01ec63222125634c61963b7`.

## Output

Attempt 1: `local/deepseek/prompt-vsa-compression/2026-11-06` (sealed
instrument fault; no panel created).

Attempt 2: `local/deepseek/prompt-vsa-compression/2026-11-06-attempt-2`

Fresh panel: `.../panels/dd-2778`

The attempt-2 root and panel were absent when the repair began.

## Attempt log

- Attempt 1 verified all input pins and the absent output roots, then stopped
  before panel generation because the standalone generator `Cargo.lock` was
  stale under `--locked`. The trap sealed the output root containing only
  `generator-build.time.txt` and `SHA256SUMS`. No panel, model or evaluation
  input was read by the generator.
- Attempt 2 uses a byte copy of the generator under an ignored local path. Its
  five path dependencies are rewritten to the reviewed worktree, and Cargo
  resolves offline into that copy. The reviewed repository lockfile is not
  modified. Generator manifest SHA-256:
  `eabe169cfca1bf1a4562e14d6986ec2aeb06a933d4b78d52486e3a0b9e49c365`;
  isolated lock SHA-256:
  `a0d705600982aa1b60c9e5cc6d3b9503740376bffaffd4129a123cec272ee408`.
  The generator source, seed, cell, tokenizer, protocol and requested panel
  shape are unchanged.

## Steps

Run only after FIT and GO. Fill the implementation pin from the reviewed source
head before starting.

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-prompt-vsa-compression
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/prompt-vsa-compression/2026-11-06-attempt-2
PANEL=$OUT/panels/dd-2778
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
SIDECAR1=$SHARED/local/deepseek/prompt-reply-serving/2026-11-05-attempt-4/reports/seed1.sidecar.json
SIDECAR2=$SHARED/local/deepseek/prompt-reply-serving/2026-11-05-attempt-4/reports/seed2.sidecar.json
BUILD=$SOURCE/target/deepseek/prompt-vsa-compression
GEN_TARGET=$SOURCE/target/deepseek/prompt-vsa-compression-generator
GEN_SOURCE=$SOURCE/local/deepseek/prompt-vsa-compression-generator-attempt-2
IMPL=16f35277a

test ! -e "$OUT"
test ! -e "$PANEL"
git -C "$SOURCE" diff --quiet "$IMPL" -- \
  crates/uor-r4-integer/src/stack/prompt_reply.rs \
  crates/uor-r4-integer/src/stack/mod.rs \
  crates/uor-r4-cpu-train/src/bin/cpu-prompt-reply-serving-check.rs
sha256sum -c <<EOF
a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377  $TOKENIZER
a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd  $SEED1
da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1  $SEED2
140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91  $TRAIN/tokens.u16
d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad  $TRAIN/response_mask.u8
d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5  $TRAIN/manifest.json
2a298b44ea34d1a8e2d40191951d3da9b3300e2ac7de65e6c2ec13950a0b6424  $SIDECAR1
6504dcb047ce7157dd5e968c29441505cf48d993f01ec63222125634c61963b7  $SIDECAR2
EOF

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
/usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GEN_TARGET" \
  cargo build --offline \
  --manifest-path "$GEN_SOURCE/Cargo.toml" \
  --features panel-writer --bin heldout-panel
GEN=$GEN_TARGET/debug/heldout-panel

/usr/bin/time -v -o "$OUT/panel.time.txt" \
  timeout 900s "$GEN" out="$PANEL" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2778 protocol=2 \
  >"$OUT/panel.stdout.json"
jq -e --arg tokenizer_sha a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2778 and .dialogue_protocol_version == 2
  and .cell.value == "dev" and .cell.phrasing == "dev"
  and .conversations == 128 and .world == "m-world-v2"
  and .tokenizer.sha256 == $tokenizer_sha
' "$PANEL/manifest.json" >/dev/null

/usr/bin/time -v -o "$OUT/build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" \
  cargo build --locked --release -p uor-r4-cpu-train \
  --bin cpu-prompt-reply-serving-check
BIN=$BUILD/release/cpu-prompt-reply-serving-check

run() {
  name=$1 artifact=$2 expected=$3 sidecar=$4
  shift 4
  /usr/bin/time -v -o "$OUT/reports/$name.time.txt" \
    timeout 1800s "$BIN" "$sidecar" "$TRAIN" "$PANEL" "$artifact" "$TOKENIZER" \
    --protocol 2 --expected-artifact "$expected" \
    --expected-tokenizer a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 \
    "$@" >"$OUT/reports/$name.json"
}

for repeat in 1 2 3; do
  run "seed1-exact-$repeat" "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$SIDECAR1"
  run "seed1-compressed-$repeat" "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$SIDECAR1" --choose-compressed
  run "seed2-exact-$repeat" "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$SIDECAR2"
  run "seed2-compressed-$repeat" "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$SIDECAR2" --choose-compressed
done

for seed in seed1 seed2; do
  for arm in exact compressed; do
    jq -s -e '
      map(del(.prompts_per_second,.selection_elapsed_us)) | unique | length == 1
    ' "$OUT"/reports/$seed-$arm-*.json >/dev/null
  done
  jq -e '
    .signature_bits == 4096 and .signature_mapping == "vocabulary"
    and .reading.premature_bos == 0
  ' "$OUT/reports/$seed-exact-1.json" >/dev/null
  jq -e '
    .reading.premature_bos == 0
    and .reading.emitted == .reading.reference_reachable
    and .signature_bits < 4096
    and .calibration.selected_bits == .signature_bits
    and .calibration.selected_mapping == .signature_mapping
  ' "$OUT/reports/$seed-compressed-1.json" >/dev/null
  exact_median=$(jq -s 'map(.prompts_per_second)|sort|.[length/2]' "$OUT"/reports/$seed-exact-*.json)
  compressed_median=$(jq -s 'map(.prompts_per_second)|sort|.[length/2]' "$OUT"/reports/$seed-compressed-*.json)
  jq -n -e \
    --argjson exact "$(jq -c .reading "$OUT/reports/$seed-exact-1.json")" \
    --argjson compressed "$(jq -c .reading "$OUT/reports/$seed-compressed-1.json")" \
    --argjson exact_rate "$exact_median" \
    --argjson compressed_rate "$compressed_median" '
      $compressed.exact_reference >= $exact.exact_reference
      and $compressed.opening_correct >= $exact.opening_correct
      and $compressed.correct_body_positions >= $exact.correct_body_positions
      and $compressed_rate >= 1.25 * $exact_rate
      and $compressed_rate >= 25000
    ' >/dev/null
done

jq -s -e '
  (map(.calibration.selected_bits) | unique | length == 1)
  and (map(.calibration.selected_mapping) | unique | length == 1)
' "$OUT"/reports/seed*-compressed-*.json >/dev/null

env CARGO_TARGET_DIR="$BUILD" cargo test --locked \
  -p uor-r4-integer --features serving-wasm --lib \
  >"$OUT/reports/integer-lib-tests.log" 2>&1
env CARGO_TARGET_DIR="$BUILD" cargo test --locked \
  -p uor-r4-integer --features serving-wasm --test openai_loopback \
  >"$OUT/reports/openai-loopback-tests.log" 2>&1
env CARGO_TARGET_DIR="$BUILD" cargo test --locked -p uor-r4-cpu-train --lib \
  >"$OUT/reports/cpu-tests.log" 2>&1

printf 'KEEP\n' >"$OUT/outcome.txt"
test "$(du -sb "$OUT" | cut -f1)" -le 33554432
```

## Minimum test power

- both public artifacts;
- at least 200 reference-reachable seed-2778 rows per artifact;
- exact 471/471 row-level CPU ordinal/reply identity;
- three alternating fresh-process runs per artifact and arm;
- candidate selection on at least 40 held-out TRAIN prompts;
- equal-or-better compressed exact, opening and body aggregates;
- at least 1.25x exact and 25,000 prompts/s selection throughput;
- byte-identical sidecars, deterministic calibration and zero premature BOS.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 compressed prompt VSA
sheet: docs/harness/prep/cpu-training-pilot-prompt-vsa-compression-2026-11-06.md
reviewer: deepseek exact-head self-review (independent delegation unavailable in this run)
note: Implementation 16f35277a is clean. The opt-in compressed constructor leaves
  the exact 4,096-bit defaults and sidecar schema unchanged. Eight prompt-reply
  tests, two calibration tests, 477 serving tests (one expected ignore), 17
  OpenAI loopback tests and 394 CPU tests (four expected ignores) pass. Inputs
  and source pins match; the fresh panel and output roots remain absent.
date: 2026-11-06

## Pre-run GO

GO - exact-head self-review under the current disabled-delegation policy.
Implementation `16f35277a`, source pins and release binary
`95b75f18a4d13201b8d8db9df6708b40400e2de8d49f12892850c2d18579ab1d`
match. This was not an independent review. Seed 2778 may now be generated under
the fixed steps and stop rule above.
