# REPORT: cpu-training-pilot, deepseek-242

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](./ORDER.md). Base: `bfa5de6ce`.

**At the first reply decision, the Hamming heads retain positions from both
prompt values, but the model chooses neither value's first token.** Across four
seed/rendering cells on the sealed Claude-322 integer artifacts, the model's
chosen first token is neither value's first token in **1,024/1,024** rows. Most
read/head cells keep at least one position from each full 7-18 token value span,
and no head consistently favors the asked value over the paired-other value.
This trace alone cannot show that a kept position is weighted or selected by
the served read, or that either complete value remains available.

## What did not work, first

- **The final first token is neither value's first token.** It equals the asked
  value's first token 0/1,024 times and the other value's first token 0/1,024
  times.
- **Keeping value positions is not selecting them.** Every read/head cell keeps
  at least one position from both full value spans in 873-992 of 1,024 rows.
  This is positional support, not complete-span retention.
- **No head has a stable role preference.** Across the four cells, asked versus
  other first-token kept counts range from 33/32 to 466/484. The largest
  positive difference is +22; the largest negative difference is -53.
- **The kept-position trace is not a weight measurement.** It records support,
  not each position's distance, margin or contribution. A position in both
  spans may still have negligible weight.

## What changed

- Added `cpu-first-reply-hamming-trace`, a read-only CPU diagnostic that:
  - renders the protocol-v2 prompt exactly;
  - locates the asked and paired-other value token spans;
  - refuses an equal-shortest accepted-answer tie instead of guessing a role
    value;
  - marks the reply start before the final prompt token;
  - traces every Hamming read and head at that first-reply decision;
  - reports kept positions and their asked/other span membership.
- Parallelized the 256 independent panel rows across 32 CPU threads. The four
  final seed/rendering reports total **2.432 s**, versus **40.049 s** for the
  equivalent sequential runs, a **16.47x** reduction without changing any
  trace count. The separate determinism repeat takes another 0.576 s.
- No model, artifact, serving path or Claude-owned file changed.

## Measurements

Artifacts: Claude-322 seed 1
`86e51d5ee2c00a4041bcde4d2183884e351099f28f155fb1e6a7516f13064dfe`
and seed 2
`8838cd1044ae7152934d8ec09016e268f2dabb0c702dd45e6ab06d7b5198bdae`.
The final executable is
`df79343eada95ee9500b0efaca5c5e2f2f720faa4334e238b8aeae284c24bf1c`.

All counts below pool seed 1, seed 2, `one-user-turn/v1` and `per-turn/v1`:
1,024 row decisions and 16,384 kept positions per read/head cell.

| Read | Head | Kind | Asked first | Other first | Asked-only span | Other-only span | Both spans | Neither |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| depth 2 | 0 | copy | 33 | 32 | 65 | 62 | 897 | 0 |
| depth 2 | 1 | copy | 448 | 426 | 65 | 53 | 906 | 0 |
| depth 2 | 2 | type | 104 | 93 | 16 | 16 | 992 | 0 |
| depth 2 | 3 | type | 166 | 219 | 44 | 52 | 928 | 0 |
| depth 6 | 0 | copy | 148 | 145 | 21 | 21 | 978 | 4 |
| depth 6 | 1 | copy | 466 | 484 | 25 | 26 | 973 | 0 |
| depth 6 | 2 | type | 411 | 411 | 44 | 51 | 929 | 0 |
| depth 6 | 3 | type | 60 | 95 | 73 | 70 | 873 | 8 |

Per-stratum counts are in
[`per-stratum.json`](./evidence/per-stratum.json); exact run hashes are in
[`summary.json`](./evidence/summary.json).

The trace alignment control is exact: each traced `chosen_token` equals the
reviewed `cpu-forced-copy-panel` free arm's first token on all 1,024 rows. That
free arm reproduces `served-reply-panel` generation, with the CPUs' decision
parity retained by deepseek-241. The seed-1 one-user repeat has no non-timing
field difference.

Resources are in
[`resources.json`](./evidence/resources.json): 18 retained reports,
85.871 seconds measured probe time, 51,092 KiB maximum peak RSS,
25,165,982 retained bytes, and a final 32-thread sweep of 0.56-0.66 seconds.
The D84 ledger reconciles the previous 98,567.677487699-second admitted total
plus this unit's 1,800-second projection to 100,367.677487699 seconds.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-first-reply-hamming-trace`: 7 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib`: 455 passed,
  4 ignored.
- `cargo test --release --offline --locked --features cpu-parity --bin
  served-reply-panel`: 15 passed.
- `cargo check --offline --locked -p uor-r4-cpu-train --bin
  cpu-first-reply-hamming-trace`: passed.
- Targeted `rustfmt --check`, driver Clippy, `git diff --check`, report check,
  local policy and full loop gates pass.
- The renderer test pins the complete decoded protocol-v2 prompt for both
  registered panel forms.
- The final v4 reports reproduce every decision, row, span and count field from
  v3 after excluding executable identity, wall time and RSS. The v4 repeat is
  exact apart from timing and RSS.

## Skills used

No system-provided Codex skill applied. The repository's CPU-research and
delivery rules were followed directly.

## Still open

- Add distance and weight contribution to the kept positions. The current
  trace says positions from both spans are retained; it does not say how
  strongly either is used or whether the complete values are available.
- Run the same trace on Claude-323 and Claude-325 integer artifacts when their
  local served files are available.
- Claude-328's trained query-recall probe remains the causal test for whether
  a query-conditioned Hamming read can learn the role choice.
- The free-generation float/integer whole-reply gap remains separate.

## Plain summary

We looked inside the model at the exact moment it chooses the first word of a
reply. Its Hamming memories contain at least one position from both the asked
fact and the other fact in almost every case. Yet every reply starts with
neither fact's value. The trace does not measure weights or causal selection,
so it cannot say which retained position mattered. It does rule out the simple
explanation that no asked-value position reaches this read; the remaining
question is how the read chooses and weights from its retained positions.
