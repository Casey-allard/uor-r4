# Consolidate public 19-table CPU verification

**Status:** FIT

## Question

Can `cpu-trunk-count-driver` reduce full-trunk decoding in exact and shuffled
verification from ten imports to four, preserving independent table
reconstruction and every accepted result, while reducing mean total wall time
by at least 10% and verification-stage time by at least 20% at both one and
sixteen workers?

## Compute class

Local CPU development research under D84 on the Ryzen 9 9950X. No GPU, pod or
paid API.

## Base artifact

| Input | Identity |
|---|---|
| implementation source | `be0edd4682409398ebf2e7f86b4b44ec987f8edf` |
| baseline driver | `e4dac42e783958a082ebfe0b3cf63ec4f54ea149d73a19764114878559c14ada` |
| optimized driver | `e32f99e8494ec51d26d523861ae4511ade3a73a5a03dbc29bd557105d3680dd4` |
| public M1 trunk | `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Inputs are under
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/`.
The optimized binary was built with
`CARGO_TARGET_DIR=target/deepseek/trunk-verify-optimized-final cargo build --locked --release -p uor-r4-cpu-train --bin cpu-trunk-count-driver`.

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
and `19cbb1caa487473ac4344e1b3582060fbcb22b1e6066fa51540bcbc8977eabcd`
respectively.

## Bar

Average the two orders separately for baseline and optimized runs at each
worker count.

Both must hold:

- optimized mean wall is at most 90% of baseline mean wall at 1 and at 16;
- optimized mean `exact_verify + shuffled_verify` time is at most 80% of the
  baseline verification time at 1 and at 16.

Every artifact, decision, table, checksum and stage-accounting gate must also
pass. This is performance/instrument evidence, not a capability or
GPU-qualification claim.

## Instrument gates

- source and both binary pins hold before and after every run;
- all four input hashes match;
- every run has 21 finite non-negative stages and accounting equal to measured
  wall within 0.001 ms;
- every report records its requested worker count and positive RSS strictly
  below 2,097,152 KiB;
- every run completes below the 600-second timeout;
- every non-stage report field and artifact matches the accepted oracle;
- both binary outputs are byte-identical across passes and workers;
- baseline and optimized artifacts are byte-identical to each other.

## Minimum test power

The fixed panel has 512 document-disjoint rows, above D22's 200-row floor.
Two order-balanced runs per binary and worker count control host drift while
measuring this implementation change. The result is bounded workstation
performance evidence.

## Steps

Run only after FIT:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-trunk-verify
DATA=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity
PIPELINE=/home/callard/projects/codex-playground/uor-r4/local/deepseek/trunk-verify/2026-11-04-attempt-3
OLD_BIN="$SOURCE/target/deepseek/trunk-verify-baseline/release/cpu-trunk-count-driver"
NEW_BIN="$SOURCE/target/deepseek/trunk-verify-optimized-final/release/cpu-trunk-count-driver"
SOURCE_PIN=be0edd4682409398ebf2e7f86b4b44ec987f8edf
OLD_SHA=e4dac42e783958a082ebfe0b3cf63ec4f54ea149d73a19764114878559c14ada
NEW_SHA=e32f99e8494ec51d26d523861ae4511ade3a73a5a03dbc29bd557105d3680dd4
EXPECTED_EXACT=db4a672d6af83c4c56b7fbccc9ebc2279cb13fb8719b47829db64c910f7d2bb2
EXPECTED_SHUFFLED=19cbb1caa487473ac4344e1b3582060fbcb22b1e6066fa51540bcbc8977eabcd
EXPECTED_REPORT=64e6e9b971ae0e90281aa485112db595e440eddeb29dbf99c9976a1fc467722f
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
verification() {
  jq -er '.stages.elapsed_ms.exact_verify + .stages.elapsed_ms.shuffled_verify' \
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
  old_verify=$(mean "$(verification forward baseline "$workers")" \
    "$(verification reverse baseline "$workers")")
  new_verify=$(mean "$(verification forward optimized "$workers")" \
    "$(verification reverse optimized "$workers")")
  printf 'workers=%s baseline_wall=%s optimized_wall=%s baseline_verify=%s optimized_verify=%s\n' \
    "$workers" "$old_wall" "$new_wall" "$old_verify" "$new_verify"
  jq -e -n --argjson old "$old_wall" --argjson new "$new_wall" \
    '$new <= $old * 0.9' >/dev/null
  jq -e -n --argjson old "$old_verify" --argjson new "$new_verify" \
    '$new <= $old * 0.8' >/dev/null
done
```

## Cost and stop

Local workstation only. Rust 1.97.1 on Linux 7.0.0-42-generic x86_64. Eight
output roots retain about 1.4 GiB. Each run has a 600-second timeout and a
strict 2-GiB peak-RSS stop. Stop on source/binary/input drift, artifact or
decision mismatch, invalid accounting, either performance bar failing, a cap
failure or overlap with Claude-owned files.

## Claim boundary

A pass supports only one consolidated verification implementation on the fixed
public CPU workload and this host. It does not establish a universal speedup,
training quality, language capability or GPU qualification.

## TEST FITNESS

First fresh-context non-author review: NOT FIT at docs head
`ddcfdfd3170988620ebab696f2289b5c26516dc5`. The reviewer required a finite
test that imports and checks real trunk artifacts, direct hashes of every
generated artifact against the accepted oracle, and removal of the conflicting
premature FIT status.

Second fresh-context non-author review: NOT FIT at docs head
`5b4149dc4d831e8df7b2660ea28842f07a9b622e`. The reviewer found that the source
pin covered only the CPU crate although the implementation also touched
`crates/uor-r4-discrete`. The source gate now covers all `crates/**` and both
root Cargo manifests.

TEST FITNESS: FIT - M2 #2030 public 19-table CPU verification consolidation
sheet: `docs/harness/prep/cpu-training-pilot-trunk-verify-2026-11-04.md`
docs head: `840ae00303d92c45c2688ae8f402991a98e45ae2`
implementation pin: `9717f52ba4b60eab6eae39273dbc7ee7a0239c3c`
baseline binary: `e4dac42e783958a082ebfe0b3cf63ec4f54ea149d73a19764114878559c14ada`
optimized binary: `c92e2d2a101b64c13751fda099c41f8e88994ebc1627046332b863cdececba04`
reviewer: Codex third fresh-context non-author session
note: four verification imports; exact source tables moved into reconstruction
and shuffled expectations independently rebuilt from a fresh source import;
actual D11 finite success and mismatch controls; direct artifact oracles on all
eight runs; full-crates source pin; mirrored one/sixteen-worker order; 21
finite stages; strict positive RSS below 2,097,152 KiB; 600-second timeout;
10-percent wall and 20-percent verification bars; instrument-only claim
date: 2026-11-04

The first executed panel used that reviewed source and its
`c92e2d2a…ba04` binary. A post-run package-format check then normalized three
test-only expressions in `fcb7c8c5327800cb372529926ada62e6ba5cfb55`. Rebuilding
changed the linked binary hash to `8517f0c1…87df`, despite no production-code
change. The first attempt is retained as a passing diagnostic, but this fixed
experiment will use a fresh fourth review, fresh binary pin and fresh output
root so the delivered source, binary and timing record are aligned exactly.

TEST FITNESS: FIT - M2 #2030 formatted public 19-table CPU verification panel
sheet: `docs/harness/prep/cpu-training-pilot-trunk-verify-2026-11-04.md`
docs head: `befb6888b9042fac7fdfe1804ed6dcbb1e082127`
implementation pin: `fcb7c8c5327800cb372529926ada62e6ba5cfb55`
baseline binary: `e4dac42e783958a082ebfe0b3cf63ec4f54ea149d73a19764114878559c14ada`
optimized binary: `8517f0c1aeb69a9a0da3137e561233e7d5d83f1f3180b5233b42ec60bb2087df`
reviewer: Codex fourth fresh-context non-author session
note: production verification code unchanged from the prior FIT; test-only
formatting source and rebuilt binary are clean and pinned; four imports,
independent shuffled reconstruction, real D11 finite success/mismatch test,
direct artifact hashes on all eight fresh runs, full-crates source pin, mirrored
one/sixteen-worker order, 21 finite stages, strict positive RSS below
2,097,152 KiB, 600-second timeout, and 10-percent wall plus 20-percent
verification bars remain executable
date: 2026-11-04

A FIT verdict clears compute for the configuration in that sheet only (D22).

An exact-head review then found that the synthetic test's public constants put
`crates/uor-r4-discrete/src/trunk.rs` outside the item scope. The test now uses
the existing public kappa constants in `uor_r4_discrete::q4_import`, and
`trunk.rs` is identical to `origin/main`. The production binary changed only
through test-support source and is newly pinned at `e32f99e8…0dd4`; a fresh
output root and review are required.

TEST FITNESS: FIT - M2 #2030 in-scope public 19-table verification panel
sheet: `docs/harness/prep/cpu-training-pilot-trunk-verify-2026-11-04.md`
docs head: `ba0ca6559502739fed48261403f727e63399b83d`
implementation pin: `be0edd4682409398ebf2e7f86b4b44ec987f8edf`
baseline binary: `e4dac42e783958a082ebfe0b3cf63ec4f54ea149d73a19764114878559c14ada`
optimized binary: `e32f99e8494ec51d26d523861ae4511ade3a73a5a03dbc29bd557105d3680dd4`
reviewer: Codex fifth fresh-context non-author session
note: in-scope test access uses existing q4_import kappa constants; no diff
from origin/main in `uor-r4-discrete`; four verification imports; independent
shuffled reconstruction; real D11 success/mismatch test; direct artifact hashes
on all eight fresh runs; full-crates source pin; mirrored one/sixteen-worker
order; 21 finite stages; strict positive RSS below 2,097,152 KiB; 600-second
timeout; 10-percent wall and 20-percent verification bars
date: 2026-11-04

A FIT verdict clears compute for the configuration in that sheet only (D22).
