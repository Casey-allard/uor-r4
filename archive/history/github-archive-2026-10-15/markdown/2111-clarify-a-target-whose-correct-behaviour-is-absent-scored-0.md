# #2111 — clarify: a target whose correct behaviour is absent, scored 0 of 27 (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T23:45:10Z
- **Updated:** 2026-10-09T23:49:20Z
- **Closed:** 2026-10-09T23:46:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2111
- **Merged:** 2026-10-09T23:46:32Z
- **Merge commit:** cb81cde84718ce7699e15bd92184842fddc02dd1
- **Base / Head:** main / UOR-Foundation:reply-clarify-kind
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Measured first, and the measurement changes what this kind is for

**Two readings, two denominators, same conclusion.**

Every reply in the panel flagged as asking a question (from `verdicts-29m.tsv`; only **two** are ill-posed): a bare generic question, **three that invent a specific** (*"What type of boat are you interested in?"*), one that **parrots the request back** (`heldout-119` — so the `question` flag over-counts clarify behaviour and cannot be the basis of a kind), and `heldout-189`, which **is** `Hello! How can I help you today?`.

The **8 ill-posed** rows whose reply asks: **five are the canned greeting verbatim** (`heldout-006/009/019/102/131`), the rest `heldout-107`, `heldout-118`, `heldout-189`. **Zero of the eight name the missing material — and zero of the 67 ill-posed rows demonstrate the behaviour at all.**

**The trap is live, not hypothetical:** `heldout-189` *is* the canned greeting sitting on an ill-posed row and flagged as asking a question.

**What the 27 fabricated-specific replies do:** 15 invent a capitalised name the request never gave (`heldout-105` invents *"the capital of China"* for a passage never supplied), 4 invent a value for a `[placeholder]`, 4 echo the placeholder verbatim, 4 invent a number. The distinction *asked for the missing thing* vs *invented a specific* is real and deterministic — and `abstain_exact`'s built-in `fabricated_specifics` rule already catches the negative half.

## The kind

**`CheckKind::Clarify`**, same six sites as `reply_exact`: the reply must **ask a question AND contain a row-specific phrase naming the missing material** (`which city`, `which text`, `which words`), and contain no forbidden term or key.

**27 of the 67 ill-posed rows got a key**; the other **40 keep `abstain_exact`, not replaced** — a bare salutation or constraint-only stub has nothing to name, and inventing a phrase key for them would be loosening the key rather than measuring the behaviour. The focused test caught a real defect (a clarify term that is also forbidden made the check unsatisfiable) and `parse_checks` now refuses it.

## The controls that must fail — and do

Run by the grader's own `RowCheck::passes` through `chat-grade check constants=`, against all 88 checks:

| control | why | result |
|---|---|---|
| `Hello! How can I help you today?` | **the live risk** — the sealed reply on ill-posed `heldout-189` | **0 of 88** |
| `Could you clarify?` | a bare generic question | **0 of 88** |
| `?` | an empty question mark | **0 of 88** |
| `What type of boat are you interested in?` | **invents** a specific — the sealed reply on `follow-03` | **0 of 88** |
| 13 observed memorised strings (existing arm) | original control | **0 of 88** |

**If a bare generic question had passed, the kind was not ready.** None passed, and **nothing was loosened**: the key is a phrase the request does not contain, so it cannot be satisfied by asking in general.

## The reading — a target, not a scoreboard

| population | checked | `check_pass` |
|---|---:|---:|
| content rows, `any` | 16 | 1 |
| content rows, `reply_exact` | 5 | 1 |
| **ill-posed, `clarify`** | **27** | **0** |
| ill-posed, `abstain_exact` | 40 | 0 |
| **total** | **88** | **2** |

Only **1 of the 27** clarify-row replies even asks a question. **The kind scores zero, and that is the intended reading: building it does not raise the score, it creates the ability to measure a behaviour the model does not have.** It is **not** coverage won and **not** rows that might now pass.

## Sealed and re-validated

checks sha256 `aba96550fb89cbb0…` · kinds `any` 16 / `reply_exact` 5 / `clarify` 27 / `abstain_exact` 40 · **88 checked rows (37.9%)** · `check_panel` **pass** · worst case **168 of 384** · `checks_without_loaded_request: []` · manifest from `data/panels/` **30 files, 30 OK** · v4 + v5 novelty **pass, pass** · bin tests **18 passed, 0 failed** · `chat-grade.rs` `cargo fmt --check` clean with other labs' reformatted files reverted.

## Delivery

New record section in [`docs/labs/reply-panel-deterministic-checks-2026-10-09/README.md`](docs/labs/reply-panel-deterministic-checks-2026-10-09/README.md) with `Next:` rewritten to the measured target; newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; the adversarial controls file `data/panels/reply-panel-controls.txt`; stale docs fixed (`data/panels/README.md` inventory, ladder runbook §6). `check_claim_wording.py` passes; `scripts/__pycache__/` not committed.

**Criterion 1 remains NOT MET and 43/232 is unchanged. STATUS/ROADMAP/#2028 unchanged — checked, not assumed.** 27 previously unscorable rows are now scorable and all 27 fail: a larger instrument, the same model. **References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

