# Parallel public 19-table CPU count driver

**Status:** FIT

## Question

Can the public 19-table count driver derive training rows and evaluate held-out
CPU/integer decisions with bounded Rayon workers while reproducing every
serial artifact, reading and checksum byte-for-byte, and does any bounded
worker count reduce wall time by at least 1.5 times on the Ryzen 9 9950X?

## Compute class

Local CPU development research under D84. No GPU, pod or paid API.

## Base artifact

| Input | Identity |
|---|---|
| implementation source | `e4bc08361c95b1b2c78a357c755c9c9812e42abe` |
| driver binary | `577195fb21f29b36ef2c49bc2b02af19d0eaa8a10a04d062abe8bb4005333b5e` |
| public M1 trunk | `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Inputs are under
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/`.
The binary was built cleanly with
`CARGO_TARGET_DIR=target/deepseek/trunk-parallel-build-126-timing cargo build --locked --release -p uor-r4-cpu-train --bin cpu-trunk-count-driver`.

## Data split

Replay the accepted deepseek-125 split and dose exactly:

- 4,096 deterministically ordered training rows;
- evaluation stride 2, at most 512 held-out rows;
- seed 1;
- no-update, exact additive-count and shuffled-target arms;
- workers 1, 8, 16 and 32, each in a fresh output root;
- two passes in opposite worker order: `1,8,16,32` then `32,16,8,1`,
  each with eight fresh roots and no concurrent runs.

Only `threads`, `wall_elapsed_ms` and `peak_rss_kib` may differ between the
eight reports. Exact and shuffled artifact bytes, table readings, decision
readings, checksums, changed-row lists and all other report fields must be
identical.

The accepted deepseek-125 result is the regression oracle, not merely another
worker run:

- exact artifact:
  `db4a672d6af83c4c56b7fbccc9ebc2279cb13fb8719b47829db64c910f7d2bb2`;
- shuffled artifact:
  `19cbb1caa487473ac4344e1b3582060fbcb22b1e6066fa51540bcbc8977eabcd`;
- no-update / exact / shuffled decision checksums:
  `c707e1bebfe590b0` / `351b70cab8d7cbc9` / `c707e1bebfe590b0`;
- exact and shuffled readings `331/245/269`; no update `332/246/270`;
- changed exact/shuffled rows versus no update: `4` / `0`;
- normalized report:
  `64e6e9b971ae0e90281aa485112db595e440eddeb29dbf99c9976a1fc467722f`,
  after replacing `threads` with `1` and deleting wall time and peak RSS.

## Bar

Primary performance bar:

- average each worker's two opposite-order `wall_elapsed_ms` values;
- at least one of 8, 16 or 32 workers has an average at most two-thirds of the
  one-worker average, a speedup of at least 1.5 times;
- every worker count satisfies all equivalence and resource gates below.

This is performance/instrument evidence only. A slower result closes no model
mechanism and says nothing about language capability.

## Instrument gates

- source and binary pins hold before and after each run;
- all four input hashes match the table above;
- fresh output roots and no source drift;
- finite tests reproduce rows, artifacts, decisions and the ordered checksum
  fold at workers 1, 8, 16 and 32, and RSS parsing rejects missing or zero data;
- exact and shuffled artifacts differ within every run;
- all 13 count tables increase by exactly 4,096 entries;
- independent full decoded table contents match every arm;
- codebook, routes, gates, recurrence and normalization sections are unchanged;
- every arm has zero query-coordinate, admitted-cell, top-8 and token
  mismatches;
- each report records its requested worker count;
- exact and shuffled artifacts are byte-identical across all eight runs;
- normalized reports are identical across all eight runs;
- every run records positive peak RSS below 2,097,152 KiB and stays below the
  600-second timeout.

## Minimum test power

The fixed evaluation panel has 512 document-disjoint rows, above D22's 200-row
floor. One public artifact is sufficient for this host-performance comparison
because deepseek-124 and deepseek-125 established the mechanism, controls and
exact CPU/integer result on it. The repeated opposite-order runs test scheduling
effects; they do not turn this performance measurement into fresh-transfer
evidence.

## Steps

Run only after FIT:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-trunk-parallel
DATA=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity
PIPELINE=/home/callard/projects/codex-playground/uor-r4/local/deepseek/trunk-parallel/2026-11-04-attempt-1
BIN="$SOURCE/target/deepseek/trunk-parallel-build-126-timing/release/cpu-trunk-count-driver"
SOURCE_PIN=e4bc08361c95b1b2c78a357c755c9c9812e42abe
BIN_SHA=577195fb21f29b36ef2c49bc2b02af19d0eaa8a10a04d062abe8bb4005333b5e
EXPECTED_EXACT=db4a672d6af83c4c56b7fbccc9ebc2279cb13fb8719b47829db64c910f7d2bb2
EXPECTED_SHUFFLED=19cbb1caa487473ac4344e1b3582060fbcb22b1e6066fa51540bcbc8977eabcd
EXPECTED_REPORT=64e6e9b971ae0e90281aa485112db595e440eddeb29dbf99c9976a1fc467722f
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
  if cmp -s "$out/exact.uorlut" "$DATA/t16-frozen-zero-seed1-trunk.d11"; then exit 1; fi
  if cmp -s "$out/exact.uorlut" "$out/shuffled.uorlut"; then exit 1; fi
  test "$(jq -S '.threads = 1 | del(.wall_elapsed_ms, .peak_rss_kib)' \
    "$out/report.json" | sha256sum | cut -d' ' -f1)" = "$EXPECTED_REPORT"
  jq -e '
    .tables.source | length == 13 and all(.[]; .total == 1049856)
  ' "$out/report.json" >/dev/null
  jq -e '
    (.tables.exact | length == 13 and all(.[]; .total == 1053952)) and
    (.tables.shuffled | length == 13 and all(.[]; .total == 1053952))
  ' "$out/report.json" >/dev/null
  jq -e '
    (.readings.no_update.admitted == 332) and
    (.readings.no_update.top1 == 246) and
    (.readings.no_update.top8 == 270) and
    (.readings.no_update.checksum == "c707e1bebfe590b0") and
    (.readings.exact.admitted == 331) and
    (.readings.exact.top1 == 245) and
    (.readings.exact.top8 == 269) and
    (.readings.exact.checksum == "351b70cab8d7cbc9") and
    (.readings.shuffled.admitted == 332) and
    (.readings.shuffled.top1 == 246) and
    (.readings.shuffled.top8 == 270) and
    (.readings.shuffled.checksum == "c707e1bebfe590b0") and
    ([.readings[] |
      .query_coordinate_mismatches, .admitted_cell_mismatches,
      .top8_mismatches, .token_mismatches] | all(. == 0)) and
    (.changed_decisions.exact_vs_no_update_rows == 4) and
    (.changed_decisions.shuffled_vs_no_update_rows == 0) and
    (.changed_decisions.exact_vs_shuffled_rows == 4)
  ' "$out/report.json" >/dev/null
}
run forward 1
run forward 8
run forward 16
run forward 32
run reverse 32
run reverse 16
run reverse 8
run reverse 1
for pass in forward reverse; do
  for workers in 8 16 32; do
    cmp "$PIPELINE/$pass/workers-1/exact.uorlut" \
      "$PIPELINE/$pass/workers-$workers/exact.uorlut"
    cmp "$PIPELINE/$pass/workers-1/shuffled.uorlut" \
      "$PIPELINE/$pass/workers-$workers/shuffled.uorlut"
  done
done
for workers in 1 8 16 32; do
  cmp "$PIPELINE/forward/workers-$workers/exact.uorlut" \
    "$PIPELINE/reverse/workers-$workers/exact.uorlut"
  cmp "$PIPELINE/forward/workers-$workers/shuffled.uorlut" \
    "$PIPELINE/reverse/workers-$workers/shuffled.uorlut"
done
wall() {
  jq -er '.wall_elapsed_ms' "$PIPELINE/$1/workers-$2/report.json"
}
mean_wall() {
  jq -n --argjson forward "$1" --argjson reverse "$2" \
    '($forward + $reverse) / 2'
}
one_worker=$(mean_wall "$(wall forward 1)" "$(wall reverse 1)")
speedup_passed=0
for workers in 8 16 32; do
  mean=$(mean_wall "$(wall forward "$workers")" "$(wall reverse "$workers")")
  speedup=$(jq -n --argjson one "$one_worker" --argjson worker "$mean" \
    '$one / $worker')
  printf 'workers=%s mean_wall_ms=%s speedup=%s\n' \
    "$workers" "$mean" "$speedup"
  if jq -e -n --argjson speedup "$speedup" '$speedup >= 1.5' >/dev/null; then
    speedup_passed=1
  fi
done
test "$speedup_passed" -eq 1
```

After the eight runs, compare artifact hashes and normalized reports; write the
comparison and checksum inventory under the task evidence folder.

## Cost and stop

Local workstation only. Rust 1.97.1 on Linux 7.0.0-42-generic x86_64. At
registration the project filesystem had 508 GiB free and `/tmp` had 25 GiB
free. Eight output roots retain about 1.4 GiB total. Each run has a 600-second
timeout and 2-GiB peak-RSS cap. Stop on cap failure, source drift, a worker
count mismatch, any artifact/decision/checksum mismatch, a non-vacuous control
failure or nested Rayon pools.

## Claim boundary

A positive supports only this driver's deterministic parallelism and measured
wall-time gain on this hardware and fixed workload. It is not fresh phrasing,
capability, count-training quality or GPU qualification.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 parallel public 19-table CPU count driver
sheet: `docs/harness/prep/cpu-training-pilot-trunk-parallel-2026-11-04.md`
head: `6cf73ce09a6c4be76424a4f849ab571ba716cd45`
implementation pin: `e4bc08361c95b1b2c78a357c755c9c9812e42abe`
binary pin: `577195fb21f29b36ef2c49bc2b02af19d0eaa8a10a04d062abe8bb4005333b5e`
reviewer: Codex fresh-context non-author session
note: eight fresh runs in opposite worker order; exact 125 artifact,
decision, reading and normalized-report oracle pins; one prebuilt local
Rayon pool; ordered worker-count tests; executable mean-wall 1.5x bar;
fail-closed positive RSS strictly below 2,097,152 KiB; 600-second timeout
date: 2026-11-04

A FIT verdict clears compute for the configuration in that sheet only (D22).
