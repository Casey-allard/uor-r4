# REPORT: one-model-end-to-end, claude-332

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-carry-hold-2026-11-09.md](../../../../harness/prep/one-model-end-to-end-carry-hold-2026-11-09.md).

## What did not work, first

- **The sheet is NOT_FIT after this unit's two fresh rounds, so the unit stops at the sheet** (its ORDER, step 2). No code was written and no GPU was used.
- **Round 1** (a fresh non-author reviewer, at `126ab2256`, after claude-331's review findings were applied): one major.
  - (r1y) ranked the ask payload by raw distance. The payload is weighted by `max(0, m − d) / max(m, 1)` and divided by a count (`flock_read_hamming.rs:1643–1656`, `1713–1715`). So it shrinks toward 0, and a right payload could read as a payload fault.
  - This is the third reading in this sheet's history caught on scale. (r1u) was caught in claude-331's round 2, and (r1hs) in that unit's exact-head review.
  - Three minors: I-H1 named neither the hold's place against the `q16` rounding nor what it compares; `y` was called a plain mean; and (r1hs)'s swamping branch did not say it reads swamping in projection.
- **Round 2** (a different fresh reviewer, at `e20d6a47f`): two majors.
  - **Rows with no payload:** where no ask head found a candidate, `y = u = 0`, every scaled distance is 0, and the rank comes from the tie-break alone. With claude-330's 376–512 of 512 rows kept, a right payload could have averaged near rank 67. Outcome 4 would then have called it "any other pattern", an instrument artifact read as a mechanism result (L-007).
  - **No exit from the extension:** after extending an arm to 16,384 steps, outcome 1 could match again.
  - Two minors (I-H3 cannot pass from the zero start; I-H1's "every gradient") and one nit (`max(m, 1)`).
- **Round 2's fixes are applied in the text, but not reviewed.** Round 2 also ran out of budget before it checked six code facts, listed in the sheet's FIT block.

## What worked

- **The FIT rounds checked the sheet's code facts at the head**, and found them true:
  - the slot layout `Q H1 H2 H3 H4 v`, with header `[48, 49, 50, 51]` and `H4` as the scored row (`in_space_long.rs:11978`, `12118–12131`);
  - the trunk's causal width-4 convolution and scan (`geometric_stack_native.rs:173`, `1984–1998`);
  - the payload and combine (`flock_read_hamming.rs:1640–1740`);
  - the export's existing refusals (`fbs_export.rs:402–405`, `527–529`).
- **The design held.** None of four FIT rounds and one exact-head review faulted the lever itself: a per-token scalar `g`, starting at 0, over `h = 8` rows. None found a breach of §3.2 or §3.3. Every finding was about a reading, a check or the outcome map.
- **Narrower findings per round, though not fewer.** The majors per review were three (claude-331 round 1), two (round 2), one (its exact-head review), one (this unit's round 1) and two (round 2). The later ones were about single readings and one outcome-map exit, not the lever or the arms.

## What changed

- **The prep sheet** now carries:
  - claude-331's review findings (the (r1hs) reference, the export refusal, outcome 1 against outcome 2, outcome 4's last step, the read-weight figure);
  - round 1's fixes ((r1y) at the best non-negative scale, I-H1's rounding and comparison, the payload's description, (r1hs) read in projection);
  - round 2's fixes (ranks over the found rows only, beside the found fraction; one extension with an exit; I-H3 at a hand-set start; I-H1's shared gradients; the weight formula);
  - I-V cases for a zero vector, a `0.3 · p_v` payload and a mixed-sign `w_y`;
  - a FIT line recording both rounds.
- The ORDER and this REPORT.
- No code, bar, status, scope or cap changed. No notebook row, since there is no result.

## Measurements

None. No training or evaluation ran.

## Checks run at the head

- The banned-token scan of the sheet, the ORDER and this REPORT (the §3.2 and §3.3 names): 0 hits.
- `cargo test -p uor-r4-harness --lib -- prep::tests preprules::tests`, the sheet's parser and shape rules: 30 passed, 0 failed.
- `uor-harness loop report --check --order docs/loop/tasks/one-model-end-to-end/claude-332/ORDER.md`, and `uor-harness loop pr-check --local`: recorded in the PR's review.

## Skills used

None beyond the loop's own steps. Each FIT round ran as a fresh, read-only reviewer subagent, given the sheet, claude-330's sheet and REPORT, and the code.

## Still open

- **The lab decision for claude-333 (this unit's bound is spent; the line goes on under this decision, logged here for Casey).**
  - **Build the code first,** behind `--hamming-hold` and the readings flags, default off. That covers the hold, `−aS`, `−H`, the six value ranks over the found rows, and the found fraction. I-1, I-H1 to I-H5 and I-V are unit tests, each with a mutation that makes it fail. Builds and finite tests need no FIT (§4.5).
  - **Then a fresh FIT reads the sheet and the code together.** Of the six majors since claude-331's round 2, five were a reading's scale, rows or reference: (r1u), the whole-stream read at `H4`, (r1hs), (r1y), and the empty rows. Text left these open, and code that computes them settles them exactly. The sixth, the extension's exit, was outcome-map logic, which code does not settle; the FIT still has to read it. A FIT of a sheet with its tests already written checks the readings as they are computed.
  - **Then the D67 pre-run review** and the two-seed run, about $1.
  - **The cost of this choice:** if the FIT then changes a reading, the code changes with it, in the same unit. Nothing runs on a GPU before FIT.
- **First, the facts round 2 did not reach:**
  - `sources[1]` is the raw table;
  - the nearest readout compares the final point against raw `p_v`;
  - `rate_select` reads `RATE_TABLE`;
  - I-R2's sealed numbers against claude-330's `B2-aq-s1/recall.jsonl`.
- **The build map** (claude-331's scout, unchanged):
  - the hold goes in the stream read's combine, between `flock_read_hamming.rs:1734` and `:1735`, before the `q16` at `geometric_stack_native.rs:1693`;
  - `g` saves with the reads (`flock_read_hamming.rs:1088–1100`);
  - `−aS` is a new `Ablation` with `classes() = [[false, true], [false, false]]`.

## Plain summary

The fix being tested, a learned hold that carries the fact the read finds at the question to the answer four words later, came through every review unchanged. What the reviews kept finding were ways the measurements could misread it: a fact shrunk in size, a fact flipped in sign, rows where nothing was found at all. Each such misreading could have blamed the wrong part of the model. All of them are now fixed in the text. The rules allow two review rounds per unit, and the next unit changes the order: it writes the code and its tests first, then has the sheet reviewed with the code in hand, and only then runs it, for about $1.
