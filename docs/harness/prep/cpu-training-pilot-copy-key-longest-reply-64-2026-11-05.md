# Longest-first copy selection in complete replies, attempt 2

**Status:** `TEST FITNESS: FIT`

## Question

Does deepseek-149's exact-copy advantage survive when longest-first selection
controls every writer decision in a complete generated reply? The 32-token
attempt stopped as an input/config instrument fault before writing a result:
the freshly generated panels contained assistant responses of 46 and 49
tokens. This attempt keeps the selectors, artifacts and bar unchanged and
raises only the cap to 64 tokens.

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

At each generated decision, use the last 192 tokens of the prompt and every
generated token. For a query ending at the current token, find the latest
exact suffix key at depths one through four:

```text
query_d(t) = x[t], x[t-1], ..., x[t-d+1]
key_d(s)   = x[s-1], x[s-2], ..., x[s-d]
```

The query and candidate are confined to the same 192-token slice, `0 < s < t`,
and missing history at slice start repeats its first token. The latest match at
a depth wins.

The five arms are:

- `longest_4_to_1`: deepest available exact match, else writer;
- `k2_backoff_k1`: depth two, else one, else writer;
- `fixed_depth_1`: depth one, else writer;
- `random_available_depth`: seeded uniform depth over available matches, else
  writer;
- `writer`: no override.

The selector replaces only the next writer token. That token is fed to the same
exact CPU forward path; memory, point-key, stopping and artifact behavior are
unchanged. Generation uses the maximum supported 64-token cap.

This is the same construction as
[attempt 1](cpu-training-pilot-copy-key-longest-reply-2026-11-05.md), with only
the cap and panel seeds changed.

## Data split

Two new m-world-v2 panels are generated after FIT and GO:

- `copy-key-reply-tt-2711`: `cell=train,train`, 128 conversations, seed `2711`;
- `copy-key-reply-dd-2712`: `cell=dev,dev`, 128 conversations, seed `2712`.

Both artifact seeds run on both panels. Every recovered prompt/reply example
is evaluated; no parameter is fit. Each artifact seed and panel must provide at
least 200 reference-reachable examples. Reachable means the reply is shorter
than 64 tokens and prompt plus reply is shorter than artifact context so the
following BOS can be requested. Unreachable examples stay in the census and
are excluded from paired comparisons.

## Instrument gate

Before a result is read:

- both source pins and all input hashes match;
- every reference fits the 64-token cap and each prompt fits artifact context;
- at least 200 examples per artifact/panel cell remain reference-reachable;
- all decisions have writer/selected tokens and copy-depth/fallback state;
- random depth chooses only available exact depths and changes at least one
  reachable longest-first decision per artifact/panel cell;
- a fresh-process repeat matches all readings, reply hashes and decision hashes
  after path/timing fields are excluded;
- finite tests cover window boundaries, latest-match order, depth precedence,
  writer fallback, BOS headroom, cap/context stopping and unreachable-row
  exclusion.

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

Every other outcome is negative for this complete-reply writer override. It
must not be generalized to exact copy, deeper keys or the geometric path.

## R12 authority

1. **What is measured:** paired prompt/reply examples compare exact
   BOS-terminated replies, non-premature BOS completions and correct body
   positions across five fixed selectors.
2. **Whether it is computable:** yes. The public artifacts, CPU forward path,
   pinned generator and deterministic panels are present.
3. **Whether the instrument can beat the bar:** yes in principle. The
   teacher-forced screen changed 8-42 discordant decisions per panel, and this
   unit applies those choices to generated trajectories.
4. **Whether it should run now:** yes. Success preserves depth three/four in
   Claude's next trained copy read; termination-limited success redirects work
   to the existing termination blocker; failure closes this override route.

## Minimum test power

At least 200 reachable examples per artifact seed and panel, plus two trained
artifact seeds. Paired units are reachable prompt/reply examples. Random depth
must be non-vacuous in every artifact/panel cell. A fresh process must reproduce
the reading, and the threshold is based on observed paired discordance.

## Stop rules

Stop on hash or pin drift; fewer than 200 reachable examples in any
artifact/panel cell; nondeterminism; a vacuous random control; a prompt outside
artifact context or reference longer than 64 tokens; malformed trace; resource
breach; or any need to modify serving, integer, CUDA, float-training or
Claude-owned source. Within-cap examples without BOS headroom remain in the
census and are excluded from pairs.

## Cost

| Component | Projection |
|---|---:|
| preparation, FIT, GO and exact-head review | 900 s |
| release build and finite controls | 300 s |
| generator build and two panel draws | 120 s |
| four artifact/panel runs and one repeat | 900 s |
| evidence and delivery | 360 s |
| total increment | **2,580 s** |

- Build jobs: 16; build RSS cap: 4 GiB.
- Each reply run: 32 Rayon workers, 900 s wall cap and 1 GiB RSS cap.
- Retained evidence cap: 64 MiB; fresh target cap: 16 GiB.
- Previous admitted cumulative projection: `74,747.677487699 s` including the
  failed 2,400 s attempt-1 allowance.
- New admitted cumulative projection: **`77,327.677487699 s`**.

The named model-time ledger at
`.uor-models/native-joint-learning-2026-09-04/model-time.json` is `ABSENT` and
is not treated as zero.

Full per-decision reports are compared in streams and stored gzip-compressed,
with no normalized report duplicates.

## Steps

Use a fresh output root after GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-key-longest-reply
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2
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
  timeout 900s "$GEN" out="$PANELS/tt-2711" tokenizer="$TOKENIZER" \
  cell=train,train conversations=128 seed=2711 protocol=2 \
  >"$OUT/tt-panel.stdout.json"
/usr/bin/time -v -o "$OUT/dd-panel.time.txt" \
  timeout 900s "$GEN" out="$PANELS/dd-2712" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2712 protocol=2 \
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
    "$artifact" --cap 64 --seed 153 >"$OUT/$name.json"
  jq -e '.random_nonvacuity.non_vacuous == true' "$OUT/$name.json" >/dev/null
  jq -e '.examples.reachable >= 200' "$OUT/$name.json" >/dev/null
  jq -e --arg expected "$expected" \
    '.inputs.artifact_sha256 == $expected' "$OUT/$name.json" >/dev/null
}

run seed1-tt "$PANELS/tt-2711" "$SEED1" "$SEED1_SHA"
run seed1-dd "$PANELS/dd-2712" "$SEED1" "$SEED1_SHA"
run seed2-tt "$PANELS/tt-2711" "$SEED2" "$SEED2_SHA"
run seed2-dd "$PANELS/dd-2712" "$SEED2" "$SEED2_SHA"
run seed1-dd-repeat "$PANELS/dd-2712" "$SEED1" "$SEED1_SHA"

normalize() {
  jq -S 'walk(if type == "object"
    then del(.wall_elapsed_us, .generated_tokens_per_second)
    else . end)' "$1"
}
cmp <(normalize "$OUT/seed1-dd.json") \
  <(normalize "$OUT/seed1-dd-repeat.json")

test "$(awk -F': ' '/Maximum resident set size/ {print $2}' "$OUT/build.time.txt")" \
  -le 4194304
test "$(awk -F': ' '/Maximum resident set size/ {print $2}' "$OUT/generator-build.time.txt")" \
  -le 4194304
for timing in "$OUT"/*.time.txt; do
  case "$timing" in "$OUT/build.time.txt"|"$OUT/generator-build.time.txt") continue ;; esac
  test "$(awk -F': ' '/Maximum resident set size/ {print $2}' "$timing")" -le 1048576
done
gzip -9 "$OUT"/seed*.json
test "$(du -sb "$OUT" | cut -f1)" -le 67108864
```

Read all four comparison blocks. A pass requires the registered paired rule for
both metrics on both artifact seeds and both panels. Complete-reply and
termination-limited classifications are read only after all reports exist.
The reports retain every decision in gzip-compressed form; the normalized
repeat comparison is a stream and creates no duplicate file.

## Rollback boundary

The unit adds an isolated CPU policy module, reply evaluator and binary.
Reverting it removes only this screen; no artifact, serving format, CUDA source
or Claude-owned file changes.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 deepseek-153
sheet: docs/harness/prep/cpu-training-pilot-copy-key-longest-reply-64-2026-11-05.md
reviewer: codex fresh-context FIT attempt-2
note: Attempt 1 produced no comparison result. The cap-only correction and new
  seeds 2711/2712 match the ORDER, code, fail-closed wrapper, reachable power
  floor, paired bar and D84 isolation.
date: 2026-11-05

A FIT verdict clears compute for this configuration only.

The evidence-resource correction was re-reviewed at HEAD `b67d67ae8b1b`.
Normalized reports are compared as streams and full decision reports are
gzip-compressed before the size gate. This FIT covers the unchanged
experiment plus that evidence-storage correction.

## Pre-run GO

GO - reviewer `codex fresh-context GO attempt-2`, 2026-11-05. HEAD was clean
at `bff2bc7ca5a0`; implementation pin `cdcb63cafe27` and generator pin
`4e42ffca5` matched. Attempt 1 is sealed with no result, seeds `2711`/`2712`
and the attempt-2 root were unopened, the executable wrapper passed `bash -n`,
and the random, reachable-power, artifact-identity and compressed-evidence
controls were verified.
