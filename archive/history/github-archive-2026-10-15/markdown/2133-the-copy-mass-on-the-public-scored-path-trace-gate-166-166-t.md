# #2133 — The copy mass on the public scored path: trace gate 166/166, the frame carries the mass (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T02:56:27Z
- **Updated:** 2026-10-10T03:00:28Z
- **Closed:** 2026-10-10T02:58:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2133
- **Merged:** 2026-10-10T02:58:14Z
- **Merge commit:** bf70a555cfe8fecf3c65732ed15ac5be96c09b6e
- **Base / Head:** main / UOR-Foundation:deepseek/pcopy-mass-read-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The reading the training plan's step 1 asked for, taken with **a caller, not an accessor**. **References #2029.** CPU only, no pod, no training, no knob, no model saved, no v5 re-run, $0.

## What was built

`StackModel::score_targets` already returns `TargetScores.pointer` → `PointerRowStats { gate, copy_mass, hit, reachable }` (`copy_mass` documented as `p_copy(target|t)` before the gate) and is **already called on the reply path** (`stack_dialogue.rs` `development`). So this needed no model code: `crates/uor-r4-training/examples/pcopy-mass-read.rs` rebuilds each v5 memory row's sequence exactly as the graded reply ran it — `history_ids` ++ the sealed last turn's reply ids, **nothing generated** — and reads `copy_mass` per target and position (the value's ids, the distractor's, the 9 decoded frame ids).

## The gate, pre-registered, and it passes

> on the natural path (target = the sealed next token) `hit` must equal the recorded trace's `matches_source` at every traced step; any mismatch makes the run VOID and reports no panel number

**13 rows, 166 traced steps, 0 mismatches** — the public scored path is the decoder's own mixture. The program enforces it (writes the mismatches, prints VOID, exits 3).

## The measurement, per row

| | numeric 13 | word control 27 |
|---|---|---|
| rows with the value's copy-mass share above the frame at its best reply step | **0 of 13** | 21 of 27 |
| rows carrying the value (share ≥ 0.40 at a reply step) | **3 of 13**: `mem-008` 0.897, `mem-024` 0.717, `mem-040` 0.601 | 16 of 27 |
| rows where the trace selects a stored digit | **3 of 13** — the same three | — |
| rows emitting the value | 1 of 13 (`mem-040`) | 9 of 27 |

On the other 10 numeric rows the value's share is ≤ 0.20 at every reply step while the frame's share is **0.34–1.00**. The copy mass follows the attention exactly, and the frame owns it.

**It lands between the first two pre-registered outcomes**, split along the already-measured selection line: on 10 of 13 rows the read does not carry the value; on the 3 where it does, the decoder still drops the digit twice out of three. The high mass at the value's *first* occurrence (`hit` true, 11 of 13 rows) collapses at its last occurrence and at the reply steps on the same rows — so "high at the digits" is position-specific, not a property of the read.

## Two by-products

- **The published bound is not a bound.** `p_copy(digit) ≤ attention at the argmax` is violated on 5 of 166 steps (one each on `mem-013`, `mem-022`, `mem-024`, `mem-038`, `mem-040`). The correct ceiling `p_copy(target) ≤ 1 − attention(argmax)` holds with **0 violations** and is **loose** (largest use 0.52).
- **The plan's decision rule needs its position named.** "If `p_copy` is high there, nothing to train" is satisfied at the value's first occurrence and not at the reply step that produces the answer; with the position named, its own test says the read is still the constraint on 10 of 13 rows while the 3 selection-positive rows name the emitter.

## Delivery

- record `docs/labs/pcopy-mass-read-2026-10-09/README.md`; evidence `docs/evidence/pcopy_mass_read_2026-10-09.txt` (identities, gate, Table A per position, Table B word control, Table C bound, verbatim output)
- newest-first `current-state.md` entry ending in **Next:**
- stale-doc fixes: dated notes on `pcopy-accessor-handover` (the Site-2 accessor was **not** needed for this reading), `pcopy-read-attempt` ("cannot be taken yet" is closed), `pointer-target-change-plan` (its rule executed; needs the position named)
- run outputs + scripts bundled at `icloud:UOR-R4/results/deepseek/pcopy-mass-read-2026-10-09.tar` (6,560,256 B, md5 `4c79f7ea…`, fetched back and MD5-verified, `all.json` `e3ea8b69…` on both sides)
- `python3 scripts/check_claim_wording.py` passes; `rustfmt --check` clean; the formatted source rebuilds to **byte-identical** JSON
- **STATUS/ROADMAP/#2028 unchanged**: no capability, served model or milestone moved; criterion 1 remains NOT MET and 43/232 is unchanged

**Limitations, stated in the record:** 13 rows / one artifact / control arm at default knobs; the copy distribution is over token ids, not positions, so the value's own occurrence cannot be separated from another occurrence of the same id; the word control is directional (multi-token spans; only `Polly` is single-token); only the 13 numeric rows carry a per-step trace; shares sum over distinct ids and are not a partition of the copy distribution.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

