# Public 19-table CPU count stage profile

**Status:** FIT

## Question

After deepseek-126 found only a 1.065x gain from parallel row derivation and
held-out evaluation, which stages of `cpu-trunk-count-driver` consume the
remaining wall time, and is the largest stage stable across one and sixteen
workers and both run orders?

## Compute class

Local CPU development research under D84 on the Ryzen 9 9950X. No GPU, pod or
paid API.

## Base artifact

| Input | Identity |
|---|---|
| implementation source | `f0c0e608ca41324ce4fa76305c83e57689426e60` |
| driver binary | `e4dac42e783958a082ebfe0b3cf63ec4f54ea149d73a19764114878559c14ada` |
| public M1 trunk | `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Inputs are under
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/`.
The binary was built with
`CARGO_TARGET_DIR=target/deepseek/trunk-stages-build-2 cargo build --locked --release -p uor-r4-cpu-train --bin cpu-trunk-count-driver`.

## Data split

- 4,096 deterministically ordered training rows;
- evaluation stride 2 and at most 512 held-out rows;
- seed 1;
- no-update, exact and shuffled-target arms;
- workers 1 and 16 in forward order `1,16`, then workers 16 and 1 in reverse
  order, each in a fresh output root.

Only `stages`, `threads`, `wall_elapsed_ms` and `peak_rss_kib` may differ from
deepseek-126. After replacing `threads` with `1` and deleting those fields, each
report must equal
`64e6e9b971ae0e90281aa485112db595e440eddeb29dbf99c9976a1fc467722f`.
Exact and shuffled artifacts must retain:

- exact:
  `db4a672d6af83c4c56b7fbccc9ebc2279cb13fb8719b47829db64c910f7d2bb2`;
- shuffled:
  `19cbb1caa487473ac4344e1b3582060fbcb22b1e6066fa51540bcbc8977eabcd`.

## Bar

This is an instrument profile, not a speed threshold.

The measurement passes only if all four runs have exact identity, finite
non-negative stage durations, 21 required stages, and
`attributed_ms + unattributed_ms == wall_elapsed_ms` within 0.001 ms.

For interpretation, average the two orders at each worker count. Name a stage
as the measured hotspot only if it is the largest at both worker counts. If the
top stage differs or the top two are within 10%, name and report the leading
group instead of selecting one stage post hoc.

## Instrument gates

- source, binary and four input pins hold before and after every run;
- each run completes below 600 seconds with positive RSS strictly below
  2,097,152 KiB;
- exact and shuffled artifacts differ and match the deepseek-126 hashes;
- all tables, readings, checksums and changed-decision fields reproduce
  deepseek-126 after deleting only the four permitted fields;
- all stage values are finite and non-negative;
- the 21 stage names are present exactly once;
- attributed and unattributed time equal measured wall time within 0.001 ms;
- artifacts are byte-identical across all four runs;
- stage results are reported for both orders and both worker counts.

## Minimum test power

The fixed panel has 512 document-disjoint rows, above D22's 200-row floor.
Two worker counts and both run orders bound scheduling effects for this
instrument question. The result is development timing evidence, not fresh
transfer, capability or GPU qualification.

## Steps

Run only after FIT:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-trunk-stages
DATA=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity
PIPELINE=/home/callard/projects/codex-playground/uor-r4/local/deepseek/trunk-stages/2026-11-04-attempt-2
BIN="$SOURCE/target/deepseek/trunk-stages-build-2/release/cpu-trunk-count-driver"
SOURCE_PIN=f0c0e608ca41324ce4fa76305c83e57689426e60
BIN_SHA=e4dac42e783958a082ebfe0b3cf63ec4f54ea149d73a19764114878559c14ada
EXPECTED_EXACT=db4a672d6af83c4c56b7fbccc9ebc2279cb13fb8719b47829db64c910f7d2bb2
EXPECTED_SHUFFLED=19cbb1caa487473ac4344e1b3582060fbcb22b1e6066fa51540bcbc8977eabcd
EXPECTED_REPORT=64e6e9b971ae0e90281aa485112db595e440eddeb29dbf99c9976a1fc467722f
STAGES='["control_rows","corpus_split","decision_reduction","exact_evaluation","exact_hash_write","exact_update_export","exact_verify","input_read_hash","input_validate","no_update_evaluation","output_prepare","parity_reduction","reader_load","resource_measure","shuffled_evaluation","shuffled_hash_write","shuffled_reader_load","shuffled_update_export","shuffled_verify","table_verification","training_row_derivation"]'
check_pins() {
  test -z "$(git -C "$SOURCE" status --porcelain -- crates Cargo.toml Cargo.lock)"
  git -C "$SOURCE" diff --quiet "$SOURCE_PIN" -- \
    crates/uor-r4-cpu-train Cargo.toml Cargo.lock
  test "$(sha256sum "$BIN" | cut -d' ' -f1)" = "$BIN_SHA"
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
  workers=$2
  out="$PIPELINE/$pass/workers-$workers"
  check_pins
  /usr/bin/time -v timeout 600s "$BIN" \
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
  test "$(jq -er '.artifacts.exact_sha256' "$out/report.json")" = "$EXPECTED_EXACT"
  test "$(jq -er '.artifacts.shuffled_sha256' "$out/report.json")" = "$EXPECTED_SHUFFLED"
  test "$(jq -er '.stages.elapsed_ms | length' "$out/report.json")" -eq 21
  test "$(jq -c '.stages.elapsed_ms | keys' "$out/report.json")" = "$STAGES"
  jq -e '
    ([.stages.elapsed_ms[]] | all(type == "number" and . >= 0)) and
    ((.stages.attributed_ms + .stages.unattributed_ms -
      .wall_elapsed_ms) | fabs <= 0.001)
  ' "$out/report.json" >/dev/null
  test "$(jq -S '.threads = 1 | del(.wall_elapsed_ms, .peak_rss_kib, .stages)' \
    "$out/report.json" | sha256sum | cut -d' ' -f1)" = "$EXPECTED_REPORT"
  if cmp -s "$out/exact.uorlut" "$out/shuffled.uorlut"; then exit 1; fi
}
run forward 1
run forward 16
run reverse 16
run reverse 1
for pass in forward reverse; do
  cmp "$PIPELINE/$pass/workers-1/exact.uorlut" \
    "$PIPELINE/$pass/workers-16/exact.uorlut"
  cmp "$PIPELINE/$pass/workers-1/shuffled.uorlut" \
    "$PIPELINE/$pass/workers-16/shuffled.uorlut"
done
for workers in 1 16; do
  cmp "$PIPELINE/forward/workers-$workers/exact.uorlut" \
    "$PIPELINE/reverse/workers-$workers/exact.uorlut"
  cmp "$PIPELINE/forward/workers-$workers/shuffled.uorlut" \
    "$PIPELINE/reverse/workers-$workers/shuffled.uorlut"
done
```

## Cost and stop

Local workstation only. Rust 1.97.1 on Linux 7.0.0-42-generic x86_64. Four
output roots retain about 700 MiB. Each run has a 600-second timeout and a
strict 2-GiB peak-RSS stop. Stop on input/source/binary drift, artifact or
decision mismatch, invalid stage accounting, cap failure or overlap with
Claude-owned files.

## Claim boundary

A result identifies where this fixed count-driver workload spends time on this
host. It does not establish a universal CPU bottleneck, a training benefit,
language capability or GPU qualification.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 public 19-table CPU count stage profile
sheet: `docs/harness/prep/cpu-training-pilot-trunk-stages-2026-11-04.md`
head: `1994b289b1e15bddbb7305cccc17be3f8feba706`
implementation pin: `f0c0e608ca41324ce4fa76305c83e57689426e60`
binary pin: `e4dac42e783958a082ebfe0b3cf63ec4f54ea149d73a19764114878559c14ada`
reviewer: Codex fresh-context non-author session
note: canonical 21-stage set fails closed on duplicate, missing or extra stages;
exact deepseek-126 artifact and normalized non-stage report pins; mirrored
one/sixteen-worker runs; finite accounting; strict positive RSS below
2,097,152 KiB; 600-second timeout; predeclared hotspot rule and instrument-only
claim
date: 2026-11-04

A FIT verdict clears compute for the configuration in that sheet only (D22).
