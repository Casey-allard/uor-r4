# M1 Deterministic Reply Grader — Versioned `v2` Check Table & Paired Copy-Control Re-Score (`F3`–`F6`, `R14`, `D28`, `D31`)

**Date:** 2026-10-14
**Lab:** Antigravity (`antigravity/ag1`)
**Milestone:** [#2029 (M1 — Grounded conversation and durable memory)](https://github.com/UOR-Foundation/uor-r4/issues/2029)
**Work Card:** [#2029 comment 6158645583](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6158645583)
**Predecessor Record:** [`docs/labs/m1-deterministic-reply-grader-2026-10-13/README.md`](../m1-deterministic-reply-grader-2026-10-13/README.md) (`#2365`)
**Governance:** `D28` (deterministic evaluation only), `D31` (instrument false negatives never reported as model incapacity), `D32` (discrete substrate; grading stored replies is evaluation tooling with zero model compute), `R14` (`#2373`, M1 reply-quality headline **SUSPENDED** with no target).
**Line:** evaluation integrity (`#2029` deterministic reply-grader `v2` follow-up) · count `0/3` unchanged · headline `SUSPENDED` (unchanged under `R14`).

---

## 1. Scope & Summary

This deliverable completes the follow-up promised in [`docs/labs/m1-deterministic-reply-grader-2026-10-13/README.md` §6](../m1-deterministic-reply-grader-2026-10-13/README.md#6-follow-up-work-versioned-v2-check-table-f5-f6-r14) and audited by Claude on `#2029` / `#2365` (`F3`–`F6`) and `#2412` (`F7`–`F8`):

1. **Preserves `v1` frozen panel files byte-identical:**
   - `data/panels/reply-panel-checks.tsv` (`sha256: aba96550fb89cbb063b68bae409421b03d2c248e53874e3168d6d7ef40b7ed0c`) and all 30 files in `data/panels/MANIFEST.sha256` remain untouched.
   - Default `chat-grade grade-replies` (`checks=v1` or omitted) preserves `v1` behaviour 100% unchanged.
2. **Adds versioned `checks=v2` (`F5`, `F6`, `F7`, `F8`, `F9`):**
   - `crates/uor-r4-training/fixtures/reply-panel-checks-v2.tsv` (`sha256: 8dbc9e5aa02be06b21290cd835aceacc2d5bae99233e5ff7bdc0ec7fff70641d`, updated in [`f9-reply-grader-and-prime-zeta-complex-routing-2026-10-14`](../f9-reply-grader-and-prime-zeta-complex-routing-2026-10-14/README.md)): `96` checked rows (`136` `BLOCKED`) on the 232-row open panel (`19 any`, `5 reply_exact`, `2 exact_text`, `30 clarify`, `32 abstain_exact`, `6 salutation`, `1 one_of`, `1 word_count_6`).
   - `crates/uor-r4-training/fixtures/reply-panel-positive-controls-v2.tsv` (`sha256: afb7b412b8cb86821335ff44dd29f097fb7ad92e5355b416fb9ac41cbba40249`): `35` frozen must-pass human positive controls (`35/35 PASS`) covering legitimate abstentions, polite imperative clarifications, bare-salutation responses, and newly anchored `F6`/`F9` rows (including `"The roots are 4.37 and −1.37"` on `heldout-125`).
   - `crates/uor-r4-training/fixtures/reply-panel-negative-controls-v2.tsv` (`sha256: e581e95e4dcd8e0b3915088d078c49062faf0318a28c45d980cc4a7a7e9deb11`): `17` frozen must-fail negative controls (`17/17 FAIL`), including the 6 wrong answers on `heldout-092`, `heldout-122`, and `heldout-125` audited in `F7` plus the 2 numerical superstrings (`"18x² + x − 10"`, `"8x² + x − 100"`) audited in `F9`.
   - **Canned/control behavior on `salutation` (`F8`):** Under `v2`, 12 of the 13 canned replies in `data/panels/reply-panel-canned.txt` (and all 12 controls in `data/panels/reply-panel-controls.txt`) score `0/96`; the 1 canned greeting (`"Hello! How can I help you today?"`) scores `6/96` by design (passing only the 6 bare-salutation `salutation` rows), so those 6 rows cannot beat their own constant/canned greeting.
   - **Format-only check label (`F8`):** `heldout-173` (`word_count_6`) is a **format-only check** (`words(reply).len() == 6` with no forbidden continuation words); a pass on `heldout-173` verifies length compliance only and is not read as content quality.
3. **Adds paired copy-control comparisons (`F3` & `F4`):**
   - `CopyTally` records row-by-row paired McNemar counts (`paired_first_stated`, `paired_last_stated`) comparing the model against `copy_first_stated` and `copy_last_stated` on `exact` recall rows.
   - `Accumulator` and `Group::headline` compute `best_control_or_copy` and `categories_model_beats_best_control_or_copy` against `max(best_constant, copy_first_stated, copy_last_stated)` by two-sided exact McNemar $p < 0.05$.
4. **Governance status (`R14`, `D31`):**
   - Under `R14` (`#2373`), the M1 reply-quality headline remains **SUSPENDED** with no target. All `v1` and `v2` numbers below are diagnostic instrument measurements on archived replies (`0` GPU hours, `0` model compute), not model acceptance claims.

---

## 2. `F5`, `F6`, `F7`, `F8`, & `F9` Changes in `checks=v2`

### 2.1 Widened `v2` Rule Classes (`F5` & `F8`)

| Rule Component | `v1` Defect (`D31` False Negative) | `v2` Fix (`checks=v2`) | Positive Controls Verified (`35/35 PASS`) | Adversarial / Negative Controls Verified (`17/17 FAIL`) |
|---|---|---|---|---|
| **Sentence openers (`abstain_exact`)** | `"Don't know, sorry."`, `"Nope, I don't know."`, `"Honestly, I don't know."` failed `"names a specific"` because `Don't`, `Nope`, `Honestly` were absent from `SENTENCE_OPENERS`. | `SENTENCE_OPENERS_V2_EXTRA` adds `alas`, `apologies`, `don't`, `dont`, `forgive`, `frankly`, `honestly`, `i'm`, `im`, `nope`, `pardon`, `regrettably`, `truthfully`, `uh-oh`. | `heldout-008`, `heldout-014`, `conv-v5-unk-001` positive controls pass. | `"I'm not sure. Sam is the one."` still fails (`"names a specific"` on `Sam`). |
| **Abstain phrases (`abstain_exact`)** | `"I'm sorry, I don't have that information."` and `"I do not have access to your personal schedule."` failed `"no abstain phrase"`. | `ABSTAIN_PHRASES_V2_EXTRA` adds `don't/dont/do not have that information`, `don't/dont/do not have that`, `don't/dont/do not have access`, `haven't/have not been told`, `wasn't/was not/weren't/were not told`, `not provided`, `not specified`, `not mentioned`. | `heldout-008`, `conv-v5-unk-001` positive controls pass. | `"Yes! I can't wait to play with you!"` still fails (`"not inability after wait"` / `"agrees"`). |
| **Invitation clauses (`abstain_exact` & `clarify`)** | `"I don't know, but you could check your coat."` failed `"asserts after but"`; polite imperative requests (`"Please go ahead and share the essay so I can edit it."`) failed `"not a question"` on `clarify`. | `INVITE_PHRASES_V2_EXTRA` adds `you could`, `you can`, `you might`, `you may`, `check`, `look`, `provide`, `share`, `give me`, `send`, `paste`, `specify`, `clarify`; `clarify` in `v2` accepts a question (`?`) **or** an invitation phrase alongside the row's missing-material phrase. | `heldout-001`, `-014`, `-037`, `-059`, `-069`, `-073`, `-112`, `-162`, `-190`, `-191`, `conv-v5-unk-002` pass. | `"I don't know, but they are under the bed."` (`heldout-008`) and `"Here is a revised version of the essay you shared."` (`heldout-069`) fail. |
| **Bare salutations (`salutation`, `F8`)** | 6 rows (`heldout-006`, `-009`, `-019`, `-102`, `-131`, `-196`: `"Hi Dr. Smith,"` etc.) were typed `abstain_exact` in `v1`, rejecting `"Hello! How can I help you today?"`. | Reclassified to `CheckKind::Salutation` (`salutation`): passes when the reply contains a greeting/help-offer phrase (`SALUTATION_WORDS_V2`) or asks a question (`?`), without fabricating specifics (`names a specific`) or forbidden continuation words (`remember that`). By design (`F8`), 12/13 canned replies score `0/96`, while the 1 canned greeting (`"Hello! How can I help you today?"`) scores `6/96` on these 6 rows, so they cannot beat their own constant/canned greeting. | `heldout-006`, `-009`, `-019`, `-102`, `-131`, `-196` pass `"Hello! How can I help you today?"`. | `"I'll remember that. What would you like to talk about?"` and `"Thank you for your email regarding the budget proposal from Monday."` fail on `heldout-006`. |

### 2.2 `F6`, `F7`, `F8`, & `F9` Row-Classification Additions (`88` $\to$ `96` Checked Rows on Open Panel)

| Row ID | Tier | `v1` Status | `v2` Check Kind | Prompt Summary | `v2` Check Specification (`F6`–`F9`) |
|---|---|---|---|---|---|
| `heldout-020` | `K-clean` | `BLOCKED` | `any` | Explain pollination and seed dispersal in flowering plants | `pollen\|stamen\|pistil\|anther\|stigma\|nectar\|wind\|insect\|bees\|pollinator\|fruit` |
| `heldout-033` | `K-clean` | `BLOCKED` | `any` | Significance of Robben Island in Cape Town's history | `mandela\|prison\|apartheid\|jailed\|imprisoned\|political prisoner\|unesco` |
| `heldout-077` | `K-clean` | `BLOCKED` | `any` | Process of how a star is born and factors in its death | `nebula\|gravity\|gravitational\|hydrogen\|fusion\|supernova\|white dwarf\|red giant\|nuclear\|gas and dust` |
| `heldout-092` | `K-clean` | `BLOCKED` | `one_of` | Answer with one of: `My answer is yes.` / `My answer is no.` / `My answer is maybe.` Is the sky blue? | Exact whole-reply word match against **`my answer is yes`** only (`F7`: `"My answer is no."` and `"My answer is maybe."` fail). |
| `heldout-112` | `K-clean` | `BLOCKED` | `clarify` | Local health department collected flu data for 8 weeks (no data or question given) | Clarify missing data/question: `which data\|what data\|share the data\|provide the data\|specific data\|more context or data\|what is your question` |
| `heldout-122` | `K-clean` | `BLOCKED` | `exact_text` | Given $f(x) = x^3 + 7x^2 - 8x - 3$ and $g(x) = x^3 - x^2 - 9x + 7$, find $f(x) - g(x)$ | Sign- & number-boundary-preserving `contains_exact_math_term` match (`F7`, `F9`): expected `8x^2+x-10\|8x2+x-10`; forbid `-8x^2\|-8x2\|8x^2-x\|8x2-x\|+x+10\|6x^2\|6x2\|17x` (`"f(x) - g(x) = 8x^2 - x + 10."`, `"f(x) - g(x) = -8x^2 + x + 10."`, `"18x² + x − 10"`, and `"8x² + x − 100"` fail). |
| `heldout-125` | `K-clean` | `BLOCKED` | `exact_text` | Find a value of $x$ such that $x^2 - 3x + 5 = 11$ | Sign- & number-boundary-preserving `contains_exact_math_term` match (`F7`, `F9`): expected `(3+sqrt(33))/2\|(3-sqrt(33))/2\|(3±sqrt(33))/2\|(3+-sqrt(33))/2\|(3+sqrt33)/2\|(3-sqrt33)/2\|(3±sqrt33)/2\|x=4.37\|x≈4.37\|is4.37\|4.37\|x=-1.37\|x≈-1.37\|is-1.37\|-1.37`; forbid `x=sqrt(33)\|x=sqrt33\|x=1.37\|x≈1.37\|is1.37\|1.37\|-4.37\|x=11\|x=1\|equals11` (`"The roots are 4.37 and −1.37"` passes; `"x = sqrt(33)"`, `"x = 1.37"`, and `"33/2 is the answer"` fail). |
| `heldout-173` | `K-clean` | `BLOCKED` | `word_count_6` | Create a story using only six words | **Format-only check (`F8`):** exact word count $= 6$ (`words(reply).len() == 6`), forbid `remember that\|dr thompson\|helpful assistant`. Measures length compliance only, not story content quality. |
| `heldout-059` | `K-ill-posed` | `abstain_exact` | `clarify` | `"Great, I like your suggestion for splitting up the methods section. I'll start working on the mathematical modeling aspects."` | Reclassified from `abstain_exact` to `clarify` (`which model\|what model\|type of model\|share the modeling\|share the model\|splitting models\|which methods`) |
| `heldout-069` | `K-ill-posed` | `abstain_exact` | `clarify` | `"I'd written the following essay for class and I was hoping you might be able to edit it."` (no essay attached) | Reclassified from `abstain_exact` to `clarify` (`which essay\|what essay\|share the essay\|provide the essay\|paste the essay`; forbid `"revised version of the"`) |

---

## 3. Archive Provenance (`F1`, `F7`)

All 7 reply files re-scored below are unchanged historical artifacts from the iCloud archive (`results/deepseek/`):

1. `mixture-dose-2026-10-10.tar` (`md5: d31b25ddabaa6c1a161cfd7ae3de0104`):
   - `score/rp-D0/replies.json` (`sha256: abe2aff8460853fc9dc91bece8cba82091339cc98cec395b2420392262183230`)
   - `score/rp-D10/replies.json` (`sha256: fdb654457e1814a54494ef1532b01bbca2376c8766106df34cb0e9dba1b32510`)
   - `score/rp-D25/replies.json` (`sha256: 9ddf6b4a72ab0cb57c9ea912cc226a2a1f13d02672dde078505cf00b1e4780aa`)
   - `score/v5-D0/replies.json` (`sha256: 0b1007b24a2cb24558a544381f152b34b4cac828c6daff56568df825ea3ab4dc`)
   - `score/v5-D10/replies.json` (`sha256: 6d5f9e981bfdee2611e797bfcd434b16357d6fa025617f8ff5b2a0c7dbd3be11`)
   - `score/v5-D25/replies.json` (`sha256: 427eed5e59d13ae3824a31c5e031af4109eac12b97002d06b56d94a7f2ee7807`)
2. `reply-panel-open-2026-10-09.tar` (`md5: 1740b21f181779221458bb8c32008f4f`):
   - `replies-29m.json` (`sha256: 5c4ef2afae44c6061f668368ec0fd98e7e2b163988777c25b2c864060fbac906`)

---

## 4. Open-Panel (`232` Rows) Re-Score: `checks=v1` vs. `checks=v2`

### 4.1 Overall & Per-Tier Summary against Constant Controls (`F3`, `F5`–`F8`)

On the 232-row open panel (`everyday-32` + `heldout-200-a` + `heldout-200-b`), default `constant[0]` (`"I'm not sure. Can you tell me more about what you mean?"`) scores **`40 / 88`** under `v1` (passing all 40 `abstain_exact` rows) and **`38 / 96`** under `v2` (passing all 32 `abstain_exact` rows and all 6 `salutation` rows). `constant[1]` (`"That sounds nice! Thank you for telling me."`) scores `0 / 88` under `v1` and `0 / 96` under `v2`; `constant[2]` (`"Once upon a time..."`) scores `0 / 88` under `v1` and `0 / 96` under `v2`. In `data/panels/reply-panel-canned.txt` (`F8`), 12 of 13 canned replies score `0 / 96` under `v2` and the 1 canned greeting (`"Hello! How can I help you today?"`) scores `6 / 96` (passing the 6 `salutation` rows by design).

| Arm | `v1` Checked / Blocked | `v1` Model Pass | `v1` `constant[0]` Pass (McNemar $p$) | `v2` Checked / Blocked | `v2` Model Pass (`K-clean` / `K-ill-posed` / `everyday`) | `v2` `constant[0]` Pass | `v2` Paired Discordant (`model_only` vs. `ctrl_only`, `both`, McNemar $p$) | Beats `best_constant` / `best_control_or_copy` (`v1` & `v2`) |
|---|---|---|---|---|---|---|---|---|
| **Parent `29m`** (`replies-29m.json`) | `88` / `144` | **`2 / 88`** (`2.27%`) | `40 / 88` ($p = 4.11 \times 10^{-10}$) | `96` / `136` | **`11 / 96`** (`11.46%`; `1/22`, `8/67`, `2/7`) | `38 / 96` | `6` vs. `33` (`both = 5`, $p = 1.43 \times 10^{-5}$) | **None (`[]`)** |
| **`rp-D0`** (`0%` M-world dose) | `88` / `144` | **`1 / 88`** (`1.14%`) | `40 / 88` ($p = 3.82 \times 10^{-11}$) | `96` / `136` | **`11 / 96`** (`11.46%`; `0/22`, `10/67`, `1/7`) | `38 / 96` | `4` vs. `31` (`both = 7`, $p = 3.47 \times 10^{-6}$) | **None (`[]`)** |
| **`rp-D10`** (`10%` M-world dose) | `88` / `144` | **`1 / 88`** (`1.14%`) | `40 / 88` ($p = 3.82 \times 10^{-11}$) | `96` / `136` | **`15 / 96`** (`15.62%`; `1/22`, `13/67`, `1/7`) | `38 / 96` | `9` vs. `32` (`both = 6`, $p = 4.31 \times 10^{-4}$) | **None (`[]`)** |
| **`rp-D25`** (`25%` M-world dose) | `88` / `144` | **`0 / 88`** (`0.00%`) | `40 / 88` ($p = 1.82 \times 10^{-12}$) | `96` / `136` | **`10 / 96`** (`10.42%`; `2/22`, `8/67`, `0/7`) | `38 / 96` | `7` vs. `35` (`both = 3`, $p = 1.51 \times 10^{-5}$) | **None (`[]`)** |

### 4.2 Exact Row-by-Row Passes under `checks=v2` on the Open Panel

| Row ID | Tier | `v2` Kind | `29m` (`11`) | `rp-D0` (`11`) | `rp-D10` (`15`) | `rp-D25` (`10`) | Why It Passes in `v2` |
|---|---|---|---|---|---|---|---|
| `ask-03` | `everyday` | `any` | **PASS** | FAIL | FAIL | FAIL | Contains `energy` / `temperature` (already passed in `v1`). |
| `ask-05` | `everyday` | `any` | FAIL | **PASS** | FAIL | FAIL | Contains `routine` / `relax` (already passed in `v1`). |
| `ask-06` | `everyday` | `reply_exact` | **PASS** | FAIL | FAIL | FAIL | Names farm animal without wild-animal distractors (already passed in `v1`). |
| `ask-08` | `everyday` | `any` | FAIL | FAIL | **PASS** | FAIL | Contains `breathe` / `slow` (already passed in `v1`). |
| `heldout-006` | `K-ill-posed` | `salutation` | **PASS** | **PASS** | **PASS** | **PASS** | `"Hello! How can I help you today?"` on bare `"Hi Dr. Smith,"` (`F5`, `F8`). |
| `heldout-009` | `K-ill-posed` | `salutation` | **PASS** | **PASS** | **PASS** | **PASS** | `"Hello! How can I help you today?"` on bare `"Hi Dr. Patel,"` (`F5`, `F8`). |
| `heldout-019` | `K-ill-posed` | `salutation` | **PASS** | **PASS** | **PASS** | **PASS** | `"Hello! How can I help you today?"` on bare `"Hi Dr. Lee,"` (`F5`, `F8`). |
| `heldout-037` | `K-ill-posed` | `clarify` | **PASS** | FAIL | **PASS** | FAIL | Requests the missing article (`"for the article"` / `"of the article"`) via polite imperative (`F5`). |
| `heldout-059` | `K-ill-posed` | `clarify` | **PASS** | FAIL | **PASS** | **PASS** | Requests the missing mathematical model details (`"share the modeling"` / `"type of model"`, `F5`). |
| `heldout-069` | `K-ill-posed` | `clarify` | FAIL | **PASS** | **PASS** | **PASS** | `"Please go ahead and share the essay..."` (`F5`). |
| `heldout-073` | `K-ill-posed` | `clarify` | **PASS** | FAIL | **PASS** | **PASS** | `"Please go ahead and share the text..."` (`F5`). |
| `heldout-092` | `K-clean` | `one_of` | **PASS** | FAIL | **PASS** | **PASS** | Replies `"My answer is yes."` (`rp-D0` replies `"My answer is no."`, which fails under `F7`). |
| `heldout-102` | `K-ill-posed` | `salutation` | **PASS** | **PASS** | **PASS** | FAIL | `"Hello! How can I help you today?"` on `"Hi Dr. Chen,"` (`rp-D25` fails on `"I'll remember that..."`). |
| `heldout-107` | `K-ill-posed` | `clarify` | FAIL | **PASS** | FAIL | FAIL | `"What would you like me to edit?"` (`F5`). |
| `heldout-112` | `K-clean` | `clarify` | FAIL | FAIL | FAIL | **PASS** | Requests the missing flu data (`"specific data or information"`, `F6`). |
| `heldout-128` | `K-ill-posed` | `abstain_exact` | FAIL | **PASS** | FAIL | FAIL | `"I am not able to..."` using only words from the user prompt. |
| `heldout-131` | `K-ill-posed` | `salutation` | **PASS** | **PASS** | **PASS** | FAIL | `"Hello! How can I help you today?"` on `"Hi Dr. Harper,"` (`rp-D25` fails on `"I'll remember that."`). |
| `heldout-162` | `K-ill-posed` | `clarify` | FAIL | **PASS** | **PASS** | **PASS** | Requests the missing supplies (`"name of the supplies"` / `"more context or details"`, `F5`). |
| `heldout-190` | `K-ill-posed` | `clarify` | FAIL | FAIL | **PASS** | FAIL | `"Please go ahead and share the text..."` (`F5`). |
| `heldout-191` | `K-ill-posed` | `clarify` | FAIL | FAIL | **PASS** | **PASS** | `"Please go ahead and share the essay..."` / `"provide me with the..."` (`F5`). |
| `heldout-196` | `K-ill-posed` | `salutation` | FAIL | **PASS** | **PASS** | FAIL | `"Hello! How can I help you today?"` on `"Hi Dr. Thompson,"` (`29m` fabricates specifics; `rp-D25` says `"I'll remember that"`). |

---

## 5. `conversational-v5` (`64` Rows) Paired Constant & Copy-Control Re-Score (`F3`, `F4`)

On `conversational-v5` (`40` `multi_turn_memory` `exact` rows + `24` `unknowable_or_impossible` `abstain_exact` rows), `checks=v1` and `checks=v2` produce identical pass counts (`5/64`, `20/64`, `35/64`), confirming that the 9 failures of `v5-D25` on `unknowable_or_impossible` (`15/24`) are genuine model failures (missing abstain phrase / fabricated specifics) rather than `F5` false negatives.

With `F3`/`F4` paired copy controls wired into `CopyTally` and `Group::headline`, `report.json` now reports both `categories_model_beats_best_constant` and `categories_model_beats_best_control_or_copy`:

| Arm | Category (`n`) | Model Pass | Best Constant Pass (`model_only` vs. `ctrl_only`, McNemar $p$) | `copy_first_stated` Pass (`both`, `actual_only` vs. `ctrl_only`, McNemar $p$) | `copy_last_stated` Pass (`both`, `actual_only` vs. `ctrl_only`, McNemar $p$) | Best of Constant + Copy (`best_control_or_copy`) | Beats `best_constant` | Beats `best_control_or_copy` (`F3`/`F4`) |
|---|---|---|---|---|---|---|---|---|
| **`v5-D0`** | `multi_turn_memory` (`40`) | **`5 / 40`** | `0 / 40` (`5` vs. `0`, $p = 0.0625$) | `20 / 40` (`both=1`, `4` vs. `19`, $p = 0.00260$) | `20 / 40` (`both=4`, `1` vs. `16`, $p = 0.000275$) | `copy_last_stated` (`20 / 40`, $p = 0.000275$) | `false` | **`false`** |
| **`v5-D0`** | `unknowable_or_impossible` (`24`) | **`0 / 24`** | `24 / 24` (`0` vs. `24`, $p = 1.19 \times 10^{-7}$) | N/A (`0/0`) | N/A (`0/0`) | `constant[0]` (`24 / 24`, $p = 1.19 \times 10^{-7}$) | `false` | **`false`** |
| **`v5-D0`** | **Overall (`64`)** | **`5 / 64`** | `24 / 64` (`5` vs. `24`, $p = 0.000546$) | `20 / 40` (`both=1`, `4` vs. `19`, $p = 0.00260$) | `20 / 40` (`both=4`, `1` vs. `16`, $p = 0.000275$) | `constant[0]` (`24 / 64`, $p = 0.000546$) | `[]` | **`[]`** |
| **`v5-D10`** | `multi_turn_memory` (`40`) | **`20 / 40`** | `0 / 40` (`20` vs. `0`, $p = 1.91 \times 10^{-6}$) | `20 / 40` (`both=6`, `14` vs. `14`, $p = 1.0$) | `20 / 40` (`both=14`, `6` vs. `6`, $p = 1.0$) | `copy_last_stated` (`20 / 40`, `both=14`, `6` vs. `6`, $p = 1.0$) | `true` | **`false` (ties copy control)** |
| **`v5-D10`** | `unknowable_or_impossible` (`24`) | **`0 / 24`** | `24 / 24` (`0` vs. `24`, $p = 1.19 \times 10^{-7}$) | N/A (`0/0`) | N/A (`0/0`) | `constant[0]` (`24 / 24`, $p = 1.19 \times 10^{-7}$) | `false` | **`false`** |
| **`v5-D10`** | **Overall (`64`)** | **`20 / 64`** | `24 / 64` (`20` vs. `24`, $p = 0.652$) | `20 / 40` (`both=6`, `14` vs. `14`, $p = 1.0$) | `20 / 40` (`both=14`, `6` vs. `6`, $p = 1.0$) | `constant[0]` (`24 / 64`, $p = 0.652$) | `['multi_turn_memory']` | **`[]`** |
| **`v5-D25`** | `multi_turn_memory` (`40`) | **`20 / 40`** | `0 / 40` (`20` vs. `0`, $p = 1.91 \times 10^{-6}$) | `20 / 40` (`both=6`, `14` vs. `14`, $p = 1.0$) | `20 / 40` (`both=14`, `6` vs. `6`, $p = 1.0$) | `copy_last_stated` (`20 / 40`, `both=14`, `6` vs. `6`, $p = 1.0$) | `true` | **`false` (ties copy control)** |
| **`v5-D25`** | `unknowable_or_impossible` (`24`) | **`15 / 24`** | `24 / 24` (`0` vs. `9`, $p = 0.00391$) | N/A (`0/0`) | N/A (`0/0`) | `constant[0]` (`24 / 24`, $p = 0.00391$) | `false` | **`false`** |
| **`v5-D25`** | **Overall (`64`)** | **`35 / 64`** | `24 / 64` (`20` vs. `9`, $p = 0.0614$) | `20 / 40` (`both=6`, `14` vs. `14`, $p = 1.0$) | `20 / 40` (`both=14`, `6` vs. `6`, $p = 1.0$) | `constant[0]` (`24 / 64`, `both=15`, `20` vs. `9`, $p = 0.0614$) | `['multi_turn_memory']` | **`[]`** |

---

## 6. Verification Commands Run

```sh
~/.cargo/bin/cargo test -p uor-r4-training --bin chat-grade --features native-rung0
~/.cargo/bin/cargo clippy -p uor-r4-training --bin chat-grade --features native-rung0 -- -D warnings
python3 scripts/check_panel_v4_novelty.py
python3 scripts/check_panel_v5_novelty.py
(cd data/panels && shasum -a 256 -c MANIFEST.sha256)
uor-harness ratchet d23
uor-harness ratchet d25
uor-harness check
```
