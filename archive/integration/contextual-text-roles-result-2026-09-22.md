# Contextual occurrence roles and exact spans from readable text

> **Principal correction:** This preserved submission account is superseded where noted by the [principal review](contextual-text-roles-review-2026-09-22.md), [original independent audit](../evidence/contextual-text-roles-principal-review-2026-09-22.json) and [corrected replay](../evidence/contextual-text-roles-corrected-replay-audit-2026-09-22.json). Retain24/24 primary answers and genuine multiword output; true initial marker-role count0/48, not14/48. Final panels were exposed. Cue endpoint signatures already distinguish the fixture; membership whitespace, source-edit parsing, later-fact and snapshot retention claims needed correction. The primary is already categorical; next learned argument-span support and surface/identity separation, not a second categorical-only comparator. Original counts below remain historical evidence.

September 22 UTC, 2026. Executes the [contextual text roles brief](deepseek-contextual-text-roles-step-2026-09-22.md)
and the [observed-text review](observed-text-session-review-2026-09-22.md). Base: the **corrected head
`f5919d8e`** of PR #1342, which was still open at recovery; `origin/main` was `d9d896e8` and did **not**
contain the #1341/#1342 repairs, so the corrected head was used directly rather than my original
`b9eca5d2` or an older `main`. Isolation worktree `.worktrees/contextual-text-roles`; owner checkout
untouched. [Evidence](../evidence/contextual-text-roles-2026-09-22.json).

## Why the observation model had to change

The reviewed model scored a marker candidate by summing nonnegative per-token role votes. Adding a
voting token cannot lower that score, so a familiar cue inside a name could extend the selected marker
into the name; the score was a commutative bag sum, so permuting the same marker tokens preserved it;
and one global sign per token could not treat a token as syntax in one occurrence and as name content
in another. Those are representational losses, not optimizer weakness.

## What replaced it

A **candidate-conditioned score over ordered local context**. For every bounded contiguous candidate
the model reads its own first and last tokens, the tokens immediately before and after it, its length,
and whether it is clause-initial or clause-final. Order is therefore encoded (first/last and
left/right slots), and no decision depends on a single global token sign. The extractor
`candidate_features` is the *one* observation path used by fitting and by serving.

Learned from declared development clauses by a bounded candidate perceptron:

| Quantity | Value |
| --- | ---: |
| Whole-clause (marker extent, role) accuracy, declared uninformed start | **14 / 48** |
| Whole-clause accuracy after fitting, same units | **48 / 48** |
| Learned feature weights | 50 |
| Candidate updates | 11 |
| Role-action table, same gold statement units | 14/24 → 24/24 |

## Readable text with shared vocabulary

Real words are encoded by the pinned BPE tokenizer. Cue words are deliberately shared with name
content: `office` and `project` also occur inside the office `Office Park` and the project
`Project Bay`, and `Cedar` is both a person and the first word of the office `Cedar Annex`, so exact
span identity is required to keep `Cedar` and `Cedar Annex` distinct.

Answers are **multi-token BPE spans emitted with their spaces intact**, for example ` Lumen` and
` Cedar Annex`, followed by the declared terminator, through the corrected `ObservedTextRuntime`
(`start` receives the observed question and the runtime alone infers the goal).

## Results

| Arm | development /24 | **fresh final /24** | depths |
| --- | ---: | ---: | ---: |
| **contextual primary** | **24** | **24** | **24** |
| registry-membership continuation | – | **13** | – |
| maximum-two-read | – | **18** | – |
| reads disabled | – | **0** | – |

The same question and entity answer after **one, two and three reads** in different worlds; the
three-read composition is **absent from development supervision**. Development exposes one- and
two-read chains only. The two behavioural controls establish that neither entity membership nor a
fixed depth substitutes for the learned contextual role semantics.

Causal interventions: a one-position source edit changes the selected source occurrence and the
request outcome; changing only the request goal with the same entity selects different evidence;
removing a required later fact gives **unresolved** with no emission; and swapping two tokens of the
observed question cue makes the clause no longer a recognised question, which is the order-sensitivity
witness the previous model could not express. A mid-session snapshot restored through the runtime
continues to the same terminal outcome.

## What actually learned, and what stayed scaffolding

**Learned:** 50 candidate-feature weights, plus per-role goal and action tables, from declared
development clauses; the whole-clause observation accuracy moves 14/48 to 48/48 on the same units.
**Supplied scaffolding, declared:** the readable clause layout, a **sentence-start space** that keeps
every name mid-sentence so subject and object positions share surface BPE tokens, bounded contiguous
span candidates with a maximum marker length of four, the name/cue vocabulary, and the terminator.

The single most consequential diagnostic of this run was a tokenization boundary, not a modelling
choice: without the declared sentence-start space, a clause-initial `Ivo` and a mid-clause ` Ivo` are
**different BPE tokens**, so exact span identity could not join a redirect target to the next clause's
subject and every two- and three-read chain stopped unresolved.

## Not established

The comparator arms are control *switches* on the same model and runtime, not independently fitted
learners; a separately fitted categorical comparator was not run. No geometric arm was added: this
milestone asked for the contextual interface first, and the earlier finding still applies that a fixed
identity or central offset reduces to a paired categorical code. The final panel withholds the
three-read composition and uses new assignments, but shares names, cues and clause forms with
development. No prose, no executed Rust task, no whole-path latency or energy measurement. Energy
`UNAVAILABLE`; whole-path D0-b not claimed.

**Base advanced during the work.** While this ran, `origin/main` moved from `d9d896e8` to `dd4c2751`,
which merged PR #1341. PR #1342 remains open. The single new commit was rebased onto `dd4c2751`; the
merged runner is byte-identical to the #1341 reviewed head, so nothing from main's merge was lost. The
rebase required re-adding the `observed_text_session` module registration (main never had it, since
#1342 is unmerged) and the tree was rebuilt and re-executed after the rebase to confirm an identical
result (24/24, 24/24, 13, 18, 0, 14->48/48). Retained roots `contextual-text-roles-{1..5}` (0 unlisted
each): `-1`/`-2` are pre-rebase diagnostics, `-3` the pre-rebase primary and `-5` the post-rebase rebind
with an identical result.
