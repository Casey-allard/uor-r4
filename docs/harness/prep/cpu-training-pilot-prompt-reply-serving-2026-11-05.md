# Pre-registration: prompt-reply serving sidecar

Run ID: `cpu-training-pilot-prompt-reply-serving-2026-11-05`

## Status

COMPLETE - **KEEP**. Attempt 4 reproduced all 471 CPU selections on both
artifacts with 39 exact, 131 openings, 948 body positions and zero premature
BOS. Record:
[deepseek-207](../../loop/tasks/cpu-training-pilot/deepseek-207/REPORT.md).

This is D84 CPU development and serving-contract evidence. It does not claim
general response generation, fresh language understanding, GPU qualification
or useful chat.

Attempt 1 completed at implementation `6a48ad613` and is preserved at
`local/deepseek/prompt-reply-serving/2026-11-05`. Exact-head review found that
the parser did not bind the sidecar's ignored protocol ids to the loaded
dialogue protocol. Attempt 2 adds that validation without changing selection,
data or bar and is preserved at `2026-11-05-attempt-2`. The exact API build
then found a CLI-only protocol scope fault. Attempt 3 fixes that binding and
is preserved at `2026-11-05-attempt-3`. Final review then required the public
serving attachment to bind the loaded tokenizer bytes and protocol directly.
Attempt 4 adds that validation and decides at a fresh output root.

## Question

Can the positive deepseek-204 exact prompt-set retrieval result be persisted as
an artifact-bound integer sidecar and served through the existing loopback
chat boundary without changing the model artifact or losing exact selection?

## Fixed construction

The sidecar schema is `uor-r4.stack-prompt-reply-readout/1`. It records:

- complete artifact SHA-256;
- complete tokenizer SHA-256;
- dialogue protocol version;
- ignored BOS/EOS/UNK ids;
- TRAIN prompt/reply token pairs.

Selection uses exact prompt-token-set Jaccard, ordered by integer
cross-multiplication and then the lower TRAIN ordinal. The selected complete
reply is emitted with EOS when it fits the request cap and model context.
Otherwise the normal greedy model path runs. The sidecar is capped at 64 MiB.

`uor-chat --prompt-reply` is accepted only with loopback serving. The sidecar
never changes `UORLUT01`, raw `step`, `step_native` or the existing
content-readout selector.

## Arms

1. persisted sidecar selection;
2. CPU in-memory `unigram_jaccard_ordinal_tie`;
3. cap-forced model fallback;
4. malformed/foreign sidecar refusals.

## Data split

- TRAIN: `tt-2711`;
- evaluation: sealed seed-2777 panel `dd-2777`, 471 reachable replies;
- both public dual-c131 raw artifacts;
- persisted sidecar built separately for each artifact;
- fresh-process repeats for both artifacts;
- no evaluation labels enter sidecar construction.

This is a serving-contract reuse of the already closed seed-2777 capability
panel, not a new fresh-capability claim.

## Instrument gate

- source, tokenizer, artifact, TRAIN and panel hashes match;
- sidecar and output root are absent before release;
- sidecar binds artifact, tokenizer and protocol;
- persisted sidecar reloads to 453 entries within 64 MiB;
- every panel decision matches the CPU selected ordinal and reply SHA-256;
- OpenAI tests exercise complete emission, cap fallback and identity metadata;
- no prohibited source path changes.

## Bar

For both artifacts:

- all **471** serving selections match the CPU arm by ordinal and reply SHA;
- exact **39**, opening **131**, body **948**, premature BOS **0**;
- all **471** reachable prompts emit;
- sidecar reload and fresh-process repeat are exact;
- selection throughput is at least **500 prompts/s**;
- API complete-hit, cap-fallback and metadata tests pass.

## Stop rule

Stop on source, artifact, tokenizer, TRAIN, panel, sidecar, selection, API,
parity, determinism or output-root drift; fewer than 200 reachable rows; a
vacuous or oversized sidecar; throughput below 500 prompts/s; or any need to
change `UORLUT01`, serving kernels, CUDA, float training or Claude-owned source.

## Cost

Local CPU only on the Ryzen 9 9950X. The whole run is capped at 30 minutes,
2 GiB peak RSS and 32 MiB retained evidence. Builds and the two persisted
sidecar checks have a 30-minute bound.

## Base, source and inputs

- base: `c0e6b6757`;
- implementation: `ab6afb920`;
- prompt-reply module:
  `344b599f2ae521656741666d907c63eb432331e11837f6d591a6341de3aedf22`;
- OpenAI integration:
  `93e5a80e058299f3d1ccc65e687320e6dc82c11281d2e6991ce6aeb2b901cde0`;
- CLI:
  `fb60135b74a27181404983b726d7d3ab69d696fbbbbea1fa0d91aeaaeff328ae`;
- builder:
  `dc061cea85383c7acd067893356acd345743e399d3c04bdf8ef804bcf2bd0080`;
- serving check:
  `97627cc56cddc4c770a89e7ddcf2f98b1a38d9cc15e5c4e846869013ac243b6f`;
- tokenizer:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- artifact seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- artifact seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- TRAIN tokens/mask/manifest:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`,
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`,
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`;
- panel tokens/mask/manifest:
  `13146ad6c1869524d4121b0c016f9548e8c28289390b07e106ec8563a7b1b2b6`,
  `ba97d44e56d690b4d0f39ded6469afc5b6a1ee5f25cc1f0822e9762812995bec`,
  `e19cae91843c8d6db15a5862aac911bcd7bf538b2fa62b927d6b9be1ead147e0`.

## Output

`local/deepseek/prompt-reply-serving/2026-11-05-attempt-4`

## Steps

Run only after FIT and GO. Use a fresh output root and fail closed.

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-prompt-retrieval-serving
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/prompt-reply-serving/2026-11-05-attempt-4
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
PANEL=$SHARED/local/deepseek/prompt-retrieval/2026-11-05/panels/dd-2777
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/prompt-reply-serving
IMPL=ab6afb920

test ! -e "$OUT"
git -C "$SOURCE" diff --quiet "$IMPL" -- crates/uor-r4-integer crates/uor-r4-cpu-train
mkdir -p "$OUT/reports"
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
  cargo build --locked --release -p uor-r4-cpu-train \
  --bin cpu-prompt-reply-sidecar --bin cpu-prompt-reply-serving-check

build_sidecar() {
  artifact=$1 expected=$2 name=$3
  "$BUILD/release/cpu-prompt-reply-sidecar" "$TRAIN" "$artifact" "$TOKENIZER" \
    --out "$OUT/reports/$name.sidecar.json" --protocol 2 \
    --expected-artifact "$expected" \
    --expected-tokenizer a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 \
    >"$OUT/reports/$name.sidecar-build.json"
}
check() {
  name=$1 artifact=$2 expected=$3
  /usr/bin/time -v -o "$OUT/reports/$name.time.txt" \
    timeout 1800s "$BUILD/release/cpu-prompt-reply-serving-check" \
    "$OUT/reports/$name.sidecar.json" "$TRAIN" "$PANEL" "$artifact" "$TOKENIZER" \
    --protocol 2 --expected-artifact "$expected" \
    --expected-tokenizer a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 \
    >"$OUT/reports/$name.json"
}

build_sidecar "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd seed1
build_sidecar "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 seed2
build_sidecar "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd seed1-repeat
build_sidecar "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 seed2-repeat
check seed1 "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
check seed2 "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
check seed1-repeat "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
check seed2-repeat "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
cmp "$OUT/reports/seed1.sidecar.json" "$OUT/reports/seed1-repeat.sidecar.json"
cmp "$OUT/reports/seed2.sidecar.json" "$OUT/reports/seed2-repeat.sidecar.json"
normalize() {
  jq -S 'walk(if type == "object"
    then del(.selection_elapsed_us, .prompts_per_second)
    else . end)' "$1"
}
cmp <(normalize "$OUT/reports/seed1.json") <(normalize "$OUT/reports/seed1-repeat.json")
cmp <(normalize "$OUT/reports/seed2.json") <(normalize "$OUT/reports/seed2-repeat.json")
jq -e '
  .reading.exact_reference == 39 and .reading.opening_correct == 131
  and .reading.correct_body_positions == 948 and .reading.premature_bos == 0
  and .reading.emitted == 471 and .reading.reference_reachable == 471
  and .cpu_arm.exact_reference == 39
  and .repeat_exact == true and .prompts_per_second >= 500
' "$OUT/reports/seed1.json" "$OUT/reports/seed2.json"
printf 'KEEP\n' >"$OUT/outcome.txt"
test "$(du -sb "$OUT" | cut -f1)" -le 33554432
```

## Minimum test power

- two artifact-bound persisted sidecars;
- 471 reachable panel prompts per artifact;
- per-row CPU/serving ordinal and reply identity;
- API complete-response, cap-fallback and identity tests;
- 500 prompts/s selection floor;
- fresh-process repeats.

## Outcomes

- **KEEP** if every exact, row-parity, emission, API and throughput condition
  passes on both artifacts.
- **REJECT** if any serving path differs or falls below the registered bar.
- **CONTROL_FAULT** if the persisted sidecar or output identity drifts.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 prompt-reply serving sidecar
sheet: docs/harness/prep/cpu-training-pilot-prompt-reply-serving-2026-11-05.md
reviewer: deepseek exact-head self-review (independent delegation unavailable in this run)
note: Implementation ab6afb920 binds ignored control ids, the CLI protocol and
  the loaded tokenizer bytes; 473 serving library tests and 19 OpenAI loopback
  tests pass; strict CPU Clippy and targeted formatting pass; earlier output
  roots are preserved and the attempt-4 root is absent.
date: 2026-11-05
A FIT verdict clears compute for the configuration in this sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone
before compute.

Earlier attempts' FIT records remain in the harness ledger. Attempt 4's
reviewer identifies the same limitation: this was a local self-review.

## Pre-run GO

GO - exact-head self-review under the current disabled-delegation policy,
2026-11-05. Implementation `ab6afb920` is clean; source, tokenizer, artifacts,
TRAIN panel, evaluation panel and generator inputs match. Earlier outputs are
preserved, and the attempt-4 root was absent at GO.
