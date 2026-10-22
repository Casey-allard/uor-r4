# REPORT: m1-faster-writer, claude-36 (the DRAFT pre-registration sheet for the bucketed CUDA writer)

Lab: claude. Base: `e10d10926`. Order: [ORDER.md](ORDER.md), committed first (`e792b5f1b`). The cancel probe at the base found no writer sheet in `docs/harness/prep/` and no registered writer bar on #2840, so the task stands.

## What did not work, first

- **There is no 16,384-piece vocabulary to qualify on.** The design note proposed reporting at 4,096 and at "the 16,384-piece tokenizer", but no such table or tokenizer exists at the base. The only 16,384 in the crate is a parameter count (`measurement.rs:213`). The sheet therefore proposes a seeded V = 16,384 table and leaves the choice open for the FIT reviewer and the owner.
- **The design note's counts were slightly off.**

  | | Design note | Code at the base |
  |---|---|---|
  | `token_cell` calls per row | V + K·V = 69,632 | V + 1 + A·V = 69,633 at A = K = 16; the extra call is the target's cell, `:311` |
  | Full-scan admission evaluations | K·256, exactly | at most K·256 |
  | "Exactly 256" cell evaluations | per row | only in the admission pass; the rank telemetry evaluates `address_distance` again (`s[11]`) |

  The sheet states the corrected forms and scopes that bar to admission.
- **The B1 output list is longer than the note's.** The `writer` signature also writes `token_hinge`, `address_hinge`, `telemetry[0..6)` and `arithmetic_status`. B1 compares all of them.
- **One sentence in the first draft paired four planted faults with three host results.** It is fixed in `9c1c60a09`.
- **The fresh-context self-review of `771ba584d` returned CHANGES,** with one HIGH and three MEDIUM findings. All are fixed in `15c8a5a4f`:
  - **HIGH: the serving prune could never meet B1.** Skipping a cell skips its rivals, which changes `token_rivals`, the rival distance, the hinge, the loss and `telemetry[3]` (`discrete.cu:320-322`, `:327`, `:342`). No existing entry is prediction-only, and the `>=` planted fault had no path to run on. **The prune is now out of scope,** as a named follow-up with its own prediction-only entry, and its planted fault is replaced by a kind-1 double-count fault.
  - **MEDIUM: `discrete_evaluate` has its own O(V) top-8 scan** (`:479-488`). It is now bucketed too, and B2 counts it.
  - **MEDIUM: the index update cost per accepted move was invisible.** It is now counted and reported beside B2.
  - **MEDIUM: `discrete_export_token_cells` cannot build the index,** because it skips ABSENT tokens. The sheet now requires the module's own count/scan/scatter kernel.
  - **LOW:**
    - MAX rows prefill a shared sentinel;
    - B2's 256 is now "at most 256, one per cell";
    - kinds 0 and 3 are named with their read modes.
- **The round-2 self-review of `15c8a5a4f` confirmed every round-1 fix and returned CHANGES on two MEDIUMs:**
  - **The full-rebuild control had no move sequence to run on.** §5 now runs 64 scripted edits and 64 real loop steps, with a full rebuild compared after each one.
    - This also corrects my first wording of that fix, which had an accepted move making a token ABSENT. `discrete_apply` refuses any proposal that does not lower the loss (status 5), and an ABSENT proposal's loss is MAX, so that move can never be accepted.
  - **The B2 token-distance count was undefined.** It now counts the membership walk only, with the target's distance and the top-8 scan counted separately, and virtual membership under kind 1, as the host counter does.
  - **LOW:** the citation and commit lists below are brought up to date.

  This was the second and last self-review round. The non-author FIT reviewer's record is the next review.

## What changed

| Commit | What |
|---|---|
| `e792b5f1b` | This ORDER, alone. |
| `9098061f8` | `docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md`, drafted by a builder subagent from the design note (#2840, 6346787358) and the host controls (`e10d109`). |
| `9c1c60a09` | §8 names each planted fault's host result. |
| `73443ddeb` | This REPORT, first version. |
| `771ba584d` | The §9 heading says "stop conditions", which is what the rule-sources check wants. |
| `15c8a5a4f` | The round-1 self-review fixes. |
| this commit | The round-2 self-review fixes: §5 move sequences against a full rebuild, the B2 count definitions, and the kind-1 fault's conditions. |

The sheet registers a finite CUDA qualification: no training, no checkpoint. It covers:
- **Kernel:** a CSR bucket index over the 256 root-pair cells, in its own nvcc module (`bucket_source.rs`, a `build.rs` block after bvgk). NVRTC is not used.
- **Panel:** the pinned A8 table (SHA `6600bdb1…`) and seeded tables at V = 4,096 and 16,384, with seeds 1–3, 4,096 seeded reads per table, every tie and ABSENT edge case, and all three kernel entries and all four kinds.
- **Counters:** compiled separately from the measured kernel. The plain and instrumented builds must give identical outputs.
- **Bar, marked "PROPOSED, NOT REGISTERED":**
  - B1: 0 output mismatches against the full scan;
  - B2, counted over `writer` and the evaluate entry's top-8 scan: 0 `token_cell` calls (≤1 for kind 1), at most 256 admission evaluations (one per cell), and token distances equal to the admitted members (no prune).
  - Writer time per row is reported, not a bar.
- **Four planted faults, stop conditions and a cost** of under $1 as a planning figure.

## Measurements

None. This is a sheet: nothing runs until FIT and Casey's registration of the bar.

## Checks run at the head

- **Citations.** Every `discrete.cu` line the sheet cites was checked at the base by the builder and the two self-review rounds, as were the pinned SHAs at `attention_learning.rs:31-33`. The lines are `:286-290`, `:301-303`, `:307-345`, `:309`, `:311`, `:319-322`, `:326`, `:327`, `:330-333`, `:342`, `:367-372`, `:370`, `:380`, `:388`, `:394`, `:400`, `:431`, `:474`, `:479-490` and `:483`.
- **Not run here:** the 6 host tests. The "4 of 6" figure for the flipped tie rule comes from my run at #2983 (DeepSeek's review there counted 3 of 6 with its own mutation).
- Claim-wording on the sheet: passed, with 0 banned-function names.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one new sheet and this task folder. No code.

## Still open

- **The non-author TEST FITNESS review,** requested on #2028.
- **Casey's registration of the bar** after the FIT: his words and the time go on #2840.
- **The two open choices,** for the reviewer and the owner:
  - the seeded or a real 16,384 table;
  - whether to include the real-read DEV panel.
- **Unit 3:** the CUDA module and the qualification run.
- **Later:**
  - whether the rows, rowsk and bvgk modules switch to the bucketed writer;
  - the serving prune, with its own prediction-only entry.

## Plain summary

This writes down, before any GPU time is spent, exactly how the faster word-picker will be tested: it must pick the very same word as today's version on every test row, while doing a fixed, small amount of work per row instead of scanning the whole vocabulary.
