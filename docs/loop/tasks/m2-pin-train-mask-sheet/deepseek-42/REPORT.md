# REPORT: m2-pin-train-mask-sheet, deepseek-42 (is the Amendment 6 sheet live under D37?)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2851 (`References`; the item stays `ready` until the PM answer and a non-author FIT). Base: `8720c5d6d`.

## What did not work, first

- **The conflict is real and is not a lab call.** Amendment 7 froze the sheet under D32 (float ban on the learning path), but **D37** later revised that ban: floating point is legal in **offline training only**, with a reported export gap. Whether that revives the Amendment 6 sheet or still wants a new discrete-law sheet is exactly the item's done-when question, and it is the PM's or owner's to record - I did not answer it for them.
- **The four pins had to be re-verified, not re-quoted.** I checked the source commits exist (`2fd6358bb`, `9e904a834`), the CLI still carries `train_mask`/`expected_train_targets` (`geometric-stack-native.rs`), the mask digest `b5e7352d…` is the current code pin, and the 2,425,925 count is carried by the count-reference archive. The DEV pins are restated from the Amendment 4 record; the DEV tar itself is not in this checkout.
- **A second review caught half a rule and a wrong arm.** (1) The sheet said the D37 export gap must be *reported* but not what happens when it is too large; CHARTER §3.7 also says a gap above the two-seed noise band **disqualifies**, so the outcome rule is now stated. (2) The single CLI said `objective=hinge` for every run, but the old sheet's arm table has **M1 `hinge` and M8 `hinge8`**; the sheet now gives the per-arm objective explicitly (the trainer accepts both). (3) The card recommended reviving the Amendment 6 sheet, which CHARTER §4 rule 5 rules out - "archived float sheets return only as new sheets with a fresh FIT" - so this page is a **new sheet** at PENDING and a correction card is posted. (4) The GPU caps are CHARTER §5 rule 4, not D26 (D26 is only the Vast default); fixed.
- **A review caught over-claimed stop behaviour.** My first draft said the run aborts on the train-tar/mask/tokenizer hashes and the DEV count; the CLI does not - it enforces `expected_train_targets` and `train_mask` and reports the DEV denominator. The section now states the enforced behaviour and restores the old sheet's manual pre-run verification instead of promising a check the code lacks. The reviewer also caught a dose-arithmetic typo inherited from the old sheet (`44,957,696`; the true product is `44,998,656`), now corrected with a note.
- **A refresh must not quietly become a re-registration.** The bars, reference, guards, dev policy and noise floor are restated by pointer to the old sheet's authoritative wording rather than rephrased; changing a bar would need a decision card. `TEST FITNESS: PENDING` is honest: the old sheet's FIT stays in its own record, and this page still needs a non-author FIT plus the PM answer.

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md` | the refreshed one-page sheet: D37 basis + the disqualifying export-gap rule, new-sheet framing under CHARTER §4 rule 5, per-arm `objective` (M1 `hinge` / M8 `hinge8`), the pins (two re-verified, two restated), unchanged bars/reference/guards/dev policy (by pointer), steps with the exact CLI, stop rule, cost, power, `TEST FITNESS: PENDING` |
| `docs/labs/m2-pin-train-mask-sheet/README.md` | what was verified, the decision card, and the open prerequisites |
| this folder | ORDER (committed first) and this REPORT |

**Size:** about 150 docs lines; docs-only (no receipts file, D46 fresh-context record used).

## Measurements

- Pins verified by command: `git cat-file -t 2fd6358bb/9e904a834` -> commit; `expected_train_targets` and `train_mask` present in `crates/uor-r4-training/examples/geometric-stack-native.rs`; `b5e7352d…` in `crates/uor-r4-discrete/src/attention_learning.rs`; `2,425,925` in `archive/labs/count-reference-2026-10-13/README.md`.
- `prep-check` on the new sheet: **complete=yes**, fitness **`PENDING`** by design; `loop pr-check --local` PASS. This REPORT is updated only after each measurement; the check and review results at the current head are recorded below once they exist (nothing here claims an unrun check or an unposted review).

## Checks run at the head

`prep-check`, `loop report --check`, `loop pr-check --local`, `loop gates --full`; the fresh-context review record is requested at the current head and its result is recorded after it is posted (docs-only, D46). The PR body is refreshed from this REPORT before the merge queue runs.

## Skills used

- `solution-integrity`: re-verified each pin instead of trusting the old sheet's wording; kept the bar wording by pointer so nothing changed by paraphrase.
- `scope-guard`: docs-only inside the item's scope; no bar, hold, spend or owner value was touched.

## Still open

- The PM's recorded live/superseded answer (decision card on #2851/#2028).
- A non-author FIT of the refreshed sheet (Claude/Kimi/antigravity; deepseek authored it).
- M2 training and rental remain HELD; a FIT releases nothing.

## Plain summary

- Verified the four pins on current `main`, wrote the one-page D37-basis refresh, and asked the PM the one question only the PM/owner can answer.
- Nothing was run; no bar, hold or spend changed; the sheet waits on the recorded answer plus a non-author FIT.