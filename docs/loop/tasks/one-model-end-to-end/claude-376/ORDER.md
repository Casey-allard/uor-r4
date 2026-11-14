# ORDER: one-model-end-to-end, claude-376 (V0: the pure-vector baseline)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `085ff55d92c31ded738913ae8907251e4dcd8a21`. Follows claude-374 (PR #3965) and the 14 Nov audit (`local/claude/one-vector-roadmap-2026-11-14.md`).
Renumbered from claude-375 on 14 Nov, while the reads ran (seed 1's exact counts had been looked at; this unit has no bar, and nothing in it changed): the docs session's PR #3969 had taken that number first. The kit folder is `local/claude/claude-376/`.

## Goal

**Measure how much of the served model's replies come from the vector state alone. Serve claude-358's three saved `p` models on the laptop with the artifact's exact turn memory switched off and on, on the same panels. Time both.**

**Why now.** Casey, 14 Nov 2026 (verbatim): "I want to focus on training directly into vector space, using vector space for the whole mechanism end to end and serving from vector space, ideally". He then said "Yes, start V0".

The audit found two parts of today's chat that are not the vector state:
- the turn memory, an exact table inside the artifact;
- the pair experts, which replace a reply with a stored TRAIN reply.

Every later stage of the roadmap (V1–V5) is read against the pure-vector number this unit writes down.

**What is already measured and is not re-run.** claude-367 (PR #3958) read these three models with and without the pair experts, on a 239-row everyday panel. Good replies were:
- 137, 135 and 134 off;
- 170, 171 and 170 with the experts held out;
- 191, 193 and 194 as served.

The slowest turn took 0.07 s with the experts against 0.014 s without. Its "off" setting kept the turn memory, the counted asks table and serve-again on. Rebuilding that panel and its counted-table chain (claude-359, claude-362, claude-365) is not needed to answer this unit's question. All the inputs to that chain are gone from the laptop, and only the scripts remain.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a read of these models with no turn memory and no pair experts, already made.
- **Result (at base):**
  - claude-367 read them with the turn memory always on.
  - claude-365 and claude-366 read with the counted asks table.
  - `uor-chat --no-turn-memory` exists (`crates/uor-r4-integer/src/bin/uor-chat.rs:2305`), but no panel was read with it. `raw-id-replies` has no such switch.
- **What earlier work warns of:**
  - The 9 Nov memory-line gap (a7r): TRAIN had recall off, while serving wrote the memory line.
  - So the turn memory may answer most of the told-value rows. With it off, those rows may fall a long way. That is the number this unit is for, not a failure.

## Claim

Written by hand, as claude-359 to claude-374 were. No code of this unit exists at this order's commit.

## The settings

Every model is served by `raw-id-replies` at this unit's head, greedy, 48 tokens, `--mark-reply 1`. No counted asks table, no serve-again and no pair experts are used in either setting.

- **V (pure vector):** the artifact with its turn memory dropped (`IntegerStackModel::drop_turn_memory`, the call `uor-chat --no-turn-memory` makes). What runs is the token table, the FBS/SpiralCore recurrence, the Hamming reads with the artifact's copy rule, and the whole-table nearest readout.
- **VM:** the artifact as exported, with its turn memory on.

## Inputs

- **Models:** claude-358's `p-s1`, `p-s2` and `p-s3`.
  - They are fetched from the store at `results/claude/one-model-end-to-end-copy-longest-2026-11-12/run/p-sN/try-1/` and checked against their SHA256SUMS.
  - They are exported by `fbs-export` at this unit's head, as `job-358.sh` did: `--legacy-artifact --raw-gates --raw-transition --run-config <run>/snap/config.json`.
  - Each export prints its artifact's SHA-256, and the REPORT records all three.
- **Panels:** P14 (1,108 rows) and P23 (1,110 rows), the prompts and references claude-370, claude-373 and claude-374 scored. They come from claude-374's fetched `pre` stage (`samples-p-greedy-memory.json`, p14 and p23). The REPORT records their identity hash, the same `[prompt, reference, position]` hash read-374.sh checks.
- **Scorer:** claude-374's `read-374.sh` row rule and `subst-374.sh` (SHA-256 `8384c472…b791b691`) for the fact rows.

## The one change in scope

`docs/labs/one-model-end-to-end/instrument/src/bin/raw-id-replies.rs` gains `--turn-memory 0|1` (default 1, today's behaviour). `0` calls `drop_turn_memory` before serving and refuses an artifact that has no turn memory. Tests:
- the default leaves every reply id as before;
- `0` on a synthetic artifact with a turn memory changes the served ids where the memory answers;
- `0` on an artifact with none is refused.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/src/bin/raw-id-replies.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-376/**`
- `docs/research/notebook/float-training.md`

## Done when

1. `--turn-memory 0|1` exists, with the three tests above, and the instrument crate's tests pass.
2. For each model and setting, the REPORT gives:
   - exact replies over P14 + P23 (of 2,218);
   - the fact and number rows, exact and ended with the answer word (of 188);
   - replies that did not end;
   - served tokens a second, timed with both settings of one model run one after the other on the laptop (`--time-steps 1`).
3. The REPORT puts these beside claude-367's pair-expert numbers. It states plainly how much of the served replies the vector state gives alone, at the scope of these panels. Replies are quoted word for word.

## Bars

None. This is a measurement for the roadmap's later stages to read against. It has no mechanism claim, and its notebook verdict word is **inconclusive** by the format page's meaning: "a number, but no bar was decided". No setting or flag is chosen from its result.

## Stop rules

- If the exported artifact's turn memory cannot be dropped, or the default setting does not reproduce the artifact's own served ids, that is an instrument failure. It is repaired, never read.
- If the panels' identity hash differs from claude-374's, the read stops.

## Rules check (CHARTER §3)

- No training, no pod, no GPU.
- Serving the exported integer artifact on the laptop is the product path (§3.6), as claude-365 to claude-367 did.
- The one code change is a serving switch in the instrument crate.
