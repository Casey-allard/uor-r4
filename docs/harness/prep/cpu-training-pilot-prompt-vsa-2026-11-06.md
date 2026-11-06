# Pre-registration: prompt-set binary VSA index

Run ID: `cpu-training-pilot-prompt-vsa-2026-11-06`

## Status

COMPLETE - **KEEP**. Binary VSA preserved all 471 decisions on both artifacts
and raised aggregate median selection throughput from **9,820.55** to
**46,362.99 prompts/s** (**4.72x**). Record:
[deepseek-209](../../loop/tasks/cpu-training-pilot/deepseek-209/REPORT.md).

This is D84 CPU serving research. It tests an exact binary-vector representation
of the existing deepseek-207 selector. It is not a new training-capability
experiment.

## Question

Can a vocabulary-dimensional binary VSA representation preserve every
literal-set prompt retrieval decision while materially increasing serving
selection throughput?

## Fixed construction

The persisted `uor-r4.stack-prompt-reply-readout/1` schema is unchanged. On
load, each prompt set becomes `ceil(vocab / 64)` `u64` words. Token `t` sets bit
`t % 64` in word `t / 64`. Selection computes:

```text
intersection = sum(popcount(query_word & prompt_word))
union        = sum(popcount(query_word | prompt_word))
```

The existing rational comparison and lower-ordinal tie are retained. The
original prompt tokens remain only so the sidecar serializes byte-identically.
Hamming symmetric difference is
`sum(popcount(query_word ^ prompt_word)) = union - intersection`.

No matrix, float or artifact change is used.

## Arms

1. `prompt_vsa`: deepseek-209 binary selector;
2. `literal_set`: the merged deepseek-207 release binary built from
   `a4981948e`, SHA-256
   `984422962013c46dd1d7ca07aa04805b7ed7bab4804296e9389cad541dd04f98`;
3. `cpu_oracle`: in-memory CPU arm used for row-level selected ordinal and
   reply SHA identity.

## Data split

- TRAIN panel `tt-2711`;
- sealed sidecars from deepseek-207 attempt 4;
- sealed seed-2777 panel `dd-2777`;
- both public artifacts;
- no new panel generation or reference fitting.

## Instrument gate

- source and binary pins match;
- `prompt_vsa` reports 64 signature words / 4,096 bits;
- the literal-set baseline binary hash matches;
- sidecar bytes and hashes remain identical to deepseek-207;
- both selectors produce the CPU oracle's ordinal and reply SHA on all 471
  rows per artifact;
- three alternating process runs per arm complete;
- API complete-hit, cap-fallback and metadata tests pass.

## Bar

For both artifacts:

- **471/471** promoted ordinals and reply SHA-256 values match;
- exact **39**, opening **131**, body **948**, emitted **471**, premature **0**;
- VSA median throughput is at least **2x** literal-set median and at least
  **25,000 prompts/s**;
- sidecar bytes and SHA-256 remain unchanged;
- all three VSA decision reports are identical after timing removal;
- API tests pass.

## Stop rule

Stop on source, binary, sidecar, panel, artifact, selection, parity, throughput
or determinism drift; fewer than 200 reachable rows; any row mismatch; failure
of either throughput floor; or any need to alter `UORLUT01`, serving kernels,
CUDA, float training or Claude-owned source.

## Cost

Local CPU only on the Ryzen 9 9950X. The full run is capped at 30 minutes,
2 GiB peak RSS and 32 MiB retained evidence. The twelve alternating selector
processes are bounded by the same deadline.

## Base and source

- base: `a4981948e`;
- implementation: `9f35dff7f796030c634a56522c4f1d2d4bf2c8ca`;
- prompt-reply representation:
  `485761293dd8d008a0eb662bcaf1647a7e2e6d7cc1f5efd24609d70902874dbc`;
- serving check:
  `8231efb03f8dda282e17c70a1ce24f44713965f52edbb23d78e32af652400b7c`;
- VSA binary:
  `2b862acb8d7c950acdc4e67e50c1290ad212754c9662ff33057bc78fc8e874e8`;
- literal baseline binary:
  `984422962013c46dd1d7ca07aa04805b7ed7bab4804296e9389cad541dd04f98`.

## Output

`local/deepseek/prompt-vsa/2026-11-06`

## Steps

Run only after FIT and GO.

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-prompt-vsa
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/prompt-vsa/2026-11-06
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/panels/tt-2711
PANEL=$SHARED/local/deepseek/prompt-retrieval/2026-11-05/panels/dd-2777
SIDECAR1=$SHARED/local/deepseek/prompt-reply-serving/2026-11-05-attempt-4/reports/seed1.sidecar.json
SIDECAR2=$SHARED/local/deepseek/prompt-reply-serving/2026-11-05-attempt-4/reports/seed2.sidecar.json
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
BUILD=$SOURCE/target/deepseek/prompt-vsa
BASELINE=$SHARED/target/release/cpu-prompt-reply-serving-check
IMPL=9f35dff7f

test ! -e "$OUT"
git -C "$SOURCE" diff --quiet "$IMPL" -- crates/uor-r4-integer crates/uor-r4-cpu-train
sha256sum -c <<EOF
984422962013c46dd1d7ca07aa04805b7ed7bab4804296e9389cad541dd04f98  $BASELINE
EOF
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
  --bin cpu-prompt-reply-serving-check
VSA=$BUILD/release/cpu-prompt-reply-serving-check
sha256sum -c <<EOF
2b862acb8d7c950acdc4e67e50c1290ad212754c9662ff33057bc78fc8e874e8  $VSA
EOF

run() {
  name=$1 bin=$2 artifact=$3 sidecar=$4 expected=$5
  /usr/bin/time -v -o "$OUT/reports/$name.time.txt" \
    timeout 1800s "$bin" "$sidecar" "$TRAIN" "$PANEL" "$artifact" "$TOKENIZER" \
    --protocol 2 --expected-artifact "$expected" \
    --expected-tokenizer a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 \
    >"$OUT/reports/$name.json"
}

for run in 1 2 3; do
  run "seed1-baseline-$run" "$BASELINE" "$SEED1" "$SIDECAR1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
  run "seed1-vsa-$run" "$VSA" "$SEED1" "$SIDECAR1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
  run "seed2-baseline-$run" "$BASELINE" "$SEED2" "$SIDECAR2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
  run "seed2-vsa-$run" "$VSA" "$SEED2" "$SIDECAR2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
done

for seed in seed1 seed2; do
  for run in 1 2 3; do
    base=$OUT/reports/$seed-baseline-$run.json
    vsa=$OUT/reports/$seed-vsa-$run.json
    jq -e '
      .reading == {
        "correct_body_positions":948,"emitted":471,"exact_reference":39,
        "generated_tokens":4960,"opening_correct":131,"premature_bos":0,
        "reference_reachable":471,"reference_tokens":4702,"replies":471
      }
      and .decision_sha256 == "a142efc85caae824c088441ff654e3c78ad828c6aab408ff1fb22d46d4fd558c"
      and .repeat_exact == true
    ' "$base" "$vsa"
    diff -u \
      <(jq -S 'del(.prompts_per_second,.selection_elapsed_us,.signature_words,.signature_bits,.representation)' "$base") \
      <(jq -S 'del(.prompts_per_second,.selection_elapsed_us,.signature_words,.signature_bits,.representation)' "$vsa")
  done
done

baseline_median=$(jq -s 'map(.prompts_per_second)|sort|.[length/2]' "$OUT"/reports/seed*-baseline-*.json)
vsa_median=$(jq -s 'map(.prompts_per_second)|sort|.[length/2]' "$OUT"/reports/seed*-vsa-*.json)
jq -n --argjson baseline "$baseline_median" --argjson vsa "$vsa_median" '
  {literal_set_prompts_per_second:$baseline,
   prompt_vsa_prompts_per_second:$vsa,
   speedup:($vsa/$baseline),
   speedup_at_least_2x:($vsa >= 2*$baseline),
   vsa_at_least_25000:($vsa >= 25000)}'
test "$(jq -n --argjson baseline "$baseline_median" --argjson vsa "$vsa_median" '$vsa >= 2*$baseline and $vsa >= 25000')" = true
printf 'KEEP\n' >"$OUT/outcome.txt"
test "$(du -sb "$OUT" | cut -f1)" -le 33554432
```

## Minimum test power

- both public artifacts;
- 471 reachable rows per artifact;
- 471/471 row-level ordinal and reply identity for both controls;
- three alternating fresh-process runs per arm;
- throughput floor of 25,000 prompts/s and 2x literal-set median;
- unchanged sidecar bytes and hash;
- focused API complete-hit and fallback tests.

## Outcomes

- **KEEP** if parity and both throughput floors pass on both artifacts.
- **REJECT** if the VSA representation fails parity or either speed floor.
- **CONTROL_FAULT** if the baseline binary, sidecar or output identity drifts.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 prompt-set binary VSA index
sheet: docs/harness/prep/cpu-training-pilot-prompt-vsa-2026-11-06.md
reviewer: deepseek exact-head self-review (independent delegation unavailable in this run)
note: Exact-head source, baseline and sidecar pins match; 474 serving library
  tests and 19 OpenAI loopback tests pass; binary signatures preserve exact
  AND/OR popcount Jaccard; output root absent.
date: 2026-11-06
A FIT verdict clears compute for the configuration in this sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone
before compute.

## Pre-run GO

GO - exact-head self-review under the current disabled-delegation policy,
2026-11-06. Implementation `9f35dff7f` is clean; the merged literal-set binary,
sidecars, panels, artifacts and tokenizer match their pins. The review
limitation is explicit: this was not an independent reviewer.
