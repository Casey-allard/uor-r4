# Generated-state content stop gate

**Status:** FIT

## Question

Can a separate two-class Hamming prototype gate over the positive modulo-256
content arm's generated states emit BOS at held-out reference endpoints
without changing content choices before the gate fires?

This is D84 CPU development research on the Ryzen 9 9950X. It does not change
integer serving and is not a language or GPU-qualification claim.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

Revision 2 supersedes implementation pin `18b0bc8c0` after delivery review
found that the driver used the total discordant count instead of the
registered control-only count in its paired thresholds. The attempt-1 run is
retained as superseded instrument evidence; it does not decide this
configuration.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `249b296072c2d716019af487dbb0afaa38281e55` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Split and mechanism

Use deepseek-137's split, memory and positive 2,048-bit modulo-256 code
readout. Generate every training reply with that exact code arm.

At each generated decision, label the row terminal when its offset equals
normalized reference length and nonterminal otherwise. Store the complete
code signature and the code arm's content fallback token.

For each class, deduplicate signatures, select a Hamming medoid, then seven
farthest-first prototypes with lexicographic ties. Score
`nearest_nonterminal - nearest_terminal`; emit BOS at or above a training-only
margin. Fit the margin to maximize terminal recall with nonterminal false
stops at or below 1%, then lower false stops, then the larger margin.

## Preflight

Seed 1 produced 1,568 generated rows: 84 terminal and 1,484 nonterminal.
Eight prototypes per class selected a margin of 982, recalling 8/84 terminals
with 9/1,484 false stops. Wider preflight prototype sets did not make the
registered 26/84 training bar:

| Prototypes/class | Terminal stops | False stops | Margin |
|---:|---:|---:|---:|
| 8 | 8 | 9 | 982 |
| 16 | 11 | 13 | 980 |
| 32 | 5 | 13 | 986 |
| 64 | 9 | 14 | 978 |
| 84 | 12 | 12 | 972 |

The fixed eight-prototype configuration is retained for the registered run;
the wider values are reported as preflight evidence, not silently used to
rescue the bar.

## Arms and bar

Run no-gate code, true terminal/nonterminal prototypes and
class-preserving shuffled-label prototypes over 16-token replies.

Both seeds must pass:

- both classes are present and every row carries 2,048 bits;
- terminal recall is at least **26/84** with at most **1%** nonterminal false
  stops;
- true BOS completion is at least **9/103**, with at least **8** at or before
  the normalized reference length;
- at least **8/103** replies exactly match the reference and stop at BOS;
- true BOS completion exceeds no-gate and shuffled by the prompt-level
  `candidate_only >= max(2, ceil(3 * sqrt(control_only)))` rule;
- true top-1 and depth are not below no-gate;
- every ungated decision is the code arm's content token and every fired
  decision is BOS;
- true terminal recall beats shuffled by
  `candidate_only >= max(2, ceil(2 * sqrt(control_only)))`; and
- seed-1 repeat matches every non-timing report and generated JSON file.

Report prototype hashes, margins, confusion counts, BOS stops at exact,
shorter and longer than the reference length, top-1/depth, openings, exact
references, distinct/repeated replies, workers, wall and RSS.

## Minimum test power

The split and two model seeds are fixed. This compares the gate against the
exact content arm it observes and a class-preserving label control. The 26/84
training floor is one quarter of generated endpoints; free-generation floors
require complete and exact returned replies.

## Instrument gates

- Source and five public inputs match.
- Terminal rows are exactly generated offsets at normalized reference length;
  all other generated rows are nonterminal.
- Prototype selection, margin fit and label shuffle are deterministic.
- Fired decisions are BOS and ungated token mismatches are zero.
- Evaluation is source-ordered and identical across one and 32 workers.
- Gate and shuffled-gate hashes, margins and training confusion counts are
  retained.

## Stop rules

Stop on source/input/split drift, missing classes, vacuous label shuffle,
ungated token mismatch, worker nondeterminism, resource breach or any need to
change serving, integer-session, CUDA or Claude-owned files.

## Decision consequence

A pass makes a separate generated-state nonparametric stop gate a serving
contract candidate. A fail closes eight-prototype code-signature stop
classification at the 1% false-stop limit and redirects termination work to
another representation or continuation objective. The positive content code
is unchanged.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT and pre-run review | 600 s |
| release build | 300 s |
| two seeds plus repeat | 600 s |
| delivery and evidence | 300 s |
| total increment | **1,800 s** |

- Build: 16 jobs, 4 GiB RSS, 2 GiB target maximum.
- Runs: 32 Rayon workers, 1 GiB RSS, 600 s each.
- Retained pipeline: 160 MiB maximum.
- Previous ledger:
  `cpu-training-pilot-content-stop-2026-11-04/resource-ledger.json` at
  `35,333.377487699 s` measured and `42,127.677487699 s` admitted cumulative.
- Admitted cumulative projection: **43,927.677487699 s**.

## Steps

After FIT and a fresh GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-content-stop-gate
SHARED=/home/callard/projects/codex-playground/uor-r4
PIPELINE=$SHARED/local/deepseek/content-stop-gate/2026-11-04-attempt-2
BUILD=$SOURCE/target/deepseek/content-stop-gate-139-attempt-2
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
IMPL_PIN=249b296072c2d716019af487dbb0afaa38281e55
MIN_FREE_KIB=31588352
free_kib() { df -Pk "$1" | awk 'NR == 2 { print $4 }'; }
test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
test -z "$(git -C "$SOURCE" status --porcelain -- crates)"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- crates/uor-r4-cpu-train
test ! -e "$PIPELINE"
test ! -e "$BUILD"
mkdir -p "$PIPELINE"
BUILD_STARTED=$(date -u +%s)
/usr/bin/time -v -o "$PIPELINE/build.time.txt" env CARGO_BUILD_JOBS=16 \
  CARGO_TARGET_DIR="$BUILD" cargo build --locked --release \
    -p uor-r4-cpu-train --bin cpu-content-readout-driver
BUILD_SECONDS=$(( $(date -u +%s) - BUILD_STARTED ))
BUILD_BYTES=$(du -sb "$BUILD" | cut -f1)
BUILD_RSS=$(awk -F': ' '/Maximum resident set size/ { print $2 }' \
  "$PIPELINE/build.time.txt")
test "$BUILD_BYTES" -le 2147483648
test "$BUILD_RSS" -le 4194304
printf '{"jobs":16,"wall_seconds":%s,"peak_rss_kib":%s,"target_bytes":%s}\n' \
  "$BUILD_SECONDS" "$BUILD_RSS" "$BUILD_BYTES" >"$PIPELINE/build-summary.json"
BIN=$BUILD/release/cpu-content-readout-driver
sha256sum "$BIN" | sed 's# .*/#  #' >"$PIPELINE/binary.sha256"
run() {
  seed=$1 label=$2 artifact=$3
  out=$PIPELINE/$label
  /usr/bin/time -v -o "$PIPELINE/$label.time.txt" timeout 600s \
    env RAYON_NUM_THREADS=32 "$BIN" \
    "$DUAL/dual-c131-seed-${seed}-raw_both.uorlut" \
    "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" "$DEV/dev-manifest.json" \
    "$out" --expected-artifact-sha256 "$artifact" \
    --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
    --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
    --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
    --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
    --reply-cap 16 --stop-prototypes 8 --seed 1 >"$PIPELINE/$label.stdout.json"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 1048576
  jq -e '.bar.code.structural_pass' \
    "$out/report.json" >/dev/null
  test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
  test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
}
run 1 seed-1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run 2 seed-2 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
run 1 seed-1-repeat a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
jq 'del(.wall_elapsed_ms, .peak_rss_kib)' \
  "$PIPELINE/seed-1/report.json" >"$PIPELINE/seed-1.normalized.json"
jq 'del(.wall_elapsed_ms, .peak_rss_kib)' \
  "$PIPELINE/seed-1-repeat/report.json" >"$PIPELINE/seed-1-repeat.normalized.json"
cmp "$PIPELINE/seed-1.normalized.json" "$PIPELINE/seed-1-repeat.normalized.json"
while IFS= read -r -d '' file; do
  cmp "$file" "$PIPELINE/seed-1-repeat/${file#"$PIPELINE/seed-1/"}"
done < <(find "$PIPELINE/seed-1" -maxdepth 1 -type f -name '*.json' \
  ! -name report.json -print0 | sort -z)
(cd "$PIPELINE" && find . -type f ! -name SHA256SUMS -print0 |
  sort -z | xargs -0 sha256sum >SHA256SUMS &&
  sha256sum -c SHA256SUMS --quiet)
test "$(du -sb "$PIPELINE" | cut -f1)" -le 167772160
```

Any source, input, binary, gate count, token identity, worker-order, repeat
or resource mismatch is an instrument fault requiring a new FIT.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 generated-state content stop gate:
docs/harness/prep/cpu-training-pilot-content-gate-2026-11-04.md
head: 8692fafb89995521354a93d5f53b2805cb12ff84
implementation pin: 249b296072c2d716019af487dbb0afaa38281e55
reviewer: independent-subagent content_gate_refit_3
date: 2026-11-04

A FIT verdict clears compute for this configuration only (D22).
