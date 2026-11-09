# REPORT: one-model-end-to-end, claude-331

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-carry-hold-2026-11-09.md](../../../../harness/prep/one-model-end-to-end-carry-hold-2026-11-09.md).

## What did not work, first

- **The sheet is NOT FIT after two rounds, so this unit stops at the sheet.** That is the ORDER's stop rule (step 2). No code was written and no GPU was used.
- **Round 1** (fresh non-author reviewer, at `35776f884`): no blocker, three majors.
  - **Major 1:** the value rank at the `Q` row was read on the whole stream. The stream holds `Q`'s own point plus two layers, and `w_y` starts at 0 (`flock_read_hamming.rs:1226`). So a right payload could read as near chance, and outcome 4 would have blamed the combine.
  - **Major 2:** nothing was read at `H4` between the hold and the trunk. So a hold that never grew would have been read as the trunk losing the value. `−aS` also removes the direct write, so it cannot attribute a result to the hold alone.
  - **Major 3:** the reproduction check I-R2 only compared B2-aq's hits, which sit at chance. A broken question placement would still have matched.
  - **Four minors:** the hold starts with no gradient; the `g` reading left out the value tokens; the refusals need a new clause; I-1 needed a no-read digest behind the reuse of claude-330's B1-n.
- **Round 2** (a different fresh reviewer, at `6aec906df`): every round 1 major is confirmed fixed in the text. Two new majors:
  - **Major A:** the write's rank, (r1u), divided `u` by one scalar `|w_y|_mean`. The stream write scales each lane by its own weight (`flock_read_hamming.rs:1717–1737`). With mixed signs that loses no information, yet the rank against the raw table points can sit near chance. That would have been an instrument reading presented as a mechanism result (L-007).
  - **Major B:** outcome 4's third branch read the whole stream at `H4`, which `H4`'s own point (the same for every query) can swamp. That is the masking round 1's Major 1 removed at the `Q` row.
  - **Two minors:** I-R2 did not name the trace's fields; the cost left out the cut and rank passes.
- **The round 2 fixes are applied but not reviewed.** Under the stop rule they are not accepted on the author's word. A fresh FIT of the fixed text is owed.

## What worked

- **The code map located the carry's only route** (a read-only scout, its findings checked by both reviewers):
  - The readout read runs after every trunk layer (`geometric_stack_native.rs:1728–1743`). So the readout read at the question row can never reach the scored row `H4`.
  - The only route from `Q(K_j)` to `H4` is the depth-2 stream read's ask write at the `Q` row, carried by trunk layers 3 to 6 (`geometric_stack_native.rs:1684–1701`; width-4 time convolution and the quaternion scan).
  - That narrows claude-330's open split to one write and four layers.
- **The lever survived both rounds as a design.** Neither reviewer found a problem with the held write itself: a per-token scalar `g`, starting at 0, over `h = 8` rows of the same window. They found no breach of §3.2 or §3.3.
- **The readings now walk the value's path.** The ranks go payload → written value → held term at `H4` → `H4`'s stream → final point, each against the right reference. The first step where the value is lost names the next lever.

## What changed

- **The prep sheet** `docs/harness/prep/one-model-end-to-end-carry-hold-2026-11-09.md`. It covers:
  - the held ask write (`--hamming-hold 8`);
  - the cuts `−aS` and `−H`;
  - six value-rank readings;
  - the arms B0-a, B1-a, B2-aq, B2-aqH, B2-aH, with B0-n and B2-n;
  - the bar, with I-R1 and I-R2 reproducing claude-330, and the outcome map;
  - the instrument checks I-1, I-H1 to I-H5, I-V, I-4, I-6 and I-7;
  - the cost, about $1 for two seeds.
- The sheet's FIT line records both rounds.
- The ORDER and this REPORT.
- No code, bar, status, scope or cap changed. No notebook row is written, because there is no result.

## Measurements

None. This unit ran no training or evaluation. The facts the sheet rests on were read from the code at `35f55d334` and from claude-330's sealed seed-1 `B2-aq-s1/recall.jsonl`. Round 2 confirmed that the trace's `readings.trace[]` question entries hold `kept_ask` 392 and 376 (depth 2, head 2) and 512 and 512 (head 3).

## Checks run at the head

- The banned-token scan of the sheet, the ORDER and this REPORT (the §3.2 and §3.3 names): 0 hits.
- `uor-harness loop report --check --order docs/loop/tasks/one-model-end-to-end/claude-331/ORDER.md`: PASS.
- No cargo check was needed: the diff is docs only.

## Skills used

None beyond the loop's own steps. The code map ran as a read-only scout subagent. The two FIT reviews ran as fresh read-only reviewer subagents, each given only the sheet, the ORDER, the prior unit's sheet and REPORT, and the code.

## Still open

- **claude-332:** a fresh non-author FIT of the sheet as fixed after round 2. If FIT, the code follows behind `--hamming-hold`, default off, with I-1, I-H1 to I-H5 and I-V as unit tests with mutations. Then the D67 pre-run review and the two-seed run, about $1.
- **The build map is already read** (the scout's, with file and line):
  - the hold goes in the stream read's combine, `flock_read_hamming.rs` between `:1734` and `:1735`, as a shifted causal sum (the zero-pad pattern at `geometric_stack_native.rs:1991–2003`);
  - `g` saves with the reads (`flock_read_hamming.rs:1088–1100`);
  - `−aS` is a new `Ablation` with `classes() = [[false, true], [false, false]]`, chained after the existing readings like `−aQ` (`in_space_long.rs:12547`);
  - the value ranks need the depth-2 stream at the `Q` and `H4` rows, kept for readings only.

## Plain summary

claude-330's read finds the right fact when the question is asked, but the fact never reaches the answer four words later. This unit traced the code and found the only road between the two, then wrote the test of a fix: hold the fact the read found for the next few words, with a learned weight per word. Two independent reviews found measurement flaws that could have blamed the wrong part of the model, and all of them are fixed in the text. The rules allow two review rounds per unit, so the next unit starts with a fresh review and then builds and runs it, for about $1.
