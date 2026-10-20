# #2161 — Token-identity pointer reaches the failure mode (wrong-value 17 -> 12) and still leaves memory at 19/40: the pointer line is archived under the 3/3 pivot (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T14:15:18Z
- **Updated:** 2026-10-10T14:19:33Z
- **Closed:** 2026-10-10T14:16:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2161
- **Merged:** 2026-10-10T14:16:53Z
- **Merge commit:** 992d9082a4bffbc41010f777500012feeb2ded98
- **Base / Head:** main / UOR-Foundation:deepseek/pointer-identity-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Result: the decisive run the 3/3 pivot named — the **token-identity pointer** (`pointer_identity=0.25` and `1.0`, the multiset overlap of the six tokens before a source with the six before the query, no parameters) — **reaches the failure mode and not the bar**. Wrong-value failures fall **17 → 16 → 12 of 40** (the best binding reading this panel has produced) while v5 memory `check_pass` stays at **19/40 on both arms** (officially graded) against the pre-registered **21/40**, so the pointer/mixture line is **ARCHIVED** under D21 §1, exactly as the pivot card declared.** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: pointer-target supervised fine-tune · count 3/3, decisive run ARCHIVED · headline before reply 39/232 + memory 17/40 → after reply *pending* + memory 19/40

## What and why

**Cycle 4** and the one decisive run the [pivot card](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6097789124) named, pre-registered on #2029 before any compute. The residual deficit was *value selection*, and the pointer that copies the value had **no identity term**: its keys are hidden states, so "copy the same token" had to be learned by a 32-wide projection, and the copy-mass instrument showed it attending the sentence frame rather than the varying slot. `pointer_identity=` supplies exactly that term, changes the *score function* rather than the data or the loss (the two things three cycles measured flat), and had never been trained on this line.

## Result

| arm | `pointer_identity` | v5 memory `check_pass` | wrong-value / distractor-key / other | `dev_response_nll` | dev pointer hit |
|---|---:|---:|---|---:|---:|
| D10 (anchor, `none`) | — | **20/40** | 17 / 0 / 3 | 0.3773 | 0.377 |
| **I25** | **0.25** | **19/40** | **16** / 0 / 5 | 0.3671 | 0.287 |
| **I100** | **1.0** | **19/40** | **12** / 1 / 8 | 0.4666 | **0.003** |

Both arms' saved configs record `identity.weight_bp` 2500 and 10000, so the term is confirmed active rather than assumed from the flag. The failures **redistribute** (no-value 3 → 5 → 8) instead of disappearing, and at `pointer_identity=1.0` the pure identity score collapses the dev pointer hit rate to **0.003** — a direct measurement that a source whose six-token context matches the query's usually does not hold the answer. Every derangement control stays ≤ 1.

**Decision: ARCHIVE under D21 §1**, as pre-registered. The bar (memory ≥ 21/40 **and** reply ≥ 29/232 together) is not met; the secondary mechanism threshold (wrong-value < 12 of 40) is missed by one row on I100 and four on I25. The line's four cycles:

| cycle | PR | lever | v5 memory | reply `f_and_r` |
|---|---|---|---|---:|
| 1 | [#2151](https://github.com/UOR-Foundation/uor-r4/pull/2151) | recall mixture, dose 0/10/25/55 % | 5 → **20** → 20 → 22 of 40 | 36 → **37** → 28 → 22 of 232 |
| 2 | [#2155](https://github.com/UOR-Foundation/uor-r4/pull/2155) | read-binding supervision | 19 / 17 of 40 | 39 of 232 |
| 3 | [#2158](https://github.com/UOR-Foundation/uor-r4/pull/2158) | binding-dense mixture | 17 / 17 of 40 | pending |
| 4 | this PR | token-identity pointer | 19 / 19 of 40 | pending |

**No source is deactivated** — every arm was trained with options the trainer already carries — so the closure is this record plus an INDEX row in `docs/history/branch-archive/`, and the four records' commands stay reproducible. **Criterion 1 remains NOT MET** on both halves (19/40 against ≥ 34/40; reply pending against ≥ 116/232); the base artifact's 10/40 and 43/232 are unchanged and it remains the base.

**Next M1 piece:** an **addressed-memory operator in the dialogue stack** — `dialogue-train` cannot build one today (`DialogueSettings` has no memory fields while the LM trainer exposes `memory_layers=`/`memory_sub_keys=`/`memory_top_k=`/`memory_score=`/`memory_codebook=`), so every artifact behind these numbers answers rows named `multi_turn_memory` with a copy pointer and **no memory reader**.

## Checklist

- [x] Checks at this exact head: `python3 scripts/check_claim_wording.py` (docs-only change; no Rust path touched — the trainer and grader are unchanged `main` builds; the round's checks are the frozen row checks, the derangement controls, the failure-cause classification, and the saved configs that record the identity weight).
- [x] Research record: `docs/labs/pointer-identity-2026-10-10/README.md`; branch-archive INDEX row added for the closed line
- [x] Newest-first `docs/integration/current-state.md` entry with **Next:**
- [x] `STATUS.md` row and M1 cell; #2028 M1 table posted after the merge
- [x] Evidence bundle: cloud-store `pointer-identity-2026-10-10`, **756,736 bytes, md5 `f366ee2d4099b05ac7d3b1728f933e82`**
- [ ] After merge: verify on fresh `origin/main`, delete branch and worktree, `scripts/storage/uor-hygiene --apply`, final card and archive notice on #2029

**Declared pending:** the primary arm's reply-panel guard (232 rows, cap 64) — its replies are generating and its grading is in the shared judge queue; the pre-registration named anything unreached as pending, never guessed, and it cannot change the archive verdict.

References #2029


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6098396445 — 2026-10-10T14:15:26Z — Casey-allard

**Validation at the exact head `9e05aa11fa816714b26f510f319a466d9050753d`** (docs-only change: one new record, one `current-state.md` entry, one `STATUS.md` row and cell, one `docs/history/branch-archive/INDEX.md` row for the closed line; no Rust source, schema or CLI path is touched, so no compile or test applies).

| check | command | result |
|---|---|---|
| claim wording | `python3 scripts/check_claim_wording.py` | **pass** — "no unlinked machine-verified/exact-equivalence wording" |
| the term is active | the arms' saved `config.json` | `{"dim": 32, "identity": {"weight_bp": 2500}, "init_seed": 1}` and `{"…": 10000}` — recorded, not assumed from the flag |
| v5 acceptance | `chat-grade grade-replies`, frozen `conversational-v5-checks.tsv` | **I25 19/40** memory, 0/24 unknowable, panel 19/64, `acceptable` 10, derangement **0**; **I100 19/40** memory, 5/24, panel 24/64, `acceptable` 10, derangement 1 |
| failure causes | judge-free classification against the same frozen checks | wrong-value **17 (anchor) → 16 → 12**; distractor-key 0 → 0 → 1; no/other 3 → 5 → 8 |
| mixture unchanged | `sha256sum` of `mix-10/tokens.u16` | `888241261c378138…` — the anchor's own store, so the identity term is the only difference |
| archive row | `docs/history/branch-archive/INDEX.md` | added, with the disposition (closed negative, no source patch because no unique branch source exists) and the four records linked |

Evidence bundle: cloud-store object `pointer-identity-2026-10-10`, **756,736 bytes, md5 `f366ee2d4099b05ac7d3b1728f933e82`**.

**Declared pending:** the primary arm's reply-panel guard (232 rows, cap 64). Its replies are generated and its grading is in the shared judge queue; the pre-registration named anything unreached as **pending, never guessed**, and the archive verdict is decided on the memory half (19/40 against a 21/40 bar).



---
_Comment/review ids in this file: 6098396445_

