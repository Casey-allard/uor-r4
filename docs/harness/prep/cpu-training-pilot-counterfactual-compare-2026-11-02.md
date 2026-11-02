# Prep sheet: bounded CPU counterfactual parameter comparison, 2 November 2026

Lab: deepseek. Item:
[cpu-training-pilot](../../roadmap/items/cpu-training-pilot.md) (#3506).
Order:
[deepseek-11](../../loop/tasks/cpu-training-pilot/deepseek-11/ORDER.md).
Source: `0325ccc07de9b651913fcf1bfcac850b5c2b93be`.

TEST FITNESS: FIT

PRE-RUN: PASS

## Question

The prior CPU rank-margin pilot updated tied token-table coordinates and failed
its public ranking bar. Can a different exact learning mechanism improve held
out public ranking by updating non-token FBS parameters through bounded BVG-K
counterfactual moves?

Four independently reported groups are tested: normalization gains, convolution
and gate biases, decay offsets, and the six FBS route tables. Each is compared
with its own zero-update control on both public seeds.

This is public internal-development research under D84. It does not establish
language capability, complete replies, fresh phrasing or GPU qualification.

## Base artifact

- Runner source:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/crates/uor-r4-cpu-train/src/bin/cpu-counterfactual-corpus-driver.rs`,
  SHA-256 `57dfe4e21f4d997612c2225f8257d0ef48bf2d95085114916fb605ed05b0b346`.
- Trainer source:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/crates/uor-r4-cpu-train/src/counterfactual.rs`,
  SHA-256 `3996837bc5abb805c3f1972cf506e6b34c964e5cfc14d070fc6ecc9e3a1649f6`.
- Corpus source:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/crates/uor-r4-cpu-train/src/corpus.rs`,
  SHA-256 `cdb1d18d0b07d248f0296146ef654d425005bf0ab3c39e60ee46b354268a5fa0`.
- Executable:
  `/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/counterfactual-prereg-target/release/cpu-counterfactual-corpus-driver`,
  SHA-256 `42d0768cd6b0d359ba229daa9f06b74dd115d6276de6541b8b2eef0b8feef59a`,
  2,244,648 bytes.
- Build command:

```sh
CARGO_TARGET_DIR=/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/counterfactual-prereg-target \
  cargo build --locked -p uor-r4-cpu-train --release \
  --bin cpu-counterfactual-corpus-driver
```

- Seed 1:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
  SHA-256 `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`.
- Seed 2:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`,
  SHA-256 `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.

Both artifacts are public raw FBS models: vocabulary 4,096, context 192,
native writer, with no Hamming read or turn memory. Re-hash the executable after
the run.

## Data split

- Tokens:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-tokens.u16`,
  SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`.
- Response mask:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-response-mask.u8`,
  SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- Manifest:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-manifest.json`,
  SHA-256 `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab`.
- Tokenizer identity:
  `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.

The existing strict loader creates document-disjoint training rows and an
evaluation stride of 2, evenly capped at 1,024 rows. The public manifest names
the source `train`; exposure history is unverified. These are not sealed
held-out rows.

## Arms

All arms use trainer seed 1, margin Q12 1, delta 1, four distinct coordinates
per BVG-K step, evaluation stride 2 and a 1,024-row cap.

| Arm | Group | Steps |
|---|---|---:|
| baseline | zero-update control | 0 |
| normalization | normalization gains | 128 |
| bias | convolution and gate biases | 128 |
| decay | recurrence decay offsets | 128 |
| route | six FBS route tables | 128 |

Each artifact seed runs all five arms. The baseline and every trained arm start
from a fresh load of the pinned artifact. The four trained groups are
independent configurations, not pooled samples.

## Bar

For each seed, first require the instrument gates:

- process exit 0, complete JSON, finite readings and the expected 1,024-row
  document-disjoint evaluation;
- baseline and final Q12 query, token and kept-cell mismatches exactly zero;
- final artifact parses and checkpoint resume reproduces the final artifact;
- baseline artifact bytes and hash remain unchanged; and
- every trained arm must record positive candidate work. Zero accepted updates
  from positive candidate work is a preserved configuration negative, not an
  instrument pass.

For each group and seed, compare against that seed's zero-update control:

- target admission cannot fall;
- top-1 or top-8 must rise by at least
  `ceil(3 * sqrt(1024 * p * (1 - p)))`, where `p` is the control's rate for
  that metric;
- the other ranking metric cannot fall; and
- the final pool gate must accept the artifact.

**Replicated positive:** one named group passes every row on both seeds.
Because four groups are tested, this is screening evidence, not a family-wide
significance claim. No replicated group is a negative for the tested 128-step,
four-coordinate, delta-one settings, not for its mechanism family.

## Measurements

- baseline/final top-1, top-8, admission, margin-loss sum and rank sum;
- accepted, refused, joint and single updates; candidates tried, eligible
  coordinates, coordinates scanned and accepted members;
- final artifact/checkpoint SHA-256 and exact reload equality;
- CPU/serving query, token and kept-cell mismatches;
- elapsed seconds and process peak RSS per arm.

## Steps

Use the fresh output root:

`/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/cpu-counterfactual-compare-2026-11-02/`

It must not exist. Run one process at a time:

```sh
set -eu
WORK=/home/callard/projects/codex-playground/uor-r4-chat-baseline
ROOT=/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/cpu-counterfactual-compare-2026-11-02
BIN=/home/callard/projects/codex-playground/uor-r4-chat-baseline/local/deepseek/counterfactual-prereg-target/release/cpu-counterfactual-corpus-driver
DATA=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity
ART=/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual
SOURCE_RUNNER=$WORK/crates/uor-r4-cpu-train/src/bin/cpu-counterfactual-corpus-driver.rs
SOURCE_TRAINER=$WORK/crates/uor-r4-cpu-train/src/counterfactual.rs
SOURCE_CORPUS=$WORK/crates/uor-r4-cpu-train/src/corpus.rs
test ! -e "$ROOT"
sha256sum -c <<PINS
57dfe4e21f4d997612c2225f8257d0ef48bf2d95085114916fb605ed05b0b346  $SOURCE_RUNNER
3996837bc5abb805c3f1972cf506e6b34c964e5cfc14d070fc6ecc9e3a1649f6  $SOURCE_TRAINER
cdb1d18d0b07d248f0296146ef654d425005bf0ab3c39e60ee46b354268a5fa0  $SOURCE_CORPUS
42d0768cd6b0d359ba229daa9f06b74dd115d6276de6541b8b2eef0b8feef59a  $BIN
a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd  $ART/dual-c131-seed-1-raw_both.uorlut
da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1  $ART/dual-c131-seed-2-raw_both.uorlut
0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b  $DATA/dev-tokens.u16
a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078  $DATA/dev-response-mask.u8
ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab  $DATA/dev-manifest.json
PINS
mkdir -p "$ROOT"
run_arm() {
  seed=$1
  arm=$2
  steps=$3
  artifact="$ART/dual-c131-seed-$seed-raw_both.uorlut"
  out="$ROOT/seed-$seed/$arm"
  mkdir -p "$out"
  /usr/bin/time -v -o "$out/time.txt" \
    "$BIN" "$artifact" "$DATA/dev-tokens.u16" "$DATA/dev-response-mask.u8" \
    "$DATA/dev-manifest.json" "$out" --seed 1 --group "$arm" --steps "$steps" \
    --margin-q12 1 --delta 1 --coordinates-per-step 4 \
    --evaluation-stride 2 --max-evaluation 1024 \
    >"$out/result.json" 2>"$out/stderr.txt"
}
for seed in 1 2; do
  run_arm "$seed" baseline 0
  run_arm "$seed" normalization 128
  run_arm "$seed" bias 128
  run_arm "$seed" decay 128
  run_arm "$seed" route 128
done
sha256sum "$BIN" >"$ROOT/postflight-binary.sha256"
find "$ROOT" -type f ! -name SHA256SUMS -print0 |
  sort -z | xargs -0 sha256sum >"$ROOT/SHA256SUMS"
```

Apply the bar from the raw JSON. A missing or inconsistent arm is an instrument
failure. Keep every completed arm, including a negative.

## Minimum test power

Two published model seeds, the same 1,024-row document-disjoint evaluation on
each, and four distinct parameter groups. The three-standard-deviation ranking
threshold is a replication screen for development evidence. It cannot estimate
fresh-language behavior or correct for the public base model's exposure.

## Cost

Local Ryzen 9 9950X, no GPU, pod or paid service. At registration the filesystem
had `574,318,002,176` bytes available, above the required `30 GiB + 128 MiB`;
the build target is 255,811,474 bytes and expected evidence is under 1 GiB.
The 10 sequential processes are projected below 45 minutes. No automatic retry.

## Stop rule

Stop on a source/artifact/corpus/hash mismatch, pre-existing output root,
nonzero process exit, malformed JSON, missing output, final parse failure,
checkpoint reload mismatch, CPU/serving mismatch or storage-floor violation.
A completed arm is not shortened because its ranking is unimpressive. A zero
accepted-update arm still writes and reports its result unless candidate work
itself is zero.

## Claim boundary

The strongest result is that a named non-token parameter group improves public
document-disjoint ranking while preserving admission and exact export on both
seed artifacts. It is not useful chat, fresh phrasing, a family-wide mechanism
victory or GPU qualification.

## Non-author TEST FITNESS review

TEST FITNESS: FIT

PRE-RUN: PASS

A fresh-context deepseek reviewer checked branch head `45edffe5d` after the
preflight repair. Source commit `0325ccc07de9b651913fcf1bfcac850b5c2b93be` is
reachable and its runner/trainer/corpus bytes match the pins. The reviewer
expanded and ran the exact nine-line preflight independently: every source,
executable, artifact, token, mask and manifest pin returned `OK`. The output
root was absent, storage exceeded the floor, the runner test passed, and no
public arm or model training had run.
