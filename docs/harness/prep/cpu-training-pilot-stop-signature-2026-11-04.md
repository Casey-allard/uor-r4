# Full-query Hamming stop signatures

**Status:** FIT

## Question

Can a two-class, multi-prototype Hamming stop candidate over the complete
256-coordinate writer query recognize held-out reply endings outside writer
admission, while keeping nonterminal false stops at or below 1% and beating a
class-preserving shuffled-label gate on both public model seeds?

This is D84 CPU development research on the Ryzen 9 9950X. It does not change
integer serving and is not GPU qualification, complete-reply correctness or
useful conversation.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `6bd0edff0e7a32e8457e2630a952bd235a9c0523` |
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Artifacts:

- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-{1,2}-raw_both.uorlut`
- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-{tokens.u16,response-mask.u8,manifest.json}`

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

The public manifest says `split=train`. This is a repeated-key,
document-disjoint screen: selected key families occur on both sides. It is not
unseen-key, fresh-phrasing or language evidence.

## Data split

The model-free selection uses 16 eight-token keys, even documents for training,
odd documents for evaluation and up to eight examples per key:

| Partition | Samples | Relevant decisions | Terminal | Nonterminal |
|---|---:|---:|---:|---:|
| training | 98 | 1,218 | 98 | 1,120 |
| evaluation | 103 | 1,188 | 103 | 1,085 |

Relevant rows are the prompt-final query and reply-content queries. The final
reply-content row before corpus EOS is terminal. Opening/reply memory is derived
from training examples only.

## Mechanism

Pack each exact Q12 writer query into four `u64` words in coordinate order; a
nonnegative coordinate is bit 1.

For each class:

1. deduplicate observed signatures;
2. choose the Hamming medoid;
3. choose up to seven farthest-first prototypes; and
4. break distance and signature ties lexicographically.

A query fires when
`d_nonterminal - d_terminal` is at least the fitted integer margin. Fit the
margin on training rows only: among thresholds with at most `floor(n/100)`
nonterminal stops, maximize terminal stops; ties choose the larger margin,
then the smaller threshold. Refuse no class, no prototype, no changed shuffled
subset or no qualifying margin.

The CPU evaluator may emit BOS before the writer fallback. Ungated decisions
must remain exactly the writer token. This does not put the candidate into
writer admission or integer serving.

## Arms

1. no gate;
2. true-label Hamming prototypes and fitted margin;
3. class-preserving shuffled-label prototypes and refitted margin; and
4. free greedy generation at a 16-token cap for arms 1-3.

## Bar

Both seeds must pass:

- at least **26/103** terminal rows stop correctly;
- nonterminal false stops are at most **10/1,085** and the true gate fires at
  least once;
- true terminal stops beat no-gate writer and shuffled-gate terminal stops
  by `max(2, ceil(2 * sqrt(b + c)))`; and
- every ungated decision is exactly its recorded writer token.

Prompt-final rows are nonterminal. Free generation reports BOS, premature BOS,
cap, context, exact reference, opening, distinct replies and repeated 4-grams.

## Minimum test power

There are 103 terminal rows per seed and a 26-event threshold. Both independent
seeds must pass, and the matched shuffled gate must trail by the paired margin.
The 1,085-row nonterminal panel limits measured false stops to a 10-event
ceiling. This is a bounded configuration screen, not precision qualification.

## Instrument gates

- Input, source and binary hashes match; both splits contain both classes.
- Training reports true/false stops, fitted margin and prototype counts.
- Shuffled labels preserve counts and change the terminal subset.
- Every gated decision is BOS and every ungated decision is the writer token.
- Seed-1 repeat reproduces every non-timing and non-resource report field.
- A positive is only a CPU candidate. Integer-session integration requires a
  later coordinated unit with exact stop/token parity.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT and pre-run review | 600 s |
| release build | 300 s |
| two seeds plus repeat | 600 s |
| delivery and evidence | 300 s |
| total increment | **1,800 s** |

- Build: 16 jobs, 4 GiB RSS maximum, 2 GiB target maximum.
- Runs: 32 Rayon workers, 1 GiB RSS maximum, 300 s each.
- Retained pipeline: 64 MiB maximum.
- Post-run free-space floor: 30 GiB plus 128 MiB.
- Previous ledger: `cpu-training-pilot-bos-gain-2026-11-04/resource-ledger.json` at `34833.112387175 s`.
- Admitted cumulative projection: **36,633.112387175 s**.

## Steps

Build in a fresh source-bound target and run seeds 1, 2 and a seed-1 repeat in
fresh roots:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-stop-signature
SHARED=/home/callard/projects/codex-playground/uor-r4
SOURCE_PIN=6bd0edff0e7a32e8457e2630a952bd235a9c0523
PIPELINE=$SHARED/local/deepseek/stop-signature/2026-11-04-attempt-1
BUILD=$SOURCE/target/deepseek/stop-signature-132
DEV=$SHARED/local/deepseek/public-m1-parity
DUAL=$SHARED/local/deepseek/public-fbs-dual
HARNESS=$SHARED/local/harness-target/release/uor-harness
MIN_FREE_KIB=37748736
MIN_RESERVE_KIB=31588352
MAX_BUILD_BYTES=2147483648
MAX_BUILD_RSS_KIB=4194304
MAX_PIPELINE_BYTES=67108864
GUARD=(crates third_party Cargo.toml Cargo.lock .cargo rust-toolchain.toml)
free_kib() {
  df -Pk "$1" | awk 'NR == 2 { print $4 }'
}
test "$(free_kib "$SOURCE")" -ge "$MIN_FREE_KIB"
test "$(free_kib "$SHARED")" -ge "$MIN_FREE_KIB"
"$HARNESS" budget show --json | jq -e \
  '.model_time_ledger.present == false
   and .resource_ledger.chain.head == "cpu-training-pilot-stop-signature-2026-11-04/resource-ledger.json"
   and .resource_ledger.chain.cumulative_projected_seconds == 36633.112387175' >/dev/null
test "$(env -u RUSTC -u RUSTUP_TOOLCHAIN rustc --version)" = \
  "rustc 1.97.1 (8bab26f4f 2026-07-14)"
test "$(env -u RUSTC -u RUSTUP_TOOLCHAIN cargo --version)" = \
  "cargo 1.97.1 (c980f4866 2026-06-30)"
test "$(lscpu | awk -F: '/Model name/ { sub(/^[ \t]+/, "", $2); print $2; exit }')" = \
  "AMD Ryzen 9 9950X 16-Core Processor"
test -z "$(git -C "$SOURCE" status --porcelain -- "${GUARD[@]}")"
git -C "$SOURCE" diff --quiet "$SOURCE_PIN" -- "${GUARD[@]}"
test ! -e "$PIPELINE"
test ! -e "$BUILD"
mkdir -p "$PIPELINE/build"
BUILD_STARTED=$(date -u +%s)
(
  cd "$SOURCE"
  /usr/bin/time -v -o "$PIPELINE/build/build.time.txt" \
    env -u RUSTC -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER \
      -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u RUSTDOCFLAGS \
      -u RUSTUP_TOOLCHAIN CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" \
      cargo build --locked --release \
        -p uor-r4-cpu-train --bin cpu-stop-signature-driver
)
BUILD_SECONDS=$(( $(date -u +%s) - BUILD_STARTED ))
BUILD_BYTES=$(du -sb "$BUILD" | cut -f1)
BUILD_RSS=$(awk -F': ' '/Maximum resident set size/ { print $2 }' \
  "$PIPELINE/build/build.time.txt")
test "$BUILD_BYTES" -le "$MAX_BUILD_BYTES"
test "$BUILD_RSS" -le "$MAX_BUILD_RSS_KIB"
printf '{"jobs":16,"wall_seconds":%s,"peak_rss_kib":%s,"target_bytes":%s}\n' \
  "$BUILD_SECONDS" "$BUILD_RSS" "$BUILD_BYTES" >"$PIPELINE/build/summary.json"
BIN=$BUILD/release/cpu-stop-signature-driver
BIN_SHA=$(sha256sum "$BIN" | cut -d' ' -f1)
printf '%s  cpu-stop-signature-driver\n' "$BIN_SHA" >"$PIPELINE/binary.sha256"
run() {
  seed=$1 artifact_pin=$2 out=$3
  test ! -e "$out"
  /usr/bin/time -v timeout 300s env RAYON_NUM_THREADS=32 \
    "$BIN" "$DUAL/dual-c131-seed-${seed}-raw_both.uorlut" \
    "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" \
    "$DEV/dev-manifest.json" "$out" \
    --expected-artifact-sha256 "$artifact_pin" \
    --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
    --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
    --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
    --keys 16 --per-key 8 --key-tokens 8 --reply-turn 8 \
    --prototypes 8 --reply-cap 16 --seed 1 >/dev/null
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 1048576
  test "$(jq -er '.bar.ungated_token_mismatches' "$out/report.json")" -eq 0
  test "$(jq -er '.bar.gate_exposed' "$out/report.json")" = true
  test "$(jq -er '.training.true.terminal_stops' "$out/report.json")" -gt 0
  test "$(sha256sum "$BIN" | cut -d' ' -f1)" = "$BIN_SHA"
  test "$(free_kib "$SOURCE")" -ge "$MIN_RESERVE_KIB"
  test "$(free_kib "$SHARED")" -ge "$MIN_RESERVE_KIB"
  (cd "$out" && find . -type f ! -name SHA256SUMS -print0 |
    sort -z | xargs -0 sha256sum >SHA256SUMS &&
    sha256sum -c SHA256SUMS --quiet)
}
run 1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$PIPELINE/seed-1"
run 2 da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 "$PIPELINE/seed-2"
run 1 a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd "$PIPELINE/seed-1-repeat"
NORMALIZE='walk(if type == "object" then
  del(.wall_elapsed_ms, .peak_rss_kib, .wall_elapsed_us,
      .generated_tokens_per_second)
else . end)'
diff <(jq -S "$NORMALIZE" "$PIPELINE/seed-1/report.json") \
  <(jq -S "$NORMALIZE" "$PIPELINE/seed-1-repeat/report.json")
test -z "$(git -C "$SOURCE" status --porcelain -- "${GUARD[@]}")"
test "$(du -sb "$PIPELINE" | cut -f1)" -le "$MAX_PIPELINE_BYTES"
```

Any source, input, binary, split, control, gate or deterministic mismatch is
an instrument fault requiring a new FIT before retry.

## Stop rules

Stop on source/input drift, no class/prototype/margin, unchanged shuffled
labels, gate never exposed, ungated writer-token mismatch, nondeterministic
repeat, resource/storage breach or any need to change serving. A
configuration-negative bar is retained without retuning prototypes or margin.

## Claim boundary

A positive is CPU candidate evidence for this fixed K=8 sign-signature gate.
It is necessary but not sufficient for chat: content correctness remains open,
and integer integration needs a separate coordinated parity unit. A negative
closes only this fixed signature/prototype/margin configuration.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 Hamming stop signatures outside writer admission
sheet: docs/harness/prep/cpu-training-pilot-stop-signature-2026-11-04.md
head: c2fc34f19a61d9aab5a5853199965b27a1bcd523
implementation pin: 6bd0edff0e7a32e8457e2630a952bd235a9c0523
reviewer: independent-subagent stop-signature-fit-2
note: two-seed D84 screen; full-query 256-bit sign signatures, K=8 observed
prototypes, training-only margin under exact 1% false-stop ceiling, no-gate and
class-preserving shuffled controls, explicit ungated token identity, gate
hashes, free-reply reporting, source/resource/storage gates and deterministic
repeat
date: 2026-11-04

A FIT verdict clears compute for this configuration only (D22).
