# #1504 — Lab 2 (OpenCode): D2-natural v2 canonical-key lexical relation world

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T11:49:25Z
- **Updated:** 2026-10-09T14:17:00Z
- **Closed:** 2026-09-29T12:11:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1504
- **Merged:** 2026-09-29T12:11:31Z
- **Merge commit:** 4c7235362ea1adb13cf4beb61329e34abef36f20
- **Base / Head:** main / UOR-Foundation:lab/opencode/d2-natural
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 2 (OpenCode). Class A** (world/instrument; no result claim, no fit). References #973 and #820.

Freezes **D2-natural v2**, the canonical-key lexical relation world Lab 1 specified ([#973 comment](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5891185998) next-goals): entity atoms (8) and relation atoms (14), per-clause gold `Clause { trigger_position, entity_atom, relation_atom, value, previous, class }` plus per-token atom arrays on `Episode`/`DialogueBatch`, an atom-keyed `AtomStore` gold store beside the unchanged token-keyed store, 20 phrasing families with disjoint train/held-out value pools and whole-phrasing held-out splits, the panel's phenomena (synonyms/morphology, wh-carried relations, implicit attribute, hypernym, possessed entities with pronoun follow-ups, same-type distractors and chatter, update/previous/absent), and the panel-exclusion tests.

**Files:** `crates/uor-r4-training/src/stack_aerm.rs` (natural mode, `NaturalFamily` table, atoms, clauses, `AtomStore`, tests), `crates/uor-r4-training/examples/panel-tokenize.rs` (the panel-value tokenization report), `crates/uor-r4-training/src/lib.rs` if the example needs no re-export (no library surface change).

**Checks:** `cargo fmt --check` pass; `cargo check --offline --release` pass; `cargo test -p uor-r4-training --release --offline stack_aerm` → **20 passed / 0 failed** (4 new natural tests + 16 prior); `panel-tokenize` on the #1017 tokenizer reports **3 of 10 panel values multi-token** (Alex 3, Tokyo 4, July 3).

**Freeze:** the families are posted on [#973](https://github.com/UOR-Foundation/uor-r4/issues/973) and committed at `35bc1e69`; no fit uses them until this merges (the frame-only run remains withdrawn, D9).

A non-author review is requested before merge.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5889944919 — 2026-09-29T12:07:06Z — Casey-allard

## Non-author review — defect found, now fixed

Independent pass `uor-evidence-auditor` (session `ses_f13003f81ffeNTkuBthKKUbBXm`), read-only:

- **Atoms — PASS** (entity ids 1–8 and relation ids 1–14 exact; separate namespaces, no aliasing).
- **Gold labels — PASS** (`Clause` populated at the statement/question end with the matching write/read/read-previous trigger and class; atom arrays on `Episode`/`DialogueBatch`, zeros elsewhere).
- **Atom store — PASS** (`AtomStore` mirrors `RelationStore` overwrite semantics; the token-keyed store and `simulate` are additive-untouched).
- **Split integrity — DEFECT:** three held-out *statement templates* were byte-identical to training templates — CAR/CAR_COLOR "I drive a {V} car.", SPEAKER/FAVORITE_COLOR "My favorite color is {V}.", SPEAKER/FAVORITE_FOOD "My favorite food is {V}." — and the rendered-string held-out test could not see it (held values differ).
- **Panel exclusion — PASS** (all 30 sentences, 2,000 training episodes, normalization; exact panel sentences stay excluded).

**Fixes applied** (commit `f7f19990`, after merging `main` — which also brought #1502's user-turn gate; the merge kept both additive field sets and now populates `user_turn` in the natural builder):

1. The three held templates are now distinct phrasings: "The car I drive is {V}."; "I like the color {V} best."; "My preferred food is {V}.".
2. New test `natural_held_out_templates_are_not_training_templates`: for every family, no held statement/query/previous template equals any training template of the same kind (structural comparison, `NSlot` now derives `PartialEq`).

`cargo fmt --check` pass; `stack_aerm` **24 passed / 0 failed** (20 natural/world + 4 gate). A one-line confirmation that the defect is closed is requested before merge.


### comment 5890012581 — 2026-09-29T12:10:35Z — Casey-allard

Reviewer confirmation (same session): **confirmed** — all three held statement templates are now distinct  sequences, the new  test locks the property and would have failed on the pre-fix duplications, and the merge with #1502 is clean (both additive field sets present, `user_turn` populated for natural episodes). Defect closed; class-A conditions hold.


---
_Comment/review ids in this file: 5889944919, 5890012581_

