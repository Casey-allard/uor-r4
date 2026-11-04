# Reuse loaded trunk models for CPU evaluation

**Status:** FIT

## Question

Can the public 19-table CPU count driver reuse its already-loaded CPU trunk
reader and integer model for the no-update held-out arm, eliminating two
duplicate full-artifact parses without changing any result?

## Compute class

Local CPU development research under D84 on the Ryzen 9 9950X. No GPU, pod or
paid API.

## Base artifact

| Input | Identity |
|---|---|
| implementation source | `83d1bbce86f272d4c396edeea14c1efba4275c6b` |
| baseline driver | `e32f99e8494ec51d26d523861ae4511ade3a73a5a03dbc29bd557105d3680dd4` |
| optimized driver | `a29541a5a9cdfdd3e07fbe6992bfbf54cbffe7f46f53fc897331b2b7fcc45d40` |
| public M1 trunk | `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Inputs are under
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/`.
The optimized binary was built with
`CARGO_TARGET_DIR=target/deepseek/eval-reuse-optimized cargo build --locked --release -p uor-r4-cpu-train --bin cpu-trunk-count-driver`.

## Data split

- 4,096 deterministically ordered training rows;
- evaluation stride 2 and at most 512 held-out rows;
- seed 1;
- no-update, exact and deterministic shuffled-target arms;
- workers 1 and 16;
- pass A: baseline 1, optimized 1, baseline 16, optimized 16;
- pass B: optimized 16, baseline 16, optimized 1, baseline 1;
- every invocation has a fresh output root and runs sequentially.

Only `stages`, `threads`, `wall_elapsed_ms` and `peak_rss_kib` may differ from
the accepted result. After replacing `threads` with `1` and deleting those
fields, every report must hash to
`64e6e9b971ae0e90281aa485112db595e440eddeb29dbf99c9976a1fc467722f`.
Every exact and shuffled artifact must match
`db4a672d6af83c4c56b7fbccc9ebc2279cb13fb8719b47829db64c910f7d2bb2`
and `19cbb1caa487473ac4344e1b3582060fbcb22b1e6066fa51540bcbc8977eabcd`.

## Bar

Average the two orders separately for baseline and optimized runs at each
worker count.

Both must hold:

- optimized mean wall is at most 95% of baseline mean wall at 1 and at 16;
- optimized mean `no_update_evaluation` stage time is at most 50% of baseline
  at both worker counts.

Every artifact, table, decision, checksum and stage-accounting gate must also
pass. This is performance/instrument evidence, not a capability or
GPU-qualification claim.

## Instrument gates

- source and both binary pins hold before and after every run;
- all four input hashes match;
- every run has exactly the canonical 21 finite, non-negative stages and
  accounting equal to measured wall within 0.001 ms;
- every report records its requested worker count and positive RSS strictly
  below 2,097,152 KiB;
- every run completes below the 600-second timeout;
- every non-stage report field and both artifacts match the accepted oracle;
- all eight artifact pairs are byte-identical across binaries, passes and
  worker counts.

## Minimum test power

The fixed panel has 512 document-disjoint rows, above D22's 200-row floor.
Two order-balanced runs per binary and worker count control host drift while
measuring this implementation change. The result is bounded workstation
performance evidence.

## Steps

Run only after FIT:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-trunk-eval-reuse
DATA=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity
PIPELINE=/home/callard/projects/codex-playground/uor-r4/local/deepseek/trunk-eval-reuse/2026-11-04-attempt-1
OLD_BIN="$SOURCE/target/deepseek/eval-reuse-baseline/release/cpu-trunk-count-driver"
NEW_BIN="$SOURCE/target/deepseek/eval-reuse-optimized/release/cpu-trunk-count-driver"
SOURCE_PIN=83d1bbce86f272d4c396edeea14c1efba4275c6b
OLD_SHA=e32f99e8494ec51d26d523861ae4511ade3a73a5a03dbc29bd557105d3680dd4
NEW_SHA=a29541a5a9cdfdd3e07fbe6992bfbf54cbffe7f46f53fc897331b2b7fcc45d40
EXPECTED_EXACT=db4a672d6af83c4c56b7fbccc9ebc2279cb13fb8719b47829db64c910f7d2bb2
EXPECTED_SHUFFLED=19cbb1caa487473ac4344e1b3582060fbcb22b1e6066fa51540bcbc8977eabcd
EXPECTED_REPORT=64e6e9b971ae0e90281aa485112db595e440eddeb29dbf99c9976a1fc467722f
STAGES='["control_rows","corpus_split","decision_reduction","exact_evaluation","exact_hash_write","exact_update_export","exact_verify","input_read_hash","input_validate","no_update_evaluation","output_prepare","parity_reduction","reader_load","resource_measure","shuffled_evaluation","shuffled_hash_write","shuffled_reader_load","shuffled_update_export","shuffled_verify","table_verification","training_row_derivation"]'
check_pins() {
  test -z "$(git -C "$SOURCE" status --porcelain -- crates Cargo.toml Cargo.lock)"
  git -C "$SOURCE" diff --quiet "$SOURCE_PIN" -- crates Cargo.toml Cargo.lock
  test "$(sha256sum "$OLD_BIN" | cut -d' ' -f1)" = "$OLD_SHA"
  test "$(sha256sum "$NEW_BIN" | cut -d' ' -f1)" = "$NEW_SHA"
  test "$(sha256sum "$DATA/t16-frozen-zero-seed1-trunk.d11" | cut -d' ' -f1)" = \
    04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d
  test "$(sha256sum "$DATA/dev-tokens.u16" | cut -d' ' -f1)" = \
    0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b
  test "$(sha256sum "$DATA/dev-response-mask.u8" | cut -d' ' -f1)" = \
    a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078
  test "$(sha256sum "$DATA/dev-manifest.json" | cut -d' ' -f1)" = \
    ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab
}
check_pins
test ! -e "$PIPELINE"
run() {
  pass=$1
  binary=$2
  workers=$3
  case "$binary" in
    baseline) bin=$OLD_BIN ;;
    optimized) bin=$NEW_BIN ;;
    *) exit 2 ;;
  esac
  out="$PIPELINE/$pass/$binary-workers-$workers"
  check_pins
  /usr/bin/time -v timeout 600s "$bin" \
    "$DATA/t16-frozen-zero-seed1-trunk.d11" \
    "$DATA/dev-tokens.u16" "$DATA/dev-response-mask.u8" \
    "$DATA/dev-manifest.json" "$out" \
    --expected-artifact-sha256 04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d \
    --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
    --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
    --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
    --train-rows 4096 --evaluation-stride 2 --max-evaluation 512 \
    --seed 1 --workers "$workers" >/dev/null
  check_pins
  test "$(jq -er '.threads' "$out/report.json")" -eq "$workers"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -gt 0
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -lt 2097152
  test "$(jq -er '.stages.elapsed_ms | length' "$out/report.json")" -eq 21
  test "$(jq -c '.stages.elapsed_ms | keys' "$out/report.json")" = "$STAGES"
  jq -e '
    ([.stages.elapsed_ms[]] | all(type == "number" and . >= 0)) and
    ((.stages.attributed_ms + .stages.unattributed_ms -
      .wall_elapsed_ms) | fabs <= 0.001) and
    (.artifacts.exact_sha256 == "'"$EXPECTED_EXACT"'") and
    (.artifacts.shuffled_sha256 == "'"$EXPECTED_SHUFFLED"'")
  ' "$out/report.json" >/dev/null
  test "$(jq -S '.threads = 1 | del(.wall_elapsed_ms, .peak_rss_kib, .stages)' \
    "$out/report.json" | sha256sum | cut -d' ' -f1)" = "$EXPECTED_REPORT"
  test "$(sha256sum "$out/exact.uorlut" | cut -d' ' -f1)" = "$EXPECTED_EXACT"
  test "$(sha256sum "$out/shuffled.uorlut" | cut -d' ' -f1)" = "$EXPECTED_SHUFFLED"
}
run forward baseline 1
run forward optimized 1
run forward baseline 16
run forward optimized 16
run reverse optimized 16
run reverse baseline 16
run reverse optimized 1
run reverse baseline 1
for pass in forward reverse; do
  for workers in 1 16; do
    cmp "$PIPELINE/$pass/baseline-workers-$workers/exact.uorlut" \
      "$PIPELINE/$pass/optimized-workers-$workers/exact.uorlut"
    cmp "$PIPELINE/$pass/baseline-workers-$workers/shuffled.uorlut" \
      "$PIPELINE/$pass/optimized-workers-$workers/shuffled.uorlut"
  done
done
wall() {
  jq -er '.wall_elapsed_ms' \
    "$PIPELINE/$1/$2-workers-$3/report.json"
}
no_update_evaluation() {
  jq -er '.stages.elapsed_ms.no_update_evaluation' \
    "$PIPELINE/$1/$2-workers-$3/report.json"
}
mean() {
  jq -n --argjson a "$1" --argjson b "$2" '($a + $b) / 2'
}
for workers in 1 16; do
  old_wall=$(mean "$(wall forward baseline "$workers")" \
    "$(wall reverse baseline "$workers")")
  new_wall=$(mean "$(wall forward optimized "$workers")" \
    "$(wall reverse optimized "$workers")")
  old_evaluation=$(mean \
    "$(no_update_evaluation forward baseline "$workers")" \
    "$(no_update_evaluation reverse baseline "$workers")")
  new_evaluation=$(mean \
    "$(no_update_evaluation forward optimized "$workers")" \
    "$(no_update_evaluation reverse optimized "$workers")")
  printf 'workers=%s baseline_wall=%s optimized_wall=%s baseline_no_update=%s optimized_no_update=%s\n' \
    "$workers" "$old_wall" "$new_wall" "$old_evaluation" "$new_evaluation"
  jq -e -n --argjson old "$old_wall" --argjson new "$new_wall" \
    '$new <= $old * 0.95' >/dev/null
  jq -e -n --argjson old "$old_evaluation" --argjson new "$new_evaluation" \
    '$new <= $old * 0.5' >/dev/null
done
```

## Cost and stop

Local workstation only. Rust 1.97.1 on Linux 7.0.0-42-generic x86_64. Eight
output roots retain about 1.4 GiB. Each run has a 600-second timeout and a
strict 2-GiB peak-RSS stop. Stop on source/binary/input drift, artifact or
decision mismatch, invalid accounting, retained-model memory above the cap,
either performance bar failing, or overlap with Claude-owned files.

## Claim boundary

A pass supports only this reuse change on the fixed public CPU workload and
this host. It does not establish a universal speedup, training benefit,
language capability or GPU qualification.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 public 19-table CPU evaluation reuse
sheet: `docs/harness/prep/cpu-training-pilot-trunk-eval-reuse-2026-11-04.md`
docs head: `b0f2a7aa572f9cf95c2f535b6556fde630ed8069`
implementation pin: `1f5fe3c3a4aee2e67de64276a4712cd0fbd23a7f`
baseline binary: `e32f99e8494ec51d26d523861ae4511ade3a73a5a03dbc29bd557105d3680dd4`
optimized binary: `a29541a5a9cdfdd3e07fbe6992bfbf54cbffe7f46f53fc897331b2b7fcc45d40`
reviewer: Codex fresh-context non-author session
note: no-update reuses the preloaded CPU/integer models; exact and shuffled
independently reload; real D11 finite equivalence test; mirrored one/sixteen
worker order; canonical 21 stages; direct artifact hashes; full-crates source
pin; strict positive RSS below 2,097,152 KiB; 600-second timeout; 5-percent wall
and 50-percent no-update-evaluation bars; instrument-only claim
date: 2026-11-04

A FIT verdict clears compute for the configuration in that sheet only (D22).

The first review passed the implementation at source `1f5fe3c3…` and docs
head `b0f2a7aa…`, but local policy then found that the ORDER's `Item:` link was
split across lines. The unpublished branch was rebased to repair that
metadata-only defect; the source tree and both binaries are byte-identical,
with the source now at `83d1bbce…`. A fresh review is required at the corrected
head.

TEST FITNESS: FIT - M2 #2030 repaired-metadata public CPU evaluation reuse
sheet: `docs/harness/prep/cpu-training-pilot-trunk-eval-reuse-2026-11-04.md`
docs head: `9adf5e8b5e50feadf39eec1ca0e72ee8dcb2aa86`
implementation pin: `83d1bbce86f272d4c396edeea14c1efba4275c6b`
baseline binary: `e32f99e8494ec51d26d523861ae4511ade3a73a5a03dbc29bd557105d3680dd4`
optimized binary: `a29541a5a9cdfdd3e07fbe6992bfbf54cbffe7f46f53fc897331b2b7fcc45d40`
reviewer: Codex second fresh-context non-author session
note: ORDER parses; source tree, Cargo manifests, binaries, technical gates
and completed eight-run panel are unchanged from the first FIT; all 27 evidence
hashes and eight report gates pass; no timing rerun
date: 2026-11-04

A FIT verdict clears compute for the configuration in that sheet only (D22).
