# Longest-first copy selection in complete replies

**Status:** `TEST FITNESS: FIT`

> **Superseded by
> [attempt 2](cpu-training-pilot-copy-key-longest-reply-64-2026-11-05.md).**
> Attempt 1 stopped as an input/config instrument fault before any comparison
> report: its fresh panels contained 46- and 49-token replies under the
> registered 32-token cap. Its bar and FIT remain preserved as the historical
> record.

## Question

Does deepseek-149's small exact-copy advantage survive when longest-first
selection controls every writer decision in a complete generated reply, rather
than only isolated teacher-forced targets? This decides whether Claude's next
trained copy read should preserve depth-three/four candidates now, or whether
the next dependency is reply termination rather than more copy depth.

This is D84 CPU development research on the Ryzen 9 9950X. It changes no float
training, integer serving, CUDA or Claude-owned source.

## Compute class

Local CPU development research in `uor-r4-cpu-train`; no GPU, pod, paid API,
upload or external model execution. The CPU result cannot substitute for a GPU
qualification bar.

## Base artifacts

| Input | Identity |
|---|---|
| implementation source | `cdcb63cafe27` |
| panel-generator source | `4e42ffca5` (`docs/labs/one-model-end-to-end/instrument`) |
| tokenizer | `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377` |
| public FBS seed 1 | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| public FBS seed 2 | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |

Hardware: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.

## Fixed construction

At each generated decision, form the last 192 tokens of the prompt and every
generated token. An exact suffix candidate at depth `1..=4` uses:

```text
query_d(t) = x[t], x[t-1], ..., x[t-d+1]
key_d(s)   = x[s-1], x[s-2], ..., x[s-d]
```

For `0 < s < t`, the latest matching `s` predicts `x[s]`. Missing history at the
start repeats `x[0]`, as in deepseek-149. The selector may replace only the
token already chosen by the artifact writer. The selected token is fed back to
the same exact CPU forward path; the writer, memory, point key, stopping
contract and artifact remain unchanged.

The five arms are:

- `longest_4_to_1`: deepest available exact match, else the writer token;
- `k2_backoff_k1`: depth two, else one, else the writer token;
- `fixed_depth_1`: depth one, else the writer token;
- `random_available_depth`: seeded uniform choice over available depths, else
  the writer token;
- `writer`: no override.

Generation uses a 32-token cap.

## Data split

Two new m-world-v2 panels are generated after FIT and GO:

- `copy-key-reply-tt-2701`: `cell=train,train`, 128 conversations, seed `2701`;
- `copy-key-reply-dd-2702`: `cell=dev,dev`, 128 conversations, seed `2702`.

Both public artifact seeds run on both panels. Every recovered prompt/reply
example is evaluated; no parameter is fit. Each artifact seed and panel must
provide at least 200 reference-reachable examples, meaning the reply is shorter
than the 32-token cap and prompt plus reply is shorter than artifact context so
the following BOS can still be requested. Paired comparisons use only reachable
examples. The examples are held out from this unit because no panel at either
seed has been opened before this registration.

## Instrument gate

Before a result is read:

- all five pinned input hashes match;
- both public artifacts parse through `CpuFbsForward`;
- every prompt fits the artifact context and every reference fits the 32-token
  cap;
- every decision has a writer token, selected token and either a copy depth or
  a writer fallback;
- a random-depth choice occurs only among available exact depths and changes at
  least one longest-first decision per panel;
- a fresh process reproduces every arm reading, reply hash and decision hash
  after path and timing fields are excluded;
- finite tests pass for latest-match order, depth precedence, fallback, BOS,
  cap and context stopping.

Any failure is an instrument or power fault, not a mechanism negative.

## Bar

Complete-reply transfer passes only if, for both artifact seeds on both panels,
`longest_4_to_1` beats each of `k2_backoff_k1`, `fixed_depth_1`,
`random_available_depth` and `writer` on both exact-reference replies and
non-premature BOS completions by the paired two-sigma rule:

```text
candidate sum > control sum
candidate-only >= max(2, ceil(2 * sqrt(control-only)))
```

Selection-surrogate transfer is positive but termination-limited only if the
complete-reply bar fails while longest-first beats `k2_backoff_k1`,
`fixed_depth_1` and `random_available_depth` on correct reference body positions
by the same rule for both artifact seeds on both panels, with no loss of
non-premature BOS stops against `writer`.

Every other outcome is negative for this fixed complete-reply writer override.
It must not be generalized to exact copy, deeper suffix keys or the geometric
path.

## R12 authority

1. **What is measured:** paired prompt/reply examples compare exact
   BOS-terminated replies, non-premature BOS completions and correct reference
   body positions under five fixed selectors.
2. **Whether it is computable:** yes. The two public artifacts parse on the
   existing exact CPU path, and the panels supply fixed token/mask streams and
   recovered replies. No live model or external service is required.
3. **Whether the instrument can beat the bar:** yes in principle. The
   teacher-forced screen selected different next tokens on 8-42 paired
   discordant decisions per panel, and a selected copy token is fed into the
   same generated trajectory. The complete-reply result is not pre-judged; a
   failure identifies whether fallback, divergence or termination consumes
   that headroom.
4. **Whether it should run now:** yes. It is the next unrun bridge between the
   passed deepseek-149 token decision and Claude's trained-copy work. A
   complete-reply gain selects depth preservation now; a termination-limited
   result redirects the next unit to the existing termination blocker rather
   than adding copy depth without transfer evidence.

## Minimum test power

At least 200 reference-reachable complete replies per artifact seed and panel,
plus two independently trained artifact seeds. Unreachable rows remain in the
input census but are excluded from paired decisions because they cannot finish
with BOS. The paired unit is one reachable prompt/reply example. The random
control must be non-vacuous on each panel. A fresh process must reproduce the
final reading. The two-sigma bar is based on observed paired discordance, not a
single-row difference.

## Stop rules

Stop on artifact, tokenizer or panel hash drift; fewer than 200
reference-reachable examples for any artifact/panel cell; nondeterminism; a
vacuous random control; a prompt outside the artifact context or a reference
longer than the 32-token cap; malformed trace; resource breach; or any need to
modify serving, integer, CUDA, float-training or Claude-owned source. A
reference that fits the cap but leaves no room for the following BOS remains in
the census and is excluded from paired comparisons; it is not an instrument
stop unless the reachable floor then fails.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT, GO and exact-head review | 900 s |
| release build and finite controls | 300 s |
| generator build and two panel draws | 120 s |
| four artifact/panel runs and one repeat | 720 s |
| evidence and delivery | 360 s |
| total increment | **2,400 s** |

- Build jobs: 16; build RSS cap: 4 GiB.
- Each reply run: 32 Rayon workers, 900 s wall cap and 1 GiB RSS cap.
- Retained evidence cap: 64 MiB; fresh target cap: 16 GiB.
- Previous admitted cumulative projection: `72,347.677487699 s`.
- New admitted cumulative projection: **`74,747.677487699 s`**.

The named model-time ledger at
`.uor-models/native-joint-learning-2026-09-04/model-time.json` is `ABSENT` and
is not treated as zero.

## Steps

After FIT and GO, use a fresh output root:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-key-longest-reply
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05
PANELS=$OUT/panels
BUILD=$SOURCE/target/deepseek/copy-key-longest-reply
GENERATOR_TARGET=$SOURCE/target/deepseek/copy-key-panel-generator
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
IMPL_PIN=cdcb63cafe27
GENERATOR_PIN=4e42ffca5
```

The implementation must remain byte-identical to `IMPL_PIN` under `crates/`,
the workspace manifests and `third_party/`. The generator must remain
byte-identical to `GENERATOR_PIN` under
`docs/labs/one-model-end-to-end/instrument/`.

The run wrapper must fail closed:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-key-longest-reply
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05
PANELS=$OUT/panels
BUILD=$SOURCE/target/deepseek/copy-key-longest-reply
GENERATOR_TARGET=$SOURCE/target/deepseek/copy-key-panel-generator
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
IMPL_PIN=cdcb63cafe27
GENERATOR_PIN=4e42ffca5
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
SEED1_SHA=a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
SEED2_SHA=da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
TOKENIZER_SHA=a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377

test ! -e "$OUT"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- \
  .cargo Cargo.toml Cargo.lock rust-toolchain.toml crates third_party
git -C "$SOURCE" diff --quiet "$GENERATOR_PIN" -- \
  docs/labs/one-model-end-to-end/instrument
sha256sum -c <(printf '%s\n' \
  "$SEED1_SHA  $SEED1" \
  "$SEED2_SHA  $SEED2" \
  "$TOKENIZER_SHA  $TOKENIZER") --quiet

mkdir -p "$PANELS"
seal() {
  (cd "$OUT"; find . -type f ! -name SHA256SUMS ! -name SHA256SUMS.tmp -print0 |
    sort -z | xargs -0 -r sha256sum >SHA256SUMS.tmp
    mv SHA256SUMS.tmp SHA256SUMS
    sha256sum -c SHA256SUMS --quiet)
}
trap 'status=$?; trap - EXIT; seal || status=$?; exit "$status"' EXIT

/usr/bin/time -v -o "$OUT/build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$BUILD" \
  cargo build --locked --release -p uor-r4-cpu-train --bin cpu-copy-key-reply
/usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GENERATOR_TARGET" \
  cargo build --locked \
  --manifest-path "$SOURCE/docs/labs/one-model-end-to-end/instrument/Cargo.toml" \
  --features panel-writer --bin heldout-panel
GEN=$GENERATOR_TARGET/debug/heldout-panel

/usr/bin/time -v -o "$OUT/tt-panel.time.txt" \
  timeout 900s "$GEN" out="$PANELS/tt-2701" tokenizer="$TOKENIZER" \
  cell=train,train conversations=128 seed=2701 protocol=2 \
  >"$OUT/tt-panel.stdout.json"
/usr/bin/time -v -o "$OUT/dd-panel.time.txt" \
  timeout 900s "$GEN" out="$PANELS/dd-2702" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2702 protocol=2 \
  >"$OUT/dd-panel.stdout.json"

BIN=$BUILD/release/cpu-copy-key-reply
run() {
  name=$1
  panel=$2
  artifact=$3
  expected=$4
  /usr/bin/time -v -o "$OUT/$name.time.txt" \
    timeout 900s env RAYON_NUM_THREADS=32 "$BIN" \
    "$panel/tokens.u16" "$panel/response_mask.u8" "$panel/manifest.json" \
    "$artifact" --cap 32 --seed 153 >"$OUT/$name.json"
  jq -e '.random_nonvacuity.non_vacuous == true' "$OUT/$name.json" >/dev/null
  jq -e '.examples.reachable >= 200' "$OUT/$name.json" >/dev/null
  jq -e --arg expected "$expected" \
    '.inputs.artifact_sha256 == $expected' "$OUT/$name.json" >/dev/null
}

run seed1-tt "$PANELS/tt-2701" "$SEED1" "$SEED1_SHA"
run seed1-dd "$PANELS/dd-2702" "$SEED1" "$SEED1_SHA"
run seed2-tt "$PANELS/tt-2701" "$SEED2" "$SEED2_SHA"
run seed2-dd "$PANELS/dd-2702" "$SEED2" "$SEED2_SHA"
run seed1-dd-repeat "$PANELS/dd-2702" "$SEED1" "$SEED1_SHA"

for report in "$OUT"/seed*.json; do
  base=$(basename "$report" .json)
  case "$base" in *-repeat) continue ;; esac
  jq -S 'walk(if type == "object"
    then del(.wall_elapsed_us, .generated_tokens_per_second)
    else . end)' "$report" >"$OUT/$base.normalized.json"
done
jq -S 'walk(if type == "object"
  then del(.wall_elapsed_us, .generated_tokens_per_second)
  else . end)' "$OUT/seed1-dd-repeat.json" >"$OUT/seed1-dd-repeat.normalized.json"
cmp "$OUT/seed1-dd.normalized.json" "$OUT/seed1-dd-repeat.normalized.json"

test "$(awk -F': ' '/Maximum resident set size/ {print $2}' "$OUT/build.time.txt")" \
  -le 4194304
test "$(awk -F': ' '/Maximum resident set size/ {print $2}' "$OUT/generator-build.time.txt")" \
  -le 4194304
for timing in "$OUT"/*.time.txt; do
  case "$timing" in "$OUT/build.time.txt"|"$OUT/generator-build.time.txt") continue ;; esac
  test "$(awk -F': ' '/Maximum resident set size/ {print $2}' "$timing")" -le 1048576
done
test "$(du -sb "$OUT" | cut -f1)" -le 67108864
```

Read all four comparison blocks. A pass requires the registered paired rule
for both metrics on both artifact seeds and both panels. The complete-reply
and termination-limited classifications are evaluated only from those
reports; no target is selected after reading them.

## Rollback boundary

The unit adds an isolated CPU policy module, reply evaluator and binary. Reverting
it removes only this screen; no artifact, serving format, CUDA source or
Claude-owned file changes.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 deepseek-153
sheet: docs/harness/prep/cpu-training-pilot-copy-key-longest-reply-2026-11-05.md
reviewer: codex fresh-context FIT round 4
note: The 192-token window, strict BOS headroom, reachable-row floor,
  paired-row exclusion, random non-vacuity and R12 authority terms match the
  code and ORDER at HEAD 2e27e5c12df4 with implementation pin cdcb63cafe27.
date: 2026-11-05

A FIT verdict clears compute for this configuration only. One stated advisory
remains: the run wrapper must fail when `random_nonvacuity.non_vacuous` is not
true.

## Pre-run GO

GO - reviewer `codex fresh-context GO`, 2026-11-05. HEAD was clean at
`3d44126fc763`; implementation pin `cdcb63cafe27` and generator pin
`4e42ffca5` matched. The output root was absent, all input hashes matched, the
executable wrapper passed `bash -n`, and the fail-closed random, reachable-power
and artifact-identity checks were present. The 64 MiB evidence cap is tight but
enforced; an overrun stops cleanly before a result.
