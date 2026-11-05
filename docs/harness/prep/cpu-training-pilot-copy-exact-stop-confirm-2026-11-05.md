# Pre-registration: exact-body stop gate on a fresh dev panel

Run ID: `cpu-training-pilot-copy-exact-stop-confirm-2026-11-05`

## Decision

Does the exact-body-trained geometric stop gate transfer to a newly generated
dev panel that was unopened when the gate and its bar were fixed?

This is D84 CPU development research. It is not GPU qualification and makes no
serving or language claim.

## Prior evidence and change

Deepseek-157 rejected all-reference-length training: both seeds produced zero
exact replies. Deepseek-159 attempt 7 filtered training to the 11 exact train
bodies and fitted 7/11 terminal decisions, but its already-opened dev panel
produced only 1/13 exact replies on each seed. Attempt 7 lacked a recorded
pre-run independent FIT and is exploratory evidence only.

This confirmation changes only the evaluation panel. It does not retune the
labels, prototypes, seed, threshold rule, decode path or controls.

## Base artifact

- seed 1: `dual-c131-seed-1-raw_both.uorlut`, SHA-256
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2: `dual-c131-seed-2-raw_both.uorlut`, SHA-256
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- exact-body stop implementation pin: `466101669649`, source SHA-256
  `f9a9e88dda91f4fc4b85dfb34da354adf548c6dd8121c31fb3d82102f0c46067`;
- copy-policy source at base `330185de6`, SHA-256
  `3c4c4ba0f82999914008653ca83ef9e8dcdb1c048acdb4907c0893cba63e12db`;
- panel-generator tree at base `330185de6`, tree
  `869e65db6d0bb997a5f1094ab0de4efa1ac10c25`;
- tokenizer SHA-256
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`.

## Data split

- Fit: sealed longest-first train report and panel 2711, unchanged:
  - `seed1-tt.json.gz` SHA-256
    `37715a5d621c5b8c2269940505491d13d84c37b414fa2eb9646a2de54000e6ee`;
  - `seed2-tt.json.gz` SHA-256
    `4a6a274822f287a806a52704417c3c2bd75bee604484ce610852cfd40e809372`;
  - tokens `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`,
    mask `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`,
    manifest `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`.
- Fresh eval: generate a new `cell=dev,dev` panel, 128 conversations,
  protocol 2, seed 2722, after FIT and GO. Its root is
  `local/deepseek/copy-exact-stop-confirm/2026-11-05/panels/dd-2722`, which
  is absent now and is not inside any sealed evidence tree.
- This is one confirmation attempt. It may not be rerun on another panel after
  its result without a new pre-registration.
- Generate the copy report for `dd-2722` with `cpu-copy-key-reply`, cap 64 and
  policy seed 153. Both artifact seeds must have at least 200 reachable rows.
- No read of the fresh gate output occurs until both copy reports exist.

## Steps

Use a fresh output root after GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-copy-exact-stop
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/copy-exact-stop-confirm/2026-11-05
PANELS=$OUT/panels
PANEL=$PANELS/dd-2722
TRAIN=$SHARED/local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2
TOKENIZER=$SHARED/local/deepseek/m2-evidence/tokenizer.json
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut
SEED1_SHA=a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
SEED2_SHA=da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
TOKENIZER_SHA=a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377
TT1_SHA=37715a5d621c5b8c2269940505491d13d84c37b414fa2eb9646a2de54000e6ee
TT2_SHA=4a6a274822f287a806a52704417c3c2bd75bee604484ce610852cfd40e809372
TT1_RAW_SHA=28a8caaffef6d815da86a6e84207729f3acd41892ced8a205cfe5248d7c2f62a
TT2_RAW_SHA=466b94e7b4c1ae2d58337c51b07171c6d2229b23edaa366949e7a2ed8a5642e3
TRAIN_TOKENS_SHA=140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91
TRAIN_MASK_SHA=d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad
TRAIN_MANIFEST_SHA=d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5
BUILD=$SOURCE/target/deepseek/copy-exact-stop
GEN_TARGET=$SOURCE/target/deepseek/copy-exact-panel
IMPL_PIN=466101669649
BASE=330185de6

test ! -e "$OUT"
test ! -e "$PANEL"
git -C "$SOURCE" diff --quiet "$IMPL_PIN" -- \
  crates/uor-r4-cpu-train/src/bin/cpu-copy-generated-stop.rs
git -C "$SOURCE" diff --quiet "$BASE" -- \
  crates/uor-r4-cpu-train/src/bin/cpu-copy-key-reply.rs \
  docs/labs/one-model-end-to-end/instrument
printf '%s  %s\n' \
  "$TOKENIZER_SHA" "$TOKENIZER" \
  "$SEED1_SHA" "$SEED1" \
  "$SEED2_SHA" "$SEED2" \
  "$TT1_SHA" "$TRAIN/seed1-tt.json.gz" \
  "$TT2_SHA" "$TRAIN/seed2-tt.json.gz" \
  "$TRAIN_TOKENS_SHA" "$TRAIN/panels/tt-2711/tokens.u16" \
  "$TRAIN_MASK_SHA" "$TRAIN/panels/tt-2711/response_mask.u8" \
  "$TRAIN_MANIFEST_SHA" "$TRAIN/panels/tt-2711/manifest.json" |
  sha256sum -c -

mkdir -p "$PANELS" "$OUT/reports" "$OUT/oracle" "$OUT/gate"
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
  --bin cpu-copy-key-reply --bin cpu-copy-key-reply-oracle \
  --bin cpu-copy-generated-stop
/usr/bin/time -v -o "$OUT/generator-build.time.txt" \
  env CARGO_BUILD_JOBS=16 CARGO_TARGET_DIR="$GEN_TARGET" \
  cargo build --locked \
  --manifest-path "$SOURCE/docs/labs/one-model-end-to-end/instrument/Cargo.toml" \
  --features panel-writer --bin heldout-panel
GEN=$GEN_TARGET/debug/heldout-panel

/usr/bin/time -v -o "$OUT/panel.time.txt" \
  timeout 900s "$GEN" out="$PANEL" tokenizer="$TOKENIZER" \
  cell=dev,dev conversations=128 seed=2722 protocol=2 \
  >"$OUT/panel.stdout.json"
jq -e --arg tokenizer_sha "$TOKENIZER_SHA" '
  .schema == "uor-r4-chat-corpus/v1"
  and .seed == 2722
  and .dialogue_protocol_version == 2
  and .cell.value == "dev"
  and .cell.phrasing == "dev"
  and .conversations == 128
  and .world == "m-world-v2"
  and .tokenizer.sha256 == $tokenizer_sha
' "$PANEL/manifest.json" >/dev/null

gzip -dc "$TRAIN/seed1-tt.json.gz" >"$OUT/reports/seed1-tt.json"
gzip -dc "$TRAIN/seed2-tt.json.gz" >"$OUT/reports/seed2-tt.json"
printf '%s  %s\n' \
  "$TT1_RAW_SHA" "$OUT/reports/seed1-tt.json" \
  "$TT2_RAW_SHA" "$OUT/reports/seed2-tt.json" |
  sha256sum -c -
POWER_OK=1
for seed in seed1 seed2; do
  case "$seed" in
    seed1) artifact=$SEED1; artifact_sha=$SEED1_SHA ;;
    seed2) artifact=$SEED2; artifact_sha=$SEED2_SHA ;;
  esac
  /usr/bin/time -v -o "$OUT/reports/$seed-dd.time.txt" \
    timeout 900s env RAYON_NUM_THREADS=32 \
    "$BUILD/release/cpu-copy-key-reply" \
    "$PANEL/tokens.u16" "$PANEL/response_mask.u8" "$PANEL/manifest.json" \
    "$artifact" --cap 64 --seed 153 >"$OUT/reports/$seed-dd.json"
  if ! jq -e '.random_nonvacuity.non_vacuous == true' \
    "$OUT/reports/$seed-dd.json" >/dev/null; then
    printf '%s random-control-vacuous\n' "$seed" >>"$OUT/instrument-fault.txt"
    POWER_OK=0
  fi
  if ! jq -e '.examples.reachable >= 200' \
    "$OUT/reports/$seed-dd.json" >/dev/null; then
    printf '%s reachable-below-200\n' "$seed" >>"$OUT/power-limited.txt"
    POWER_OK=0
  fi
  if ! jq -e --arg expected "$artifact_sha" \
    '.inputs.artifact_sha256 == $expected' \
    "$OUT/reports/$seed-dd.json" >/dev/null; then
    printf '%s artifact-hash-mismatch\n' "$seed" >>"$OUT/instrument-fault.txt"
    POWER_OK=0
  fi
  "$BUILD/release/cpu-copy-key-reply-oracle" \
    "$OUT/reports/$seed-dd.json" "$PANEL/tokens.u16" \
    "$PANEL/response_mask.u8" "$PANEL/manifest.json" \
    >"$OUT/oracle/$seed-dd.json"
  if ! jq -e '.arms.longest_4_to_1.reading.exact_reference >= 7' \
    "$OUT/oracle/$seed-dd.json" >/dev/null; then
    printf '%s fresh-oracle-below-7\n' "$seed" >>"$OUT/power-limited.txt"
    POWER_OK=0
  fi
done

test ! -e "$OUT/instrument-fault.txt"
if test "$POWER_OK" -ne 1; then
  printf 'POWER_LIMITED\n' >"$OUT/outcome.txt"
  exit 0
fi

for seed in seed1 seed2; do
  case "$seed" in
    seed1) artifact=$SEED1 ;;
    seed2) artifact=$SEED2 ;;
  esac
  /usr/bin/time -v -o "$OUT/gate/$seed.time.txt" \
    "$BUILD/release/cpu-copy-generated-stop" \
    "$artifact" "$OUT/reports/$seed-tt.json" \
    "$TRAIN/panels/tt-2711/tokens.u16" \
    "$TRAIN/panels/tt-2711/response_mask.u8" \
    "$TRAIN/panels/tt-2711/manifest.json" \
    "$OUT/reports/$seed-dd.json" "$PANEL/tokens.u16" \
    "$PANEL/response_mask.u8" "$PANEL/manifest.json" \
    --exact-body --prototypes 8 --seed 157 \
    --min-train-exact 6 --min-dev-exact 7 --min-dev-replies 200 \
    --min-dev-oracle 7 --min-distinct-terminal 8 \
    >"$OUT/gate/$seed.json"
  /usr/bin/time -v -o "$OUT/gate/$seed-repeat.time.txt" \
    "$BUILD/release/cpu-copy-generated-stop" \
    "$artifact" "$OUT/reports/$seed-tt.json" \
    "$TRAIN/panels/tt-2711/tokens.u16" \
    "$TRAIN/panels/tt-2711/response_mask.u8" \
    "$TRAIN/panels/tt-2711/manifest.json" \
    "$OUT/reports/$seed-dd.json" "$PANEL/tokens.u16" \
    "$PANEL/response_mask.u8" "$PANEL/manifest.json" \
    --exact-body --prototypes 8 --seed 157 \
    --min-train-exact 6 --min-dev-exact 7 --min-dev-replies 200 \
    --min-dev-oracle 7 --min-distinct-terminal 8 \
    >"$OUT/gate/$seed-repeat.json"
  cmp "$OUT/gate/$seed.json" "$OUT/gate/$seed-repeat.json"
  jq -r \
    'if .train.bar.passes == true and .dev.bar.passes == true
     then "KEEP" else "REJECT" end' \
    "$OUT/gate/$seed.json" >"$OUT/gate/$seed.outcome.txt"
done
cat "$OUT"/gate/seed*.outcome.txt >"$OUT/outcomes.txt"
```

The driver's `train.bar.passes` and `dev.bar.passes` contain every registered
exact, power, control and premature-stop input. The wrapper requires both for
each artifact seed in `*.outcome.txt`; it also enforces the final fresh-process
`cmp`. The exit trap seals all outputs in `SHA256SUMS`, including the
power-limited or instrument record when one applies.

## Minimum test power

- two artifact seeds;
- at least 200 reachable fresh-dev rows per artifact seed;
- at least eight distinct exact-body terminal signatures in train;
- a fresh-dev oracle ceiling of at least seven exact bodies;
- deterministic evaluator, so a fresh process must reproduce every decision.

If the fresh panel has fewer than seven oracle-exact bodies, classify the
confirmation **POWER-LIMITED** and do not call the gate negative.

## Bar

The mechanism passes only if, on both artifact seeds:

- the exact-body gate fits a nonzero threshold under the selected-row 1%
  training false-stop cap and produces at least 6/11 exact train replies;
- fresh-dev exact replies are at least seven and at least
  `ceil(fresh_oracle / 2)`;
- true exact replies beat shuffled exact-body, all-reference-length and
  no-gate arms by the paired two-sigma rule:
  `candidate_only >= max(2, ceil(2*sqrt(control_only)))`;
- premature BOS stops are at most `floor(replies / 100)` on train and fresh
  dev;
- the fresh-process repeat matches every non-timing field;
- all input and selected-trace hashes match.

The evaluator is deterministic. Process noise is therefore zero by contract;
artifact variation is covered by requiring both seeds, and the paired rule
requires a candidate-only gain of at least two replies.

## Stop rule

Stop as a power limit if train has fewer than eight distinct terminal
signatures, fresh dev has fewer than 200 reachable rows, or fresh dev has
fewer than seven oracle-exact bodies. Stop as an instrument fault on any
source, artifact, tokenizer, panel or trace hash mismatch; unreplayable or
non-monotonic decisions; vacuous shuffled labels; nondeterminism; or a need to
change serving, CUDA, float training or Claude-owned source. Once fresh dev is
read, do not change the gate, panel, seeds or bar.

## Cost

- Host: AMD Ryzen 9 9950X, 16 physical cores / 32 threads, 59 GiB RAM.
- Panel generation: one fresh dev panel, capped at 900 seconds.
- Copy reports: two artifact seeds at 32 Rayon workers, capped at 900 seconds
  each.
- Gate runs: two artifact seeds plus two deterministic repeats, about nine
  seconds each.
- Build and checks: under five minutes incremental.
- Complete run, evidence and delivery projection: under 45 minutes.
- Local-only cost: $0; no GPU pod, paid API, remote upload or new paid panel.
- New retained evidence: under 32 MiB including the generated panel; scratch
  target reuse is required and capped at the existing working target.

## Outcomes

- KEEP: the exact-body gate clears every bar item on the unopened fresh panel;
  the next unit tests a copy-boundary construction independent of oracle
  filtering.
- REJECT: the gate fits but fails any fresh exact, control or premature-stop
  item; close exact-body filtering with this prototype family.
- POWER-LIMITED: the fresh panel cannot meet the registered exact/reachability
  floor; preserve it without a mechanism verdict.

## TEST FITNESS

TEST FITNESS: FIT - fresh-context confirmation review at `64fac2483`; isolated
fresh root, pinned inputs and commands, machine-enforced power/exact/control
floors, fail-closed panel and hash validation, and evidence sealing verified.
No fresh panel had been generated or read.

## Pre-run GO

GO - independent fresh-context review at `3641b70f2`. The output root and
seed-2722 panel were absent; FIT and every pin matched; the command block
passed `bash -n`; power and instrument paths were fail-closed; train and dev
bars both gated each seed; and no forbidden source was in scope.
