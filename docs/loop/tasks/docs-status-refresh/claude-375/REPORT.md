# REPORT: the mechanism board and pipeline map brought to 14 Nov 2026, claude-375

Lab: claude. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md) (#3380). Order: [ORDER.md](./ORDER.md). Base: `085ff55d92c31ded738913ae8907251e4dcd8a21`.

Every claim below is [claude]'s.

## What did not work, or is not done

- **The board's statuses and the code disagree in four places, and this unit does not fix them.** A status change is a decision card (CHARTER §5.1, §5.2). The four are listed under "Still open". The board now says so in its own words, in a new section, rather than changing a row.
- **The pipeline map's stage tables are still the 31 Oct revision.** This unit adds a dated "what is different on 14 Nov" table and the units claude-345 to claude-374, and a note above them. It does not rewrite the stage tables cell by cell. The units claude-190 to claude-344 are still not on the page; the map says so and points to the notebook.
- **Two open codex drafts edit the same files.** #3967 and #3968 both change `AGENTS.md`, `CHARTER.md`, `PLAN.md` and `docs/model/pipeline-map.md`, among about forty files each. This unit's edits sit in other lines, so whichever PR merges second merges forward. A textual conflict is still possible if their drafts move.
- **The audit's first draft of the board said "Flock is in no current recipe".** That is true of the FBS recipe only: the saved discrete M1 model reads with `Flock` (`FrozenZeroV1`). The board says "no current FBS recipe".
- **AGENTS.md is unchanged.** Its stale-fact check found none: "1 of 8 milestones … is complete" matches `docs/harness/milestones.json` (M1 complete; M2, M3 and M7 in progress; the rest not started). It is an owner path, so no edit was made for style.
- **The V0–V5 roadmap of the audit is not written anywhere.** It is Casey's open decision.
- **Three slips were fixed before the review was recorded.** The fresh-context reviewer found the first; [claude] found the other two while checking the figures it left unverified. The first draft edited the README attention row's status cell ("ACTIVE on the discrete M1 path ..."), a scope change by wording; the cell is restored and the fact moved to a sentence under the table. The board listed `geometric_stack_native.rs` under "connected to neither the trainer nor the integer server", but it is a trainer file with an inactive part; the heading and item now say so. The REPORT named "open issues #2030 to #2032" as milestone evidence; that clause was removed.

## What changed

- **`docs/mechanisms/README.md`:**
  - the opening says the statuses were last set on 31 Oct and points to a new section;
  - new section "Since 31 Oct: what the current model trains and serves (14 Nov 2026)", with three parts:
    - claude-358's `p` recipe, flag for flag from the [pair-rows sheet](../../../../harness/prep/one-model-end-to-end-pair-rows-2026-11-13.md), mapped to board rows, and what is not in it (Flock in no FBS recipe; Route, Relation, prime-zeta, address cells and discrete learning on the discrete path; the address memory and pair rows as flags only; no state carried across training windows);
    - a table of the new forms with no board row: the E8 address memory (claude-346, claude-366), the pair experts (claude-367, claude-368) and the pair rows (claude-370 Adds; claude-373 and claude-374 Do not add), each with whether it is served;
    - code built but off the trained and served path: nine attention modules and two zeta modules in `uor-r4-core`, `helm_d_r4_attention.rs` named as a softmax oracle, named by neither `uor-r4-float-train` nor `uor-r4-integer`; and the inactive part of the trainer's own `geometric_stack_native.rs`, whose Lorentz Starling read the trainer refuses.
- **`docs/model/pipeline-map.md`:** a dated update note under the introduction, and a section "Units since 31 Oct" with a stage-by-stage table of what changed (Hamming reads served; turn memory served; the opt-in pair-expert override in `uor-chat`; nearest served with near-tie differences; text served through `uor-chat`; no carry across training windows; `lut-stack/10` bound, `/11` research only; the export refuses pair rows; claude-358's speeds) and one row per unit from claude-345 to claude-374.
- **`README.md`:**
  - the human-text question now records claude-301 (ahead of T2 at both seeds, 6 Nov) and claude-317 (width 256 over 128, 8 Nov). The old text said the model "does not yet beat the T2 baseline" and named the batch-64 run as next;
  - a second open question: whether the model answers in a conversation, with claude-358's recall counts and the pair-expert override named as a table, not the model;
  - "Where we're going" names that second question as the immediate work, in place of the human-text bar;
  - the product line names `lut-stack/10` beside the historical `/5`;
  - one sentence under the parts table says the current FBS models read with the Hamming read (Ternary/XOR); the attention row's status cell is unchanged;
  - the acknowledgements say Codex rejoined the registry on 12 Nov (#3942).
- **Redirect-stub targets.** The four stubs stay redirects. Their targets gain the content of the archived brief they lacked, each with a link to it:
  - `flock-read.md`: the rank-table form of `flock-rank-reads` (sink, window, top-K, rank weights, at most W+K+1 rows, its test caveats, failure condition and open questions). It already held the `starling-read` brief's fair-test list.
  - `route-read.md`: the `invariant-addressed-read` brief's motivation, cost share, spanning-tree argument and the two nulls.
  - `exact-memory.md`: the `exact-addressed-memory` brief's M1 pointer negatives (#2127, #2128, #2133, #2161), direction through the expert, failure conditions and one open question.
  - Where a brief framed its bar against a softmax read, the page says the owner has since withdrawn those comparators (L-006).
- **`docs/roadmap/items/docs-status-refresh.md`:** this unit's task folder added to the item's scope, nothing else.

## Measurements

None. This is a documentation unit with no model run. Every number it writes is copied from a named REPORT or notebook row; every code location was read at the base:
- `uor-r4.lut-stack/10` and `/11`: `STACK_FBS_BOUND_SCHEMA` and `STACK_FBS_ADDRESS_SCHEMA`, `crates/uor-r4-integer/src/stack/format.rs`;
- the pair-expert override: the pair check after `serve_reply`, `crates/uor-r4-integer/src/stack/chat.rs` (about lines 497 to 535), opt-in through `uor-chat --pair-experts`;
- `PastWindowRule::Carry`, `crates/uor-r4-float-train/src/in_space_long.rs` (about line 7124);
- the pair-row refusal: `a_checkpoint_with_pair_rows_is_refused`, `crates/uor-r4-float-train/src/fbs_export.rs`;
- the trainer's refusal of a non-`Dot` read: `in_space_long.rs` (about line 1093);
- the unused modules: no file under `crates/uor-r4-float-train` or `crates/uor-r4-integer` names any of the twelve `uor-r4-core` modules (grep by module name, all twelve 0);
- `stack_whole_table_nearest` (`-L1` at Q12), `crates/uor-r4-integer/src/stack/session.rs` line 3968.

## Checks run at the head

- `cargo run -q --locked -p xtask -- links`: 7,504 sources, 1,288 broken repo-wide, none with a source in a file this unit changed (filtered from `links --json` by source path).
- `cargo run -q --locked -p uor-r4-claim-wording -- .`: passed.
- `uor-harness check`: exit 0 (warnings only, all branch hygiene unrelated to this diff).
- Fresh-context review (a `delegate:reviewer` given only the task, the ORDER and the diff) at `fbb7b35a6`: REQUEST CHANGES, one medium and two low findings. The medium one is fixed; the two lows checked clean (the six remaining module names: 0 references each; #3407 is the one-model-end-to-end item the replies work belongs to). The review record at the final head is on the PR.

## Skills used

None.

## Still open

Decision cards this unit would propose (none is made here):
1. **Ternary/XOR (READY).** Its Hamming read form is trained and served in claude-358's p models. Either a separate "Hamming read" row, or moving the row, needs a decision row: D74 set it READY.
2. **Exact memory (PARKED).** Its turn-memory form is trained and served in the p models. A separate ACTIVE row for exact turn memory would match the code.
3. **New rows** for the E8 address memory (claude-346 not called, claude-366 inconclusive), the pair experts (claude-367 adds; counts, not training) and the pair rows (claude-370 adds; not served). Their statuses are a decision; the pair experts may not count as a mechanism at all under CHARTER §3.1, since they replace a reply with a stored TRAIN reply.
4. **The D11 integer serving and SpiralCore/FBS rows** describe the 29 Oct FBS export under `lut-stack/5` and give no current form; new exports are `lut-stack/10`. That is content, not status; it is left for a refresh of those rows.

Other open work:
- The pipeline map's stage tables, cell by cell, and the units claude-190 to claude-344.
- `docs/start/where-we-are.md` and `docs/research/findings.md` were not read in this unit and may carry the same stale human-text sentence the README had.
- The family-tree figure and the mermaid tree carry no row for the new forms.

## Plain summary

The mechanism board, the pipeline map and the README now say what the model trains and serves at `085ff55d9`:
- which mechanisms are in the current recipe and which are only flags or unused modules;
- the three new forms since 31 Oct and their results;
- that the chat program can swap a reply for a stored training reply;
- that the model beat the T2 table on human text on 6 Nov.

No status changed. Four status questions are listed above for a decision.

References #3380.
