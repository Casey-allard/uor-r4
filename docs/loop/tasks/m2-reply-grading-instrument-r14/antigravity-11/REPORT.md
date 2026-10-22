# REPORT: m2-reply-grading-instrument-r14, antigravity-11

Lab: antigravity. Item: [m2-reply-grading-instrument-r14](../../../../roadmap/items/m2-reply-grading-instrument-r14.md). Order: [ORDER.md](ORDER.md).

References #2852. References #2029.

## What did not work, first

- Scoring `abstain_exact` rows inside single-type categories (`unknowable_or_impossible` on `conversational-v2`/`v3`/`v4`/`v5` or `K-ill-posed` on the open panel) cannot distinguish selective abstention from an indiscriminate constant abstainer: `constant[0]` (`"I'm not sure. Can you tell me more about what you mean?"`) scores `100%` on `abstain_exact` rows (`24/24` on `conversational-v5`, `32/32` on the 232-row open panel under `v2`), so no model can ever beat `constant[0]` within an `abstain_exact`-only category.
- Across all 13 archived reply artifacts re-scored on the host under `checks=v2` (`replies-29m.json`, `rp-D0`, `rp-D10`, `rp-D25`, `v5-D0`, `v5-D10`, `v5-D25`, and the 6 `tiered-v2-2026-10-05` `v2`/`v3` arms), no archived model run achieves `abstention_discriminates = true` at two-sided exact McNemar $p < 0.05$:
  - On the 232-row open panel (`60` discrimination rows under `v2`: `32` `abstain_exact` + `28` `answer_target`), `29m`, `rp-D0`, `rp-D10`, and `rp-D25` score `3/60`, `2/60`, `2/60`, and `1/60` (`selective_abstention_determinant` $\in [-64, -32]$) vs. `32/60` for `constant[0]`.
  - On `conversational-v5` (`64` discrimination rows: `24` `abstain_exact` + `40` `exact`), `v5-D25` achieves positive selectivity (`true_abstain = 15/24`, `false_abstain = 1/40`, `true_answer = 20/40`, `selective_abstention_determinant = +576`) and scores `35/64` vs. `24/64` for `constant[0]` (`+11` rows, discordant `20` model-only vs. `9` constant-only), but its two-sided exact McNemar $p = 0.0614$ does not reach $\alpha = 0.05$ (`abstention_discriminates = false`).

## What changed

- **Scope note on `#2029`:** Posted [`#2029` comment 6338175116](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6338175116) (cross-linked at [`#2852` comment 6338175553](https://github.com/UOR-Foundation/uor-r4/issues/2852#issuecomment-6338175553)) auditing each of the three R14 requirements (`v2` check table with frozen positive/negative controls, per-category best-constant and copy-control pairing, and abstention-as-discrimination scoring), naming all positive (`35`), negative (`17`), constant (`3` default + `13` canned + `12` control), and copy (`copy_first_stated`, `copy_last_stated`) controls, and recording all `MANIFEST.sha256` and `v2` fixture SHA-256 hashes.
- **Abstention-as-discrimination scoring (`crates/uor-r4-training/src/bin/chat-grade.rs`, +368/-0 lines):**
  - Added `AbstentionRole` (`AbstainTarget` for `abstain` and `abstain_exact`; `AnswerTarget` for `any`, `exact`, `reply_exact`, `exact_text`, and `one_of`; `None` for `clarify`, `salutation`, `word_count_6`, and `question`) and `CheckKind::abstention_role`.
  - Added `AbstentionDiscrimination` to `Accumulator`, `Group::headline`, and `check_only_controls` (`report.json` top-level `abstention_discrimination`, `per_tier.overall.headline.abstention_discrimination`, per-tier `abstention_discrimination`, and `check_only_controls.abstention_discrimination`), recording `true_abstain`, `false_answer`, `true_answer`, `false_abstain`, `wrong_answer`, exact integer `selective_abstention_determinant` ($\text{true\_abstain} \times N_{\text{ans}} - \text{false\_abstain} \times N_{\text{abs}}$), and paired two-sided exact McNemar comparisons against `best_constant` and `best_control_or_copy` ($\max(\text{best\_constant}, \text{copy\_first\_stated}, \text{copy\_last\_stated})$) across all $N_{\text{abs}} + N_{\text{ans}}$ discrimination rows.
  - Added unit test `tests::r14_abstention_as_discrimination_scoring_and_controls` verifying that `constant[0]` (`24/64`), `constant[1]` (`0/64`), `copy_last_stated` (`20/64`), and `copy_first_stated` (`20/64`) all fail `abstention_discriminates` on `conversational-v5`, while a selective abstaining+answering control passes `abstention_discriminates` (`64/64` vs. `24/64`, $p = 1.82 \times 10^{-12}$).
- **Lab report (`docs/labs/m2-reply-grading-instrument-r14/README.md`, +99/-0 lines):** Recorded the R14 audit, the `AbstentionDiscrimination` specification, the null-control table, and the host re-score across all 13 archived `replies.json` artifacts.

## Measurements

- **Unit tests (`~/.cargo/bin/cargo test -p uor-r4-training --bin chat-grade --features native-rung0`):** `26 passed; 0 failed; 0 ignored` (including `tests::r14_abstention_as_discrimination_scoring_and_controls`).
- **Host re-score of archived `replies.json` artifacts (`chat-grade grade-replies out=<dir> replies=<file> checks=v2`, `0` GPU hours):**
  - Null controls on `conversational-v5` (`64` discrimination rows: `24` `abstain_exact` + `40` `exact`): `constant[0]` scores `24/64` ($\Delta_{\text{sel}} = 0$, $p = 1.0$, `abstention_discriminates = false`), `copy_last_stated` scores `20/64` ($\Delta_{\text{sel}} = 0$, $p = 0.652$, `abstention_discriminates = false`), `copy_first_stated` scores `20/64` ($\Delta_{\text{sel}} = 0$, $p = 0.652$, `abstention_discriminates = false`), and a selective abstaining+answering oracle scores `64/64` vs. `24/64` ($\Delta_{\text{sel}} = +960$, $p = 1.82 \times 10^{-12}$, `abstention_discriminates = true`).
  - Open panel (`60` discrimination rows under `v2`: `32` `abstain_exact` + `28` `answer_target`): `replies-29m.json` scores `3/60` ($\Delta_{\text{sel}} = -64$), `rp-D0` scores `2/60` ($\Delta_{\text{sel}} = -36$), `rp-D10` scores `2/60` ($\Delta_{\text{sel}} = -32$), and `rp-D25` scores `1/60` ($\Delta_{\text{sel}} = -32$) vs. `32/60` for `constant[0]` (all `abstention_discriminates = false`).
  - `conversational-v5` (`64` discrimination rows): `v5-D0` scores `5/64` ($\Delta_{\text{sel}} = 0$), `v5-D10` scores `20/64` ($\Delta_{\text{sel}} = 0$), and `v5-D25` scores `35/64` (`15/24` `true_abstain`, `1/40` `false_abstain`, `20/40` `true_answer`, $\Delta_{\text{sel}} = +576$) vs. `24/64` for `constant[0]` (discordant `20` model-only vs. `9` constant-only, two-sided exact McNemar $p = 0.0614$, `abstention_discriminates = false`).

## Checks run at the head

- `~/.cargo/bin/cargo test -p uor-r4-training --bin chat-grade --features native-rung0` -> `26 passed; 0 failed; 0 ignored`.
- `~/.cargo/bin/cargo clippy --no-deps -p uor-r4-training --bin chat-grade --features native-rung0 -- -D warnings` -> `0` warnings in `chat-grade.rs`.
- `(cd data/panels && shasum -a 256 -c MANIFEST.sha256)` -> all 30 frozen panel files `OK`.
- `uor-harness ratchet d23`, `uor-harness ratchet d25`, `uor-harness ratchet d32` -> `0` new sites.
- `~/.cargo/bin/cargo run -q --locked -p xtask -- links` -> `930 broken` (unchanged from baseline `930`).
- `uor-harness loop gates --full` and `uor-harness loop report --receipts` -> 6 gates `GREEN` (`fmt-claim-wording`, `fmt-harness`, `claim-wording`, `loop-gates-watch`, `loop-drive`, `git-hooks`), receipts recorded in [`receipts.jsonl`](receipts.jsonl).

## Skills used

- `uor-research-architect`

## Still open

- Non-author review of the `#2029` scope note and this PR, followed by the PM's R14 non-vacuity ruling on whether the instrument passes R14 (tracked on `#2852` and `#2029`).

## Plain summary

A conversational model can cheat an abstention test by replying `"I'm not sure"` to every prompt, which passes every unknowable prompt (`100%`) while failing every answerable question (`0%`). Conversely, a model that always copies the last word from the prompt passes half of the two-value memory questions while failing every unknowable prompt (`0%`). This task adds an abstention-as-discrimination score (`AbstentionDiscrimination`) to the deterministic reply grader (`chat-grade`) that joints unknowable prompts (`abstain_target`) and answerable factual/memory prompts (`answer_target`) into a single $2 \times 2$ discrimination test and compares the model row-by-row against both the best constant reply and the copy controls. Every constant reply and copy control fails this discrimination test, while a model that selectively abstains on unknowable prompts and answers answerable prompts passes it. Re-scoring all 13 archived reply sets shows that `v5-D25` comes closest (`35/64` vs. `24/64` for the constant abstainer, with `15/24` true abstentions and `20/40` true answers), at two-sided exact McNemar $p = 0.0614$.
