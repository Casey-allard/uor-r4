# Copy-key depth-age router

**Status:** `TEST FITNESS: FIT`

## Question

Can a discrete router learned from exact copy depth and candidate age choose
better copy keys than the fixed longest-first rule selected by deepseek-149?

This is D84 CPU development research on the Ryzen 9 9950X. It changes no float
training, integer serving, CUDA or Claude-owned source.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `ab2836f13` |
| panel-generator source | `4e42ffca5` |
| tokenizer | `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377` |
| public M1 DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| public M1 DEV mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| public M1 DEV manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Data split

- Public M1 DEV is instrument parity only.
- Fit on new `cell=train,train` seed `2601`, 64 conversations.
- Evaluate only on new `cell=dev,dev` seed `2602`, 64 conversations.
- Each decision panel must contain at least 200 eligible target positions.

The earlier opened panels at seeds 2501/2502 were used only for implementation
debugging: the router matched longest at 813/815 on train and 826/826 on dev.
Those readings are **NOT A DECISION RESULT**.

## Fixed construction

Use the deepseek-149 192-token BOS/window scan. Record each candidate's suffix
depth `1..=4` and age `t - s`. Learn an empirical `right / matches` table over
the fixed age uppers:

```text
1, 2, 4, 8, 16, 32, 64, 128, 192
```

Fit the precision table on copyable target positions only, matching the bar's
evaluation population. At inference, select the available depth with highest
learned precision; tie by deeper key. A depth-age cell with no training matches
is unavailable. If no cell is available, fall back to longest-first. This is a
lookup table with no softmax, normalization or matrix operation.

## Instrument gate

Before reading the router:

- public DEV must reproduce 8,243 copyable positions, depth-1 6,306 / 3,859,
  depth-2 4,383 / 3,506 and `k2>k1` 4,388;
- the router has 36 `(age, depth)` cells and `right <= matches` everywhere;
- both decision panels have at least 200 eligible positions;
- the random control changes at least one copyable longest-first decision; and
- a fresh process reproduces the router report byte for byte.

## Bar

On the new dev panel, the router must beat longest-first, `k2>k1`, and random
depth by the paired two-sigma rule. Each comparison requires greater candidate
sum and at least `max(2, ceil(2*sqrt(control-only)))` candidate-only
disagreements.

## Stop rules

Stop on public parity drift, input/hash drift, malformed router tables,
non-determinism, inadequate rows, a vacuous random control, resource breach, or
any need to change serving or Claude-owned source.

## Minimum test power

The evaluation panel is a new 64-conversation m-world-v2 draw with at least 200
target positions. The router is fitted only on a separate new panel, and every
comparison is paired by target position. This guards against evaluating the
fitted table on its own rows.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT, GO and review | 700 s |
| release and generator builds | 300 s |
| two panel draws and router runs | 120 s |
| delivery and evidence | 300 s |
| total increment | **1,420 s** |

- Build jobs: 16; build RSS cap 4 GiB.
- Build wall-time cap: 600 s per build; fresh target storage cap: 16 GiB.
- Screen runs: 600 s and 1 GiB RSS each.
- Evidence cap: 33,554,432 bytes.
- Post-build and post-run free-space floor: **30 GiB + 128 MiB**
  (`32,346,472,448` bytes) on both source and shared volumes.
- Previous admitted cumulative: `70,927.677487699 s`.
- New admitted cumulative: **72,347.677487699 s**.

The model-time ledger remains `ABSENT`.

## Steps

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-key-router
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-key-router/2026-11-05
PANELS=$OUT/panels
BUILD=$SOURCE/target/deepseek/copy-key-router-run
GENERATOR_TARGET=$SOURCE/target/deepseek/copy-key-router-generator
DEV=$SHARED/local/deepseek/public-m1-parity
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
IMPL_PIN=ab2836f13
FREE_FLOOR=32346472448
TARGET_CAP=17179869184
EVIDENCE_CAP=33554432

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
check_storage() {
  phase=$1
  source_free=$(df -PB1 "$SOURCE" | awk 'NR==2 {print $4}')
  shared_free=$(df -PB1 "$SHARED" | awk 'NR==2 {print $4}')
  printf '{"phase":"%s","source_free_bytes":%s,"shared_free_bytes":%s,"free_floor_bytes":%s}\n' \
    "$phase" "$source_free" "$shared_free" "$FREE_FLOOR" >"$OUT/storage-$phase.json"
  test "$source_free" -ge "$FREE_FLOOR"
  test "$shared_free" -ge "$FREE_FLOOR"
}
seal() {
  (cd "$OUT"; find . -type f ! -name SHA256SUMS ! -name SHA256SUMS.tmp -print0 |
    sort -z | xargs -0 -r sha256sum >SHA256SUMS.tmp
    mv SHA256SUMS.tmp SHA256SUMS
    sha256sum -c SHA256SUMS --quiet
    test "$(du -sb . | cut -f1)" -le "$EVIDENCE_CAP")
}
trap 'status=$?; trap - EXIT; seal || status=$?; exit "$status"' EXIT

check_storage before-build
timeout 600s /usr/bin/time -v -o "$OUT/build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" cargo build --locked --release \
  -p uor-r4-cpu-train --bin cpu-copy-key-depth --bin cpu-copy-key-router
timeout 600s /usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GENERATOR_TARGET" cargo build --locked \
  --manifest-path "$SOURCE/docs/labs/one-model-end-to-end/instrument/Cargo.toml" \
  --features panel-writer --bin heldout-panel
check_storage after-build
target_bytes=$(du -sb "$BUILD" "$GENERATOR_TARGET" |
  awk '{total += $1} END {print total}')
printf '{"target_bytes":%s,"target_cap_bytes":%s}\n' \
  "$target_bytes" "$TARGET_CAP" >"$OUT/target-storage-after-build.json"
test "$target_bytes" -le "$TARGET_CAP"

GEN=$GENERATOR_TARGET/debug/heldout-panel
/usr/bin/time -v -o "$OUT/train-panel-gen.time.txt" \
  timeout 600s "$GEN" out="$PANELS/train-train-2601" tokenizer="$TOKENIZER" \
  cell=train,train conversations=64 seed=2601 protocol=2 \
  >"$OUT/train-panel-gen.stdout.json"
/usr/bin/time -v -o "$OUT/dev-panel-gen.time.txt" \
  timeout 600s "$GEN" out="$PANELS/dev-dev-2602" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=64 seed=2602 protocol=2 \
  >"$OUT/dev-panel-gen.stdout.json"
jq -e '.split == "train" and .cell.phrasing == "train" and
  .cell.value == "train" and .seed == 2601 and .conversations == 64 and
  .dialogue_protocol_version == 2' \
  "$PANELS/train-train-2601/manifest.json" >/dev/null
jq -e '.split == "dev" and .cell.phrasing == "dev" and
  .cell.value == "dev" and .seed == 2602 and .conversations == 64 and
  .dialogue_protocol_version == 2' \
  "$PANELS/dev-dev-2602/manifest.json" >/dev/null

DEPTH=$BUILD/release/cpu-copy-key-depth
ROUTER=$BUILD/release/cpu-copy-key-router
/usr/bin/time -v -o "$OUT/dev-parity.time.txt" \
  timeout 600s "$DEPTH" "$DEV/dev-tokens.u16" "$DEV/dev-response-mask.u8" \
  "$DEV/dev-manifest.json" --context 192 --seed 149 >"$OUT/dev-parity.json"
/usr/bin/time -v -o "$OUT/router.time.txt" \
  timeout 600s "$ROUTER" \
  "$PANELS/train-train-2601/tokens.u16" \
  "$PANELS/train-train-2601/response_mask.u8" \
  "$PANELS/train-train-2601/manifest.json" \
  "$PANELS/dev-dev-2602/tokens.u16" \
  "$PANELS/dev-dev-2602/response_mask.u8" \
  "$PANELS/dev-dev-2602/manifest.json" \
  --context 192 --seed 151 >"$OUT/router.json"
/usr/bin/time -v -o "$OUT/router-repeat.time.txt" \
  timeout 600s "$ROUTER" \
  "$PANELS/train-train-2601/tokens.u16" \
  "$PANELS/train-train-2601/response_mask.u8" \
  "$PANELS/train-train-2601/manifest.json" \
  "$PANELS/dev-dev-2602/tokens.u16" \
  "$PANELS/dev-dev-2602/response_mask.u8" \
  "$PANELS/dev-dev-2602/manifest.json" \
  --context 192 --seed 151 >"$OUT/router-repeat.json"

test "$(jq -r '.reading.copyable_positions' "$OUT/dev-parity.json")" -eq 8243
test "$(jq -r '.reading.fixed[0].positions_with_match' "$OUT/dev-parity.json")" -eq 6306
test "$(jq -r '.reading.fixed[0].right' "$OUT/dev-parity.json")" -eq 3859
test "$(jq -r '.reading.fixed[1].positions_with_match' "$OUT/dev-parity.json")" -eq 4383
test "$(jq -r '.reading.fixed[1].right' "$OUT/dev-parity.json")" -eq 3506
test "$(jq -r '.reading.k2_backoff_k1.right' "$OUT/dev-parity.json")" -eq 4388
test "$(jq -r '.router.stats | length' "$OUT/router.json")" -eq 36
jq -e '.router.stats | all(.right <= .matches)' "$OUT/router.json" >/dev/null
test "$(jq -r '.training.eligible_positions' "$OUT/router.json")" -ge 200
test "$(jq -r '.test.eligible_positions' "$OUT/router.json")" -ge 200
test "$(jq -r '.test.random_changed_from_longest' "$OUT/router.json")" -gt 0
cmp "$OUT/router.json" "$OUT/router-repeat.json"
check_storage after-run
test "$(awk -F': ' '/Maximum resident set size/ {print $2}' "$OUT/build.time.txt")" -le 4194304
test "$(awk -F': ' '/Maximum resident set size/ {print $2}' "$OUT/generator-build.time.txt")" -le 4194304
for t in "$OUT"/*.time.txt; do
  [ "$t" = "$OUT/build.time.txt" ] || [ "$t" = "$OUT/generator-build.time.txt" ] ||
    test "$(awk -F': ' '/Maximum resident set size/ {print $2}' "$t")" -le 1048576
done
test "$(du -sb "$OUT" | cut -f1)" -le "$EVIDENCE_CAP"
```

Read `.test.router_vs_longest.passes`, `.test.router_vs_k2_backoff.passes` and
`.test.router_vs_random.passes` from `router.json`. All three must be true for
a candidate pass.

## Rollback boundary

The unit adds a CPU router to the isolated research module. Reverting it
removes only the surrogate; no model artifact, serving format, CUDA code or
Claude-owned source changes.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 copy-key depth-age router:
docs/harness/prep/cpu-training-pilot-copy-key-router-2026-11-05.md
head: 857485ffe7d3748db090d9f9e85032781ecf524a
implementation pin: ab2836f13f2d73e02bc4c395511f81f879fd03ff
reviewer: codex-fresh-context-fit-round-2 (session
  01a1a98d-5ff8-78c1-9a0a-f847925a5f86)
note: Round 1 found the candidate-age novelty-count blocker, missing target
  storage/free-space gates, and a large-count threshold edge. All were repaired.
  Round 2 found no blocker or major. Its provenance and storage-audit minors are
  fixed above without moving the bar. The remaining notes state that fitting is
  copyable-conditioned and that the unfixed >192 age alias is unreachable at the
  registered 192-token context.
date: 2026-11-04

A FIT verdict clears compute for this configuration only (D22).

## Pre-run GO

GO - reviewer `codex-fresh-context-go` (session
01a1a992-ee86-7571-9a73-2977b32e9044), 2026-11-04. HEAD was clean at
`edcbfab83aad5fe1f5ad0d1d3c5357b8280b8139`; implementation `ab2836f13` and
generator `4e42ffca5` matched. The four public inputs verified, the decision root
was absent, all registered provenance/power/control/resource gates were present,
and the host had sufficient CPU, RAM and storage. Its two non-gating findings
(the stale top status and the evidence cap's placement before final sealing) are
fixed here without moving the bar.

## Outcome

Decision: **REJECT**. On seed 2602 the router was right on **913 / 1,655**
copyable positions versus **921** for longest-first. It beat `k2>k1` at
**913 / 912** and random depth at **913 / 876**, but all three comparisons were
required. See
[REPORT.md](../../loop/tasks/cpu-training-pilot/deepseek-151/REPORT.md) and
[the sealed evidence](../../labs/cpu-training-pilot-copy-key-depth-2026-11-05/copy-key-router/evidence/PUBLISHED-SHA256SUMS).
