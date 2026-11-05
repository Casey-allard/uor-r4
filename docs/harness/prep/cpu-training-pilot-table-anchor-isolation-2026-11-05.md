# Pre-registration: CPU table-anchor isolation

Run ID: `cpu-training-pilot-table-anchor-isolation-2026-11-05`

## Status

COMPLETE. Both registered panel arms ran after FIT and D67 GO.

## Question

Which part of the table-wide port anchor accounts for the decoding loss
observed in deepseek-183: changed recurrent input rows, changed writer scoring
rows, or their interaction?

This is a D84 CPU development instrument. It does not train, promote a model,
qualify a GPU result or establish language quality.

## Why

`UORLUT01` normally uses one shared token table for both the recurrent input and
the final writer. Deepseek-183 projected both 8-coordinate port blocks of every
row to their token-cell roots. Against raw frozen, admitted/top-1/top-8 fell
from 245/181/236 to 120/55/86 on seed 1 and from 171/136/164 to 29/10/20 on
seed 2. That combined intervention cannot attribute the loss.

This diagnostic holds the trunk, tokens and targets fixed and substitutes two
in-memory tables independently. No model artifact is written.

## Base artifact

Raw public FBS artifacts:

- seed 1:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
  SHA-256
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`,
  SHA-256
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.

Table-anchored artifacts from deepseek-183:

- seed 1:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/query-port-root/2026-11-05/seed-1/table-anchored.uorlut`,
  SHA-256
  `88c6d1e3a40db3d23a72a0435060147306e98bf9b7ad46d7c42e4099e1ad5a03`;
- seed 2:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/query-port-root/2026-11-05/seed-2/table-anchored.uorlut`,
  SHA-256
  `275fcde2c67153d6ea0140da067e3151616d0bbd6ed3e40439e6a69618fb3930`.

## Data split

Panel:
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/m2-evidence/dev-panel-v3`

- tokens:
  `add1f3954fc67e12822bdac395d7167bf6d6a0f074856efe2beec2c4ac81e442`;
- response mask:
  `da1ae9ee3bdc4fe1bf4a84a12ee9c1ac02ee51a419e3806cb77df430a44a3c01`;
- manifest:
  `ce489b38e91b2514f302560e65ac8bf7c337ca8a15b2aedba428e312bb003949`;
- split `dev`, vocabulary 4,096, BOS 0;
- exact filtered power: 58 conversations and 616 response rows per seed.

The panel was read in earlier CPU units. This is component calibration, not a
fresh-capability test.

## Fixed construction

The driver parses each raw and anchored artifact, rejects Hamming or turn-memory
records, reconstructs the registered root projection from the raw table and
requires the anchored artifact's table to equal it exactly.

For each response target the driver replays the same causal prefix and obtains
the final Q12 query. It then constructs four in-memory forwards:

| Arm | Recurrent input rows | Writer scoring rows |
|---|---|---|
| raw/raw | raw | raw |
| input-only | anchored | raw |
| writer-only | raw | anchored |
| combined | anchored | anchored |

The writer's target admission, chosen token, top-1/top-8 and kept cells are read
from the selected writer table. Port distance is the Q12 L1 distance from query
coordinates `0..8` and `W-8..W` to the target cell's two parsed roots.

`UORLUT01` stores one shared table. The hybrid arms are therefore diagnostic
only and make no export or serving claim. Only raw/raw must match the integer
serving session exactly.

## Source and binary

- base: `1c5209f8e`;
- first-run implementation: `1f02345e69abc4c6fcc7bf906b63f900df6ab025`;
- final implementation:
  `40542adf069f1fe5b5d97a0df136eac4b6d35188`;
- first-run binary:
  `target/deepseek/table-anchor-isolation/release/release/cpu-table-anchor-isolation`,
  SHA-256
  `ab39c515f15856245d5681dc2230f1fa184a959224739d0933dcf10a7ec58e84`;
- serial review-repair binary SHA-256:
  `4623d93c0a1e099b5fc942b079f0e0ce2482300749250ff5817bb9bf06815bcc`;
- final ordered-parallel binary SHA-256:
  `e195e52ec20cac06287410fbe15bdd2d269b5ca4f37f0a4edb6616229ce850c6`;
- host: AMD Ryzen 9 9950X, 16 physical cores and 32 hardware threads;
- final execution: 16 actual Rayon workers over ordered independent rows.

## Output

Initial serial discovery:
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/table-anchor-isolation/2026-11-05/seed-{1,2}`.

Final 16-worker verification:
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/table-anchor-isolation/2026-11-05/parallel-verification/seed-{1,2}`.

Both output roots are absent at registration.

## Steps

### Historical initial run

The first run used implementation `1f02345e6` and binary SHA-256 `ab39c515…`.
Its sealed outputs are retained under `seed-{1,2}`. That implementation
reported the Rayon pool size but still evaluated rows serially.

### Final reproducible verification

The final driver at implementation `40542adf0`, binary SHA-256 `e195e52e…`,
evaluates independent rows and serving checks in source order with 16 workers.
Choose a new absent output root:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-table-anchor-isolation
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/table-anchor-isolation/REPRODUCTION-DATE
BIN=$SOURCE/target/deepseek/table-anchor-isolation/release/release/cpu-table-anchor-isolation
PANEL=$SHARED/local/deepseek/m2-evidence/dev-panel-v3

test ! -e "$OUT"
git -C "$SOURCE" merge-base --is-ancestor 40542adf069f1fe5b5d97a0df136eac4b6d35188 HEAD
test "$(sha256sum "$BIN" | cut -d' ' -f1)" = e195e52ec20cac06287410fbe15bdd2d269b5ca4f37f0a4edb6616229ce850c6

mkdir -p "$OUT"
run() {
  seed=$1 raw_sha=$2 anchored_sha=$3
  RAYON_NUM_THREADS=16 timeout 1200s "$BIN" \
    "$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-$seed-raw_both.uorlut" \
    "$SHARED/local/deepseek/query-port-root/2026-11-05/seed-$seed/table-anchored.uorlut" \
    "$PANEL/tokens.u16" "$PANEL/response_mask.u8" "$PANEL/manifest.json" \
    "$OUT/seed-$seed" \
    --expected-raw-sha256 "$raw_sha" \
    --expected-anchored-sha256 "$anchored_sha" \
    --expected-tokens-sha256 add1f3954fc67e12822bdac395d7167bf6d6a0f074856efe2beec2c4ac81e442 \
    --expected-mask-sha256 da1ae9ee3bdc4fe1bf4a84a12ee9c1ac02ee51a419e3806cb77df430a44a3c01 \
    --expected-manifest-sha256 ce489b38e91b2514f302560e65ac8bf7c337ca8a15b2aedba428e312bb003949 \
    >"$OUT/seed-$seed.stdout.json" 2>"$OUT/seed-$seed.stderr.log"
}

run 1 \
  a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd \
  88c6d1e3a40db3d23a72a0435060147306e98bf9b7ad46d7c42e4099e1ad5a03
run 2 \
  da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1 \
  275fcde2c67153d6ea0140da067e3151616d0bbd6ed3e40439e6a69618fb3930
```

## Minimum power and checks

- two artifact seeds and 616 response rows per seed;
- four arms on every row, with 157,696 Q12 coordinates per arm per seed;
- complete query, token and kept-cell signatures for every arm;
- every arm run twice in-process with exact row equality;
- one-worker and four-worker ordered row/parity tests are identical;
- raw/raw exact integer-serving parity over every final query coordinate, token
  and kept-cell vector;
- combined arm exactly equal to the parsed table-anchored artifact forward;
- anchored table exactly equal to the freshly reconstructed projection;
- zero Hamming, turn memory, malformed input, or partial-row coverage.

## Attribution

For each seed, report the raw/raw value and the signed delta from raw/raw for:

- admitted targets;
- top-1 targets;
- top-8 targets;
- target port-distance sum;
- changed query, token and kept-cell rows.

The REPORT assigns the combined effect using input-only and writer-only
observed deltas and reports the residual interaction. A larger input-only loss,
writer-only loss, or interaction is a diagnostic attribution only; no arm is
promoted.

## Instrument bar

**COMPLETE** only if all four arms execute on both artifacts, all checks above
hold, and complete deterministic reports are sealed. Otherwise the result is
an instrument fault and clears no attribution.

## Cost and stop

- local CPU only, $0;
- two registered runs, 16 workers, at most 20 minutes each;
- stop prematurely above 1 GiB peak RSS;
- stop on source, binary, artifact, panel or table drift, nondeterminism,
  incomplete rows, Hamming or memory records, a hybrid serving-parity claim, or
  any need to edit serving, CUDA, float-training, exporter or Claude-owned
  source;
- do not retune after a report is read. Review-driven instrument repairs use
  fresh output roots and preserve every earlier sealed report.

## Claim boundary

A completed diagnostic attributes this four-arm table substitution on two
public CPU-development artifacts. It does not make the hybrid tables
representable, establish a trained model, qualify a GPU result, or show a
language, memory or complete-reply gain.

## Verdict

TEST FITNESS: FIT - M2 #2030
sheet: docs/harness/prep/cpu-training-pilot-table-anchor-isolation-2026-11-05.md
reviewer: deepseek fresh-context subagent table_anchor_fit_d67_2
note: No blockers at a5e472c36; source, binary, hashes, panel power, absent
  outputs, hybrid serving boundary, determinism/parity and D84 scope verified;
  public arms not run.
date: 2026-11-05

A FIT verdict clears compute for the configuration in this sheet only (D22).

## Pre-run GO

GO - reviewer `deepseek fresh-context subagent table_anchor_fit_d67_2`,
2026-11-05, at clean review head
`a5e472c363f848be7ff4b5b462a6b5391478afeb`. The reviewer verified the
four-arm construction, raw/raw serving parity, hybrid no-serving boundary,
combined-artifact equality, fail-closed Hamming/memory/shape checks, all input
and binary hashes, absent outputs, 58/616 panel power, full CPU tests, focused
tests, scoped strict Clippy, formatting, D23/D32 ratchets and D84 scope. The
public-panel arms had not run.

## Outcome

**COMPLETE (instrument).** Both sealed seeds completed, every arm was
deterministic, and raw/raw retained exact serving parity.

The exact reviews repaired a weak driver floor and exposed the initial run's
serial execution. The final driver requires exactly 616 response rows, tests
the malformed writer table, and evaluates independent rows and serving checks
with ordered Rayon workers. Fresh sealed 16-worker verification roots with
binary SHA-256 `e195e52e…` reproduced every arm field exactly. Seed 1 took
**2.598 s / 115,880 KiB** and seed 2 **2.593 s / 115,980 KiB**, against about
35 seconds for the preserved initial serial runs on the same local host.

The anchored recurrent-input arm reproduced the combined decoding collapse:
admitted/top-1/top-8 was `120/55/89` versus raw `245/181/236` on seed 1 and
`29/9/20` versus `171/136/164` on seed 2. Writer-only preserved raw admission
and port distance, with top-1/top-8 `195/233` and `137/162`.

The result attributes the loss to the input-code change, not direct writer-row
mismatch. The one-table artifact cannot represent the hybrid split, so this is
an in-memory diagnostic and no serving or model result.
