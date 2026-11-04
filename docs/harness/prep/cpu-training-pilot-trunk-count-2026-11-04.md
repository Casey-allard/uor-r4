# Public 19-table CPU count training

**Status:** FIT

## Question

Can the independent CPU path derive exact trunk training rows from a public M1
artifact, add them to all 13 sufficient-statistic count tables, export a new
19-table artifact, and preserve exact CPU/integer serving decisions on
document-disjoint evaluation rows?

## Compute class

Local CPU development research under D84 on the Ryzen 9 9950X. No GPU, pod or
paid API. This is not GPU qualification or language-quality evidence.

## Base artifact

| Input | Identity |
|---|---|
| implementation source | `1ca202ef217918d4c8444499ebaabc3a91e1878a` |
| public M1 trunk | `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |

The artifact is
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/t16-frozen-zero-seed1-trunk.d11`.
The corpus files are in the same directory.

## Data split and arms

Selection-only was run before this FIT revision and fixed the split:

- 200 training documents / 7,080 eligible targets;
- 193 evaluation documents / 512 evaluation targets at stride 2;
- the first 128 training targets are used for count updates.

Arms:

1. `no_update`: the source 19-table artifact;
2. `additive`: exact target counts from the training rows;
3. `shuffled`: the same training prefixes with a deterministic non-identity
   permutation of target assignments.

Every arm evaluates the same held-out rows through both `CpuDiscreteTrunkReader`
and `IntegerDiscreteModel`.

## Training procedure

For each training prefix and target, derive:

- the base CPU read code;
- active recurrence, shuffled-order recurrence and unit-normalization codes;
- the final query token and supervised target.

Increment the unigram and all 12 sparse tables exactly as `TrunkTables::build`
does. Counts are checked for overflow. `unigram_best_global` follows count
descending then smaller token id. Export with `export_trunk_d11`; re-import and
require every snapshot/routes/gates/step/norm value and all tables to match.

## Bar

Instrument and mechanism gates:

- 19 tables remain present and parse;
- codebook, routes, gates, recurrence candidates and normalization candidates
  are unchanged;
- exact arm's decoded count tables equal source tables plus the independently
  tallied training rows;
- shuffled and exact differ in target assignments, while their prefix-derived
  codes and non-count sections stay equal;
- every arm has zero query-coordinate, admitted-cell, top-8 and token
  mismatches against integer serving on every held-out row;
- a fresh seed-1 repeat matches every non-timing field byte for byte;
- training and shuffled artifacts are deterministic across construction.

The capability readings (admitted, top-1, top-8) are descriptive. A change is
not a positive language result and no change is not a training failure.

## Steps

Run only after FIT. The source worktree and shared input/result root are
separate:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-trunk-count-training
SHARED=/home/callard/projects/codex-playground/uor-r4
SOURCE_PIN=1ca202ef217918d4c8444499ebaabc3a91e1878a
PIPELINE=$SHARED/local/deepseek/trunk-count/2026-11-04-attempt-3
BUILD=$SOURCE/target/deepseek/trunk-count-build-124-attempt-3
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
    --train-rows 128 --evaluation-stride 2 --max-evaluation 512 \
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

The repeat removes exactly the top-level `wall_elapsed_ms` and
`peak_rss_kib`; this report schema has no nested timing field. Any source,
binary, argv, output-root or deterministic mismatch is an instrument fault
requiring a new FIT.

## Minimum test power

One public trained artifact is sufficient for writeback correctness. Held-out
coverage must include every selected evaluation row and all four decision
surfaces: Q12 query, admitted cells, top-8 and token. A second seed or a
capability bar is not required for this host mechanism unit.

## Cost and stop

Local workstation only; no GPU, pod or paid API. Rust 1.97.1 on Linux
7.0.0-42-generic x86_64. At registration the project filesystem had
548,723,466,240 bytes free and `/tmp` had 26,339,098,624 bytes free. The fresh
build is capped below 3 GiB temporary storage; two output roots retain below
400 MiB because each contains two approximately 86 MiB trunk artifacts. The
128 MiB reserve remains intact. Each run has a 600-second timeout and 2 GiB
peak-RSS cap. Stop on artifact drift, semantic-section change, count
mismatch/overflow, nondeterminism, output-root reuse or CPU/serving divergence.

Attempt 1 is preserved at
`local/deepseek/trunk-count/2026-11-04-attempt-1/`. It completed its mechanism
checks but reached 3,063,024 KiB peak RSS, so the resource rule stopped it
before repeat or evidence acceptance. Attempt 2 adds explicit drops of the
source serving model and training reader before later verification/evaluation.
Attempt 2 is preserved at `local/deepseek/trunk-count/2026-11-04-attempt-2/`
and was also stopped at 2,297,824 KiB. Attempt 3 consumes each training reader,
writes and releases one artifact before constructing the next, and reloads each
artifact only for its own sequential verification/evaluation phase.

## Claim boundary

This proves CPU writeback over the full public 19-table family. It does not show
that added counts improve held-out quality, complete replies, fresh phrasing,
useful chat or GPU-qualified training.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 public-19-table-count-training
sheet: `docs/harness/prep/cpu-training-pilot-trunk-count-2026-11-04.md`
head: `3baa15318af8b9e9fab855ab1eea5ff0fc54b02c`
implementation pin: `1ca202ef217918d4c8444499ebaabc3a91e1878a`
reviewer: deepseek fresh-context non-author subagent
note: attempts 1 and 2 excluded as resource stops above the 2-GiB cap; fresh
attempt-3 roots; streaming reader/artifact phases; exact CPU row derivation,
independent full decoded table counts, checked bounds, smaller-id unigram tie,
deterministic non-identity shuffled targets, semantic preservation, 128/512
document-disjoint rows, exact CPU/serving parity, binary/output checksums,
600-second/2-GiB bounds, storage bound and deterministic repeat
date: 2026-11-04

A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone
before compute.
