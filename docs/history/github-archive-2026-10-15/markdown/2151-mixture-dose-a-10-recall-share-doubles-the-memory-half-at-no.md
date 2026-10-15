# #2151 — Mixture dose: a 10% recall share doubles the memory half at no measurable reply cost; abstention needs 25% (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T10:05:08Z
- **Updated:** 2026-10-10T10:09:49Z
- **Closed:** 2026-10-10T10:06:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2151
- **Merged:** 2026-10-10T10:06:35Z
- **Merge commit:** d9f883fcf16a7eb7328ed944db8395c08ae654c6
- **Base / Head:** main / UOR-Foundation:deepseek/mixture-dose-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Result: at a 10 % recall dose the v5 memory half doubles (10/40 → 20/40) at a reply panel statistically indistinguishable from the base's (43 → 37, paired p = 0.42); abstention is bought separately and only at 25 % and above (0/24 → 15/24 → 19/24). The pre-registered KEEP bar is missed by one memory row, so this is REJECT with the dose–response frontier as the result.** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: pointer-target supervised fine-tune · count 1/3 · headline before reply 28/232 `acceptable` + memory 21/40 `check_pass` → after reply **37/232** + memory **20/40** at a 10 % recall dose (base: 43/232 + 10/40)

## What and why

**Cycle 1** of the standing goal. The previous cycle measured the lever — the generated recall-dialogue mixture — but not its shape: the same recipe at a 55.06 % recall share moved v5 memory 10/40 → 21/40 and cost the reply panel 43/232 → 28/232. This cycle answers the question the next step needs: **is the memory gain separable from the reply cost by dose?** Four recall shares, one recipe, base artifact and seed, `pointer_gate_supervision=0` everywhere, pre-registered on #2029 before any compute.

## Result

| arm | recall share | v5 memory `check_pass` | unknowable `check_pass` | reply `fluent_and_relevant` | vs base (paired exact McNemar) |
|---|---:|---:|---:|---:|---|
| base | — | 10/40 | 1/24 | **43/232** | — |
| D0 | 0 % | 5/40 | 0/24 | pending — see below | — |
| **D10** | **10 %** | **20/40** | 0/24 | **37/232** | 22 / 16, **p = 0.42 (n.s.)** |
| **D25** | **25 %** | **20/40** | **15/24** | **28/232** | 22 / 7, **p = 0.0081** |
| D55 (#2145 control) | 55.06 % | 22/40 | 19/24 | **22/232** | 26 / 5, **p = 0.0002** |

Per category (`heldout_first_turn`): 34 → 28 → 23 → 17. Every v5 derangement control is 0. **D55 re-ran bit-identically to #2145's matched control** (`f76c8bc0…`), so its panels transfer without re-scoring.

**Decision: REJECT (no keeper).** The pre-registered bar needs one arm to hold memory ≥ 21/40 **and** reply ≥ 29/232 at the same time; the new doses are **one memory row short** at 20/40 and D55's reply half is 22/232. No threshold is moved. The line's count increments to **1/3** under the rule stated on #2029 before the reply numbers existed (both halves must be at least at their last-cycle levels; memory is not).

**What the cycle establishes:** (1) a **10 % dose doubles the memory half at no measurable reply cost** — the first artifact in this line to do so; (2) **abstention is a separate dose** (0/24 at 10 %, 15/24 at 25 %, 19/24 at 55 %) and costs reply cells when it is bought; (3) **more training on the base's own store is destructive, not neutral** (D0: 10/40 → 5/40, 1/24 → 0/24), so the mixture — not the step budget — is the lever. Criterion 1 remains **NOT MET** on both halves (20/40 against ≥ 34/40; 37/232 against ≥ 116/232); the base artifact is unchanged.

## Scope note (declared before the affected numbers existed)

The pre-registration named three arms for the reply panel. The shared single-slot local judge carries up to four clients across labs, so the cycle declared on #2029 — in #2029 comments 6095764623 and 6096059506, before any affected number existed — that it would score **D10 first, then D25**, and drop **D0** last: D0 already fails the memory half at 5/40 and cannot change the verdict. D0's reply reading is therefore **pending, not guessed**; its v5 half is reported. Everything else is complete.

## Checklist

- [x] Checks at this exact head: `python3 scripts/check_claim_wording.py` (docs-only change; no Rust path touched — the training binary and the grader are unchanged `main` builds, and the round's checks are the panel's own: frozen row checks, derangement controls, the base re-grade reproducing 43/232 per category, and the trainer's byte-identical re-run at D55).
- [x] Research record: `docs/labs/mixture-dose-2026-10-10/README.md`
- [x] Newest-first `docs/integration/current-state.md` entry with **Next:**
- [x] `STATUS.md` row and M1 cell; #2028 M1 table posted after the merge
- [x] Evidence bundle: cloud-store `mixture-dose-2026-10-10`, **5,058,048 bytes, md5 `d31b25ddabaa6c1a161cfd7ae3de0104`**
- [ ] After merge: verify on fresh `origin/main`, delete branch and worktree, `scripts/storage/uor-hygiene --apply`, final card on #2029

References #2029


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6096395376 — 2026-10-10T10:05:16Z — Casey-allard

**Validation at the exact head `5a93eb55220937f69cfbfdf06b2b90bef3b17514`** (docs-only change: one new record, one `current-state.md` entry, one `STATUS.md` row and cell; no Rust source, schema or CLI path is touched, so no compile or test applies).

| check | command | result |
|---|---|---|
| claim wording | `python3 scripts/check_claim_wording.py` | **pass** — "no unlinked machine-verified/exact-equivalence wording" |
| v5 acceptance, four arms | `chat-grade grade-replies` with the frozen `conversational-v5-checks.tsv` | memory 5 / 20 / 20 of 40 for D0 / D10 / D25, unknowable 0 / 0 / 15 of 24; **every derangement control 0** |
| reply panel, three points | `chat-grade grade-replies`, 232 rows, cap 64 | `fluent_and_relevant` 43 (base, re-graded from the sealed replies) / 37 (D10) / 28 (D25) / 22 (D55, #2145) |
| paired comparisons | exact McNemar on the 232 rows | base vs D10 **p = 0.42**, base vs D25 **p = 0.0081**, base vs D55 **p = 0.0002**, D10 vs D25 p = 0.136 |
| trainer reproducibility | D55 re-run at `8421e605f`, different pod | `model.safetensors` **byte-identical** to #2145's control arm (`f76c8bc0…`) |
| corpus disjointness | `dialogue-recall-corpus generate … panels=` = `data/panels` + the three open-reply-panel request files | **`leak pass`** for both new corpora (and for the reused 124 k corpus in #2145) |
| v5 never in training | the corpus generator's own leak report against `data/panels` | 0 matching examples in train or dev |

Evidence bundle: cloud-store object `mixture-dose-2026-10-10`, **5,058,048 bytes, md5 `d31b25ddabaa6c1a161cfd7ae3de0104`**.

**One declared gap:** the zero-dose arm's reply panel is **pending, not guessed** — the reduction was declared on #2029 (comments 6095764623 and 6096059506) before any affected number existed, because D0 already fails the memory half at 5/40 and cannot change the verdict. Its grading is running; if it lands before the merge it will be added in a follow-up commit to this PR.



---
_Comment/review ids in this file: 6096395376_

