# 4,096-row public 19-table count training

**Status:** FIT

## Question

On the fixed 512-row document-disjoint evaluation panel, does an exact additive
dose of 4,096 training rows improve the public 19-table trunk's top-1 over no
update and deterministic shuffled targets, without losing admitted or top-8
coverage?

## Compute class

Local CPU development research under D84 on the Ryzen 9 9950X. No GPU, pod or
paid API.

## Base artifact

| Input | Identity |
|---|---|
| implementation source | `09c4f4abb5c864ec7ac11b98cbfb4e1059394081` |
| public M1 trunk | `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

Inputs are under
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/`.

## Fixed split and dose

Replay deepseek-124 exactly:

- 200 training documents / 7,080 eligible training targets;
- 193 evaluation documents / 512 evaluation targets at stride 2;
- the first 4,096 deterministically ordered training rows.

The neutral 128-row result motivates asking for a larger dose. This sheet fixes
4,096 independently as the merged driver's pre-existing maximum before any
intermediate dose is observed; it is not chosen from a partial-result sweep.

## Arms

1. no update;
2. exact additive counts from 4,096 training rows;
3. deterministic, non-identity shuffled targets over the same prefixes.

Every arm evaluates the same 512 held-out rows.

## Bar

Pair exact and each control row by row:

- exact top-1 must exceed no update;
- exact top-1 must exceed shuffled targets;
- each gain must be at least `max(2, ceil(2 * sqrt(b + c)))`, where `b` and `c`
  are the paired discordant top-1 rows;
- exact admitted and top-8 counts must not be lower than either control.

All instrument gates must pass. A positive is bounded evidence for this public
panel and dose. A negative closes only this 4,096-row additive configuration.

## Instrument gates

- source pin, artifact, corpus and manifest hashes;
- fresh output/build roots and no source drift;
- exact and shuffled artifacts differ;
- all 13 count tables increase by exactly 4,096;
- independent full decoded table contents match each arm;
- codebook, routes, gates, recurrence and normalization sections unchanged;
- zero query-coordinate, admitted-cell, top-8 and token mismatches against
  integer serving;
- deterministic artifact/report repeat;
- peak RSS at most 2,097,152 KiB and wall at most 600 seconds per run.

## Steps

Run only after FIT:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-trunk-count-dose
SHARED=/home/callard/projects/codex-playground/uor-r4
SOURCE_PIN=09c4f4abb5c864ec7ac11b98cbfb4e1059394081
PIPELINE=$SHARED/local/deepseek/trunk-count-dose/2026-11-04-attempt-1
BUILD=$SOURCE/target/deepseek/trunk-count-dose-build-125
DATA=$SHARED/local/deepseek/public-m1-parity
test -z "$(git -C "$SOURCE" status --porcelain)"
git -C "$SOURCE" diff --quiet "$SOURCE_PIN" -- crates Cargo.toml Cargo.lock
test ! -e "$PIPELINE"
test ! -e "$BUILD"
(
  cd "$SOURCE"
  CARGO_TARGET_DIR="$BUILD" cargo build --locked --release \
    -p uor-r4-cpu-train --bin cpu-trunk-count-driver
)
BIN="$BUILD/release/cpu-trunk-count-driver"
BIN_SHA=$(sha256sum "$BIN" | cut -d' ' -f1)
run() {
  out=$1
  test ! -e "$out"
  /usr/bin/time -v timeout 600s "$BIN" \
    "$DATA/t16-frozen-zero-seed1-trunk.d11" \
    "$DATA/dev-tokens.u16" "$DATA/dev-response-mask.u8" \
    "$DATA/dev-manifest.json" "$out" \
    --expected-artifact-sha256 04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d \
    --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
    --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
    --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
    --train-rows 4096 --evaluation-stride 2 --max-evaluation 512 \
    --seed 1 >/dev/null
  test "$(sha256sum "$BIN" | cut -d' ' -f1)" = "$BIN_SHA"
  test "$(jq -er '.peak_rss_kib' "$out/report.json")" -le 2097152
  printf '%s  cpu-trunk-count-driver\n' "$BIN_SHA" >"$out/binary.sha256"
  (
    cd "$out"
    find . -type f ! -name SHA256SUMS -print0 |
      sort -z | xargs -0 sha256sum >SHA256SUMS
    sha256sum -c SHA256SUMS --quiet
  )
}
run "$PIPELINE/seed-1"
run "$PIPELINE/seed-1-repeat"
cmp "$PIPELINE/seed-1/exact.uorlut" "$PIPELINE/seed-1-repeat/exact.uorlut"
cmp "$PIPELINE/seed-1/shuffled.uorlut" "$PIPELINE/seed-1-repeat/shuffled.uorlut"
diff <(jq -S 'del(.wall_elapsed_ms, .peak_rss_kib)' "$PIPELINE/seed-1/report.json") \
  <(jq -S 'del(.wall_elapsed_ms, .peak_rss_kib)' "$PIPELINE/seed-1-repeat/report.json")
```

The report schema has only top-level timing/resource fields, so those are the
complete normalization. Anything else changing across the repeat is an
instrument fault.

## Minimum test power

The fixed panel has 512 document-disjoint evaluation rows, above D22's 200-row
count floor. The public DEV panel is not genuinely fresh with respect to the
base model, so this is bounded development evidence and does not satisfy a
fresh-transfer bar. One public artifact is sufficient for this host comparison
because deepseek-124 established the mechanism and parity on it.

## Cost and stop

Local workstation only; no GPU, pod or paid API. Rust 1.97.1 on Linux
7.0.0-42-generic x86_64. At registration the project filesystem had
548,723,466,240 bytes free and `/tmp` had 26,339,098,624 bytes free. The fresh
build is capped below 3 GiB temporary storage. Each output root retains below
200 MiB; two roots stay below 400 MiB, leaving the 128 MiB reserve intact.
Each run has a 600-second timeout and 2 GiB peak-RSS cap. Stop on cap failure,
source drift, a required code change, count/parity mismatch, nondeterminism or
an evaluation-tuned dose.

## Claim boundary

A positive supports only this additive count dose on the fixed public DEV split.
It does not establish fresh phrasing, complete replies, useful chat or GPU
qualification.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 public-19-table-count-dose
sheet: `docs/harness/prep/cpu-training-pilot-trunk-count-dose-2026-11-04.md`
head: `0c108f6c8c78c6d9e8ca596680a7741754fc2072`
implementation pin: `09c4f4abb5c864ec7ac11b98cbfb4e1059394081`
reviewer: deepseek fresh-context non-author subagent
note: fixed 4,096-row additive count dose, 512 document-disjoint public DEV
rows, exact/no-update/shuffled arms, paired top-1 margin, admitted/top-8
non-loss, independent full table reconstruction, exact CPU/serving parity,
fresh roots, binary/output checksums, 600-second/2-GiB bounds and deterministic
repeat
date: 2026-11-04

A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone
before compute.
