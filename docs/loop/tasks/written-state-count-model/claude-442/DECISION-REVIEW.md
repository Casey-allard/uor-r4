# Decision review: claude-442's card on the per-distance Pick split in S5

- **Reviewers:** three fresh-context, read-only reviewer subagents of the claude top session (Opus).
  Each saw the card, its ORDER and the track's sources at exact refs, and never the reasoning behind
  the card. Each stopped at its context budget, so the slices were split and the top session closed
  the rest (below).
- **Read at:** round 1 at card head `52c1228a0`; round 2 (two reviewers, one slice each) at
  `545bac411`. Base `origin/main` `47c0c82c8`.
- **Verdicts:** round 1 APPROVE WITH CHANGES (partial, on what it verified); round 2, code and codex
  slice, APPROVE WITH CHANGES; round 2, charter, plan and numbers slice, APPROVE. Every change is made;
  see "Disposition".

## The reviewers' findings (summarised; severity and evidence as returned)

**Numbers.** Checked with jq against `claude-441/run1/pick-p1000-retry.json`: `band_records` 2,670;
the bands A1s − A1 [−32, −16] (novel [−44, −23]), C3s − C3 [−27, −10], A1s − C3s [−241, −28],
A1 − C3 [−232, −24]; the eight arm scores (A1 1,360,806 / 1,651,188; A1s 1,360,781 / 1,651,154; C3
1,360,932 / 1,651,364; C3s 1,360,913 / 1,651,338); A1s's class targets 5,340 / 4,111,759 / 0 /
36,362 / 6,653,956, summing to the 10,807,417 scored targets; hits 6,160 (16.9%) and 505,294 (7.6%).
The arithmetic (528, 880, 7,392 and 12,320 words, 39,424 bytes, 17 per million, 19,712 bytes) holds.

**Code, at `47c0c82c8`.** `code_pick.rs`: d is the sum of the two pairs' popcounts over all blocks,
and it returns `(pick, best, ties)`. `rank.rs:23-27`: serving uses d only for exact against near.
`frozen_pick.rs:9-10, 122`: magic `UORS5W01`, a 44-byte header with no class count, and
`word_count = bucket_count × PICK_CLASSES × (expert_count + 1)`; `mixture.rs:7-14, 73`:
`expert_count = h + 3`, `bucket_count = (h + 1) × 16`, `PICK_CLASSES = 3`. So 7,392 × 8 + 44 = 59,180
bytes. `pick_split_class` is at `kn_ceiling.rs:179`; `count_runtime.rs:23` and `kn_ceiling.rs:25` both
include `runtime/mixture.rs`. The D85 argument of item 1 holds.

**On the decision.** Both full reviewers would make the same choice: adopting now spends a unit on an
artifact no bar scores (L-008), and "never" gives up a real if tiny gain for 39 KB. No conflict with
§3.1 (the class is a Hamming distance in code space), §3.7 / D85 (one shared class function) or §5.1
(a scope choice, logged for Casey).

| # | Round | Severity | Finding |
|---|---|---|---|
| 1 | 1 | MINOR | The four-class argument had a redundant premise ("no two tokens share a code"); d = 1 is ruled out by no two distinct codes being 1 bit apart alone. |
| 2 | 1 | MINOR | The class counts did not show class 0 (5,340), so they did not visibly sum to the scored targets. |
| 3 | 1 | MINOR | Say that DEV-tune only selects the arm at the refit. |
| 4 | 2a | MINOR | "now in `kn_ceiling.rs`, moved where …" reads as if the move were done. Name the lines and who moves it. |
| 5 | 2b | MINOR | A deferral could lapse unrecorded if the freeze is P1(b)'s last refit. |
| 6 | 2b | MINOR | Item 2 called the fit costs both measured and [est]. |
| 7 | 2b | MINOR | No speed or memory clause, unlike claude-440's rule; say what covers them. |
| 8 | 2b | MINOR | PLAN's sentence omitted class 0. |

## Disposition by the top session

| Finding | Done |
|---|---|
| 1 | The premise is dropped; the card now states d as `code_pick` defines it. (`545bac411`) |
| 2 | Class 0's 5,340 targets are listed, and the five classes are said to sum to 10,807,417. (`545bac411`) |
| 3 | The bits rule says DEV-tune only selects the arm. (`545bac411`) |
| 4 | Item 1 names `kn_ceiling.rs:179`, the two includes and `rank.rs:23-27`, and says the refit unit moves the rule. |
| 5 | The cost rule says the refit unit's REPORT and PLAN's P1 row record a deferral, and what happens if that refit was P1(b)'s last. |
| 6 | Item 2 now reads "measured at C1 (codex-31)", and full C is marked unmeasured. |
| 7 | A line says the refit unit's bar B checks (≥ 1,000 tok/s warm, ≤ 4 GB resident) cover the split. |
| 8 | PLAN's sentence now gives class 0. |

**Gaps closed by the top session**, read at `47c0c82c8`:
- **The 37‰ weight.** From `pick_arms.main.a1s.pick_weights`, weighting each bucket's `weight_q32` by
  its `fit_targets`: class 1 190‰ (6,001,324 fit targets), class 3 37‰ (53,736), class 4 10‰
  (9,884,404). These match the card and claude-441's REPORT.
- **The charter.** §3.1 (`CHARTER.md` line 26), §3.7 (line 35: "Training and serving call the same
  typed operator definitions and the same state-transition path"), and §5.1 item 1 (line 58: a scope
  choice goes ahead on the lab's own decision review, D68).
- **claude-409's bar B.** Its card, lines 47-48: ≥ 1,000 tok/s warm at 2k context on one M1
  performance core; D85 replay bit-identical between wick and the M1 on ≥ 64 prompts.
- **L-008** (`docs/harness/mistakes/L-008.md`): merged PRs that change no model while the milestone
  stands still. The card's "never alone" rests on it.
- **codex's reports.** codex-31's REPORT, line 23: "7,392 Q32 weights in 59,180 bytes", feature
  preparation 270.21 s, GPU optimizer 2.37 s. codex-31's `fit/SHEET.md`, line 11: "the full 528×14 Q32
  state". codex-33's REPORT: 2,316,604,155 model and tokenizer bytes, 192/192 replayed positions, and
  under "Still open" an R-only fit and freeze of the full-C candidate. The card's "next S5 refit" is
  that step.
