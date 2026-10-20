# #2158 — Binding-dense mixture fails (20/40 -> 17/40) and the pointer line reaches 3/3: pivot to the token-identity run (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T12:59:18Z
- **Updated:** 2026-10-10T13:03:37Z
- **Closed:** 2026-10-10T13:00:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2158
- **Merged:** 2026-10-10T13:00:49Z
- **Merge commit:** 020a13fe7c5ae735fd06e7c6503bdbc9f7c99ad9
- **Base / Head:** main / UOR-Foundation:deepseek/binding-dense-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Result: doubling the recall mixture's binding families at the same dose makes the memory panel *worse* — v5 memory `check_pass` **20/40 (v2 stream, 10 % dose) → 17/40 (v1 stream, 10.2 %)**, and 17/40 at 25.5 % with a **new** failure mode (7 rows name the distractor's *key*). The standing bar is not met, the line's count reaches **3/3**, and this PR carries the **D21 pivot card**: one decisive model-changing run — the **token-identity pointer** — with the line archived if it fails.** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: pointer-target supervised fine-tune · count 3/3 · headline before reply 39/232 `acceptable` + memory 20/40 `check_pass` → after reply *pending* + memory **17/40**

## What and why

**Cycle 3** of the standing goal. Cycle 2 named the deficit — the failing memory rows are **selection between two stated values** — and the recall generator's `generator=v2` stream dilutes the families that teach that selection (binding is ~15 % of its dialogues) with six added families. This cycle ran the **same recipe on `generator=v1`**, whose dialogues are *only* those six families: the 12,150-dialogue arm carries **4,561 binding dialogues (37.5 %)**, a 2.5× density at the same response share. Pre-registered on #2029 before any compute, including the statement that a cycle which does not move the headline **is** the 3/3 pivot.

## Result

| arm | recall share | generator | v5 memory `check_pass` | unknowable | failure mix (wrong-value / distractor-key / other) | v2 counterpart at the same dose |
|---|---:|---|---:|---:|---|---|
| D10 (cycle 1) | 10.00 % | v2 | **20/40** | 0/24 | 17 / 0 / 3 | — |
| **V10** | 10.21 % | **v1** | **17/40** | 2/24 | **19 / 2 / 2** | 20/40 |
| D25 (cycle 1) | 25.00 % | v2 | **20/40** | 15/24 | 15 / 0 / 5 | — |
| **V25** | 25.48 % | **v1** | **17/40** | **18/24** | 16 / **7** / 0 | 20/40 |
| D55 (cycle 1) | 55.06 % | v2 | 22/40 | 19/24 | — | — |

**Doubling the binding density does not buy binding**: three rows worse at the same dose on both arms, and at V25 **seven rows name the distractor's key** (`"Your kitten is named …"` for a question about the brother) — a failure mode the v2 arms never produced (0 rows at D10 and D25). V25 does keep the abstention face of the v1 stream (**18/24**, the best of any arm at that dose). The reply-panel guard for the primary arm is **pending** — declared in the pre-registration and named as pending rather than guessed; it cannot change the verdict, which the memory half decides.

**Decision: REJECT, count 3/3, pivot card delivered.** Three merged PRs on this line — the mixture dose ([#2151](https://github.com/UOR-Foundation/uor-r4/pull/2151)), the read-binding objective ([#2155](https://github.com/UOR-Foundation/uor-r4/pull/2155)) and this one — without the bar (memory ≥ 21/40 **and** reply ≥ 29/232 at once). The pivot names **one decisive model-changing run**: the **token-identity pointer** (`pointer_identity=WEIGHT`, the multiset overlap of the six tokens before a source with the six before the query — already in the trainer, never trained on this line), at the 10 % mixture dose, arms 0.25 and 1.0 against cycle 1's D10 as the matched anchor, with the **standing bar unchanged**. **Stop rule:** if no arm clears it, the line stops and is archived, and the next M1 piece moves to an **addressed-memory operator in the dialogue stack** — which `dialogue-train` cannot build today (its settings carry no memory fields, though the LM trainer has `memory_layers=`), so every artifact behind these numbers answers "multi-turn memory" rows with a copy pointer and **no memory reader**.

**Criterion 1 remains NOT MET** on both halves (17/40 against ≥ 34/40; reply pending against ≥ 116/232); the base artifact's 10/40 and 43/232 are unchanged.

## Checklist

- [x] Checks at this exact head: `python3 scripts/check_claim_wording.py` (docs-only change; no Rust path touched — the trainer and grader are unchanged `main` builds; the round's checks are the panel's own frozen row checks, the derangement controls, the v1 family composition from the generator's own report, and the byte-level corpus identities).
- [x] Research record: `docs/labs/binding-dense-2026-10-10/README.md` (the pivot card is on #2029, posted with this delivery)
- [x] Newest-first `docs/integration/current-state.md` entry with **Next:**
- [x] `STATUS.md` row and M1 cell; #2028 M1 table posted after the merge
- [x] Evidence bundle: cloud-store `binding-dense-2026-10-10`, **1,600,512 bytes, md5 `0cc1b5ef952306971d218bab5dd11f3c`**
- [ ] After merge: verify on fresh `origin/main`, delete branch and worktree, `scripts/storage/uor-hygiene --apply`, final card on #2029

References #2029


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6097752515 — 2026-10-10T12:59:22Z — Casey-allard

**Validation at the exact head `cf22bf757c67324b14e83fa830897fb763266642`** (docs-only change: one new record, one `current-state.md` entry, one `STATUS.md` row and cell; no Rust source, schema or CLI path is touched, so no compile or test applies).

| check | command | result |
|---|---|---|
| claim wording | `python3 scripts/check_claim_wording.py` | **pass** — "no unlinked machine-verified/exact-equivalence wording" |
| v1 family composition | `dialogue-recall-corpus generate generator=v1` probe, 200 dialogues | exactly the six v1 families — **binding 80/200 (40 %)**, update 23, assistant-stated 24, abstain 23, cross-relation 23, reverse 27; the 12,150-dialogue arm carries 4,561 binding dialogues (37.5 %) |
| corpus disjointness | generator's own leak report, `panels=` = `data/panels` + the three open-reply-panel request files | **`leak pass`** for both v1 corpora |
| doses | measured mask-1 response shares over the mixed stores | **10.21 %** (V10) and **25.48 %** (V25), against the v2 counterparts' 10.00 % and 25.00 % |
| v5 acceptance | `chat-grade grade-replies`, frozen `conversational-v5-checks.tsv` | **V10 17/40** memory, 2/24 unknowable, panel 19/64, derangement 1; **V25 17/40** memory, **18/24** unknowable, panel 35/64, derangement 0 |
| failure causes | judge-free classification against the same frozen checks | V10 wrong-value 19 / distractor-key 2 / other 2; **V25 wrong-value 16 / distractor-key 7** — a mode the v2 arms produced 0 of |

Evidence bundle: cloud-store object `binding-dense-2026-10-10`, **1,600,512 bytes, md5 `0cc1b5ef952306971d218bab5dd11f3c`**.

**Declared pending:** the primary arm's reply-panel guard (232 rows, cap 64). Its replies are generated (10,913 ids) and its grading is still in the shared judge queue; the pre-registration named anything unreached as **pending, never guessed**, and the verdict is decided on the memory half (17/40 against a 21/40 bar) so it cannot change the outcome.



---
_Comment/review ids in this file: 6097752515_

