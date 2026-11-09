# REPORT: one-model-end-to-end, deepseek-264

Lab: deepseek. Base: `402b4091e`. Order: [ORDER.md](ORDER.md). This unit audits real
claude-339 artifacts. It trains nothing, changes no serving arithmetic, moves no bar and
uses no GPU.

## What did not work

The early and final seed-1 artifacts do not yet show the carry selecting the asked value
at the first reply position. At step 16,384 the carried artifact has **0 asked-value
first lines** and **0 complete rows**; the exact `−C` also has 0 asked-value first lines.
The carry changes ranks and continuations, but not in a way that produces the registered
recall behavior so far.

The official seed-2 pair is slightly different but does not change that diagnosis. Both
carried and `−C` have **2 asked-value first lines**, 0 complete rows and 0 EOS stops. The
carry changes 44 generation rows, 168 first-target ranks and 207 paired-first ranks, but
neither arm reaches the value reliably.

This is a diagnostic over the published final read. Claude's registered read classifies
this configuration negative at outcome row 4c; this unit supplies the implementation
controls, not the capability outcome.

## What changed

Added `real-artifact-audit` to the one-model instrument. It:

- parses a carried artifact and its supplied `−C`;
- requires identical shapes, depths and ask tables, with the carry only at the readout;
- checks optional published `SHA256SUMS` values for both artifacts;
- reproduces `−C` with `drop_hamming_carry_uorlut01` and requires byte equality;
- binds every panel and rank report to its artifact and expected engine;
- requires the explicit row count and `totals.rows`, then the same settings, rendering and
  non-artifact inputs across all reports;
- compares CPU and integer panel reports after removing only timing fields and engine identity;
- finds generation divergence by case id and token index;
- compares teacher-forced and paired-first reports by case id using the fields the panel
  actually serializes, after checking both rank inventories against the CPU generation
  report;
- reports carry-gain distribution and first-target rank changes.

The command is reusable on the remaining 16,384 cells without code changes.

At the code head `8f376d24c`, the size line was **2 code files +979/−0, 3 docs files
+202/−0, 0 tests, 0 generated, 2 other files +112/−0**. The roadmap item has no numeric
line estimate, so a size ratio is unavailable.

## Measurements

All three seed-1 L-aC artifact pairs were independently downloaded from the store and
their published `SHA256SUMS` files verified. The final carried artifact at step 16,384 is
`ba221c27…`; its reproduced `−C` is `6e85e4e2…`.

The final seed-2 audit command was:

```sh
ROOT=local/deepseek/real-artifact-audit
READ=$ROOT/official-read/results/claude/one-model-end-to-end-language-carry-2026-11-09/read
real-artifact-audit \
  --carried "$READ/served/L-aC-s2-step-016384.uorlut" \
  --minus-c "$READ/served/L-aC-s2-cut.uorlut" \
  --carried-cpu "$ROOT/official-read/panel-cpu/carried.json" \
  --carried-integer "$READ/score/L-aC-s2-step-016384.json" \
  --minus-c-cpu "$ROOT/official-read/panel-cpu/minus-C.json" \
  --minus-c-integer "$READ/score/L-aC-s2-cut.json" \
  --carried-rank "$ROOT/official-read/panel-cpu-ranked/carried.json" \
  --minus-c-rank "$ROOT/official-read/panel-cpu-ranked/minus-C.json" \
  --carried-sha256 2741d9383a80c300b90ca2d5cbc1e06a9b664d913971f84ddbc2916a1625a430 \
  --minus-c-sha256 391392f5c47af2cd6a1e091417dcc04c5340080474ebf6dad5d10ca2cfd277e5 \
  --expected-rows 256 --out "$ROOT/official-read/audit-v10.json"
```

The seed-1 runs use the same flags, with the early or final directory named in
[AUDIT.json](AUDIT.json) in place of the read-store paths.

| Step | Generation rows changed by cut | First-target rows changed | Paired-first rows changed | Asked-better carried / `−C` | First-target top-1 carried / `−C` |
|---|---:|---:|---:|---:|---:|
| 8,192 | 157 / 256 | 223 / 256 | 254 / 256 | 126 / 256 vs 125 / 256 | 0 / 256 vs 0 / 256 |
| 12,288 | 53 / 256 | 0 / 256 | 0 / 256 | 127 / 256 vs 127 / 256 | 0 / 256 vs 0 / 256 |
| 16,384 | 87 / 256 | 136 / 256 | 156 / 256 | 128 / 256 vs 127 / 256 | 0 / 256 vs 0 / 256 |

The earliest generation divergence is at token index 1 at all three seed-1 points
and token 0 for seed 2. The first changed case in id order diverges later at
6, 9, 1 and 12 respectively; [AUDIT.json](AUDIT.json) records both.

Seed 2's official artifacts are in the read store. The carried artifact hashes to
`2741d938…`, and the exact cut hashes to `391392f5…`. The audit changes 44 / 256
generation rows, 168 / 256 first-target ranks and 207 / 256 paired-first ranks.
Asked-better is 128 / 256 on both arms. The first-line count is 2 asked / 3 other / 0 both
for carried and the same for `−C`; both are 0 complete and 0 EOS at 256 cap stops.

A direct `fbs-export` from the raw sealed snapshot made a different diagnostic artifact
(`852c4721…`) and therefore read 39 changed rows. That export bypassed the official read
path's source transform, is not the published artifact, and was discarded. The audit in
this report uses the published pair and verifies both entries in the read store's
`SHA256SUMS`.

At every published seed-1 point:

- CPU generation equals the integer `serve_step` panel report on every non-timing field.
- Every non-carry artifact section and ask table is preserved.
- The supplied `−C` equals an independent reproduction byte for byte.
- The readout carry has 32 rows and 4,096 gains at depth 6.

The cut changes later teacher-forced ranks, but the first-position ranking remains
effectively tied. At 12,288, for example, 159 target/EOS-rank rows change (165 with
raw-nearest diagnostics) while no first-target rank and no paired-first rank changes. At
16,384, 198 target/EOS-rank rows change (199 with raw-nearest diagnostics); the carry
moves the paired comparison by one row while both artifacts remain at 0 asked-value first
lines and 0 complete replies.

## Checks run at the head

- `real-artifact-audit` tests: **8 passed**, including same-engine rejection,
  malformed-engine rejection, wrong-artifact rejection, partial-inventory rejection,
  cross-arm metadata rejection, missing-rank-section rejection and a rank inventory that
  diverges from the CPU generation report, plus a generation row missing its compared
  fields.
- Existing `served-reply-panel` suite with `cpu-parity`: **29 passed**.
- Real artifact audits after the review repairs: **3 / 3 passed** at steps 8,192, 12,288
  and 16,384, with both published hashes supplied and verified.
- Official seed-2 final pair: **1 / 1 passed**, with both published hashes, report identity
  and panel metadata checked. The audit receipt hashes to `86228e82…`.
- CPU panel versus stored integer report: **8 / 8 comparisons passed** across carried and
  `−C`.
- Published store `SHA256SUMS`: **4 / 4 pairs verified**.
- Strict Clippy for both binaries and changed-file rustfmt: passed.
- `uor-harness loop gates --full`: **11 / 11 green**.
- `uor-harness check`: **362 rules, 0 errors, 327 warnings**. The warnings are
  repository-wide claim, stale-branch, preparation and milestone-record findings; this
  unit adds no error.

## Skills used

No Codex skill was loaded. The audit used the project's Rust crates, stored artifacts,
panel inputs, pinned tokenizer and existing panel scorer.

## Still open

1. Audit both L-a guards once ranked reports are available; their generation reports and
   published artifacts are present.
2. Explain why the carry changes later ranks but not the first asked-value choice.
3. Re-run the same receipt against a direct `uor-chat` loopback transcript if the final
   product-path probe needs a second wrapper beyond `IntegerSession::serve_step`.

## Plain summary

The serving and control chain is clean on real artifacts: CPU and integer production
paths agree, and the carry cut is byte-exact. The geometric carry is doing something to
later ranks, but it is not yet carrying the requested value into the first reply token.
The remaining task is therefore model/query diagnosis, not export or serving repair.
