# M2 Deterministic Reply-Grading Instrument — R14 Abstention-as-Discrimination Closure (`#2852`, `#2029`)

**Date:** 2026-10-22
**Lab:** Antigravity (`antigravity-11`)
**Roadmap Item:** [`m2-reply-grading-instrument-r14`](../roadmap/items/m2-reply-grading-instrument-r14.md) (`#2852`)
**Milestone:** [#2029 (M1 — Grounded conversation and durable memory)](https://github.com/UOR-Foundation/uor-r4/issues/2029) / [#2030 (M2)](https://github.com/UOR-Foundation/uor-r4/issues/2030)
**Scope Note on `#2029`:** [`#2029` comment 6338175116](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6338175116)
**Predecessor Record:** [`archive/labs/m1-grader-v2-checks-2026-10-14/README.md`](../../archive/labs/m1-grader-v2-checks-2026-10-14/README.md) (`#2412`, `#2433`)
**Governance:** `D28` (deterministic evaluation only), `D31` (instrument false negatives never reported as model incapacity), `D32` (discrete substrate; grading stored replies is evaluation tooling with zero model compute), `R12` (null-model control for every acceptance check), `R14` (`#2373`, M1 reply-quality headline **SUSPENDED** with no target until PM non-vacuity ruling).

---

## 1. Scope & Summary of the Three R14 Requirements

In `docs/harness/pm-rules.md` (§R14), three instrument properties are required before any reply-quality target can be registered on `#2029` / `#2030`:

| R14 Requirement | Status Before `#2852` | Status After `#2852` | Evidence & Implementation |
|---|---|---|---|
| **1. Versioned `v2` check table with frozen positive & negative controls** | **Met** (`#2412`, `#2433`) | **Met** (unchanged) | `crates/uor-r4-training/fixtures/reply-panel-checks-v2.tsv` (`sha256: 8dbc9e5aa02be06b21290cd835aceacc2d5bae99233e5ff7bdc0ec7fff70641d`, `96` checked rows / `136` `BLOCKED` on the 232-row open panel), `reply-panel-positive-controls-v2.tsv` (`sha256: afb7b412b8cb86821335ff44dd29f097fb7ad92e5355b416fb9ac41cbba40249`, `35/35 PASS`), and `reply-panel-negative-controls-v2.tsv` (`sha256: e581e95e4dcd8e0b3915088d078c49062faf0318a28c45d980cc4a7a7e9deb11`, `17/17 FAIL`). All 30 files in `data/panels/MANIFEST.sha256` remain byte-identical. |
| **2. Per-category best-constant and copy-control pairing** | **Met** (`#2412`, `#2433`) | **Met** (unchanged) | `CopyTally`, `Accumulator`, and `Group::headline` in [`crates/uor-r4-training/src/bin/chat-grade.rs`](../../crates/uor-r4-training/src/bin/chat-grade.rs) compute `categories_model_beats_best_constant` and `categories_model_beats_best_control_or_copy` against `max(best_constant, copy_first_stated, copy_last_stated)` by two-sided exact McNemar $p < 0.05$. |
| **3. Abstention-as-discrimination scoring** | **Missing as an explicit joint discriminator** (`abstain_exact` was scored inside single-type categories where `constant[0]` scores `100%`) | **Built & tested in `#2852`** | `CheckKind::abstention_role` and `AbstentionDiscrimination` in [`crates/uor-r4-training/src/bin/chat-grade.rs`](../../crates/uor-r4-training/src/bin/chat-grade.rs) score the $2 \times 2$ confusion table across `abstain_target` (`abstain`, `abstain_exact`) and `answer_target` (`any`, `exact`, `reply_exact`, `exact_text`, `one_of`) rows, paired row-by-row against every constant reply and `max(best_constant, copy_first_stated, copy_last_stated)` across the joint `abstain_target ∪ answer_target` set (`clarify`, `salutation`, `word_count_6`, and `question` are excluded). |

---

## 2. Abstention-as-Discrimination Design (`AbstentionDiscrimination`)

Why single-category abstention scoring is vacuous under R12/R14:
- When `abstain_exact` rows live in a single-type category (`unknowable_or_impossible` on `conversational-v2`/`v3`/`v4`/`v5` or `K-ill-posed` on the open panel), the default constant abstention `constant[0]` (`"I'm not sure. Can you tell me more about what you mean?"`) passes `100%` of the `abstain_exact` rows (`24/24` on `v5`, `32/32` on the open panel under `v2`). No model can ever beat `100%` within an `abstain_exact`-only category.
- Conversely, an indiscriminate abstainer (`constant[0]`) fails `100%` of the factual/memory `answer_target` rows (`0/40` on `v5`, `0/28` on the open panel under `v2`), whereas an indiscriminate copy/guess policy (`copy_last_stated`) passes `20/40` on `v5` memory rows (`false_abstain = 0`) while failing all `24/24` `abstain_exact` rows (`true_abstain = 0`, `false_answer = 24`).

`AbstentionDiscrimination` in [`crates/uor-r4-training/src/bin/chat-grade.rs`](../../crates/uor-r4-training/src/bin/chat-grade.rs) evaluates abstention as **discrimination between `abstain_target` and `answer_target` rows**:

1. **Row Partition (`CheckKind::abstention_role`):**
   - `AbstentionRole::AbstainTarget`: `CheckKind::Abstain`, `CheckKind::AbstainExact`.
   - `AbstentionRole::AnswerTarget`: `CheckKind::Any`, `CheckKind::Exact`, `CheckKind::ReplyExact`, `CheckKind::ExactText`, `CheckKind::OneOf`.
   - Excluded (`None`): `CheckKind::Clarify`, `CheckKind::Salutation`, `CheckKind::WordCount6`, `CheckKind::Question`, and unchecked (`BLOCKED`) rows.
2. **Confusion Counts:**
   - On `abstain_target` rows ($N_{\text{abs}}$):
     - `true_abstain`: reply passes the row's `abstain` / `abstain_exact` check.
     - `false_answer`: reply fails the row's `abstain` / `abstain_exact` check ($N_{\text{abs}} - \text{true\_abstain}$).
   - On `answer_target` rows ($N_{\text{ans}}$):
     - `true_answer`: reply passes the row's factual/memory check.
     - `false_abstain`: reply fails the row's factual/memory check and contains an abstention phrase (`has_abstain_phrase(reply, v2)`).
     - `wrong_answer`: reply fails the row's factual/memory check without an abstention phrase ($N_{\text{ans}} - \text{true\_answer} - \text{false\_abstain}$).
3. **Exact Integer Selectivity Determinant & Paired Null-Model Control:**
   - Selective abstention determinant (exact integer cross-product, zero floating-point division):
     $$\Delta_{\text{sel}} = \text{true\_abstain} \times N_{\text{ans}} - \text{false\_abstain} \times N_{\text{abs}}$$
   - Total correct discrimination decisions:
     $$C_{\text{model}} = \text{true\_abstain} + \text{true\_answer} \quad \text{out of} \quad N_{\text{disc}} = N_{\text{abs}} + N_{\text{ans}}$$
   - Paired row-by-row against each constant reply (`constant[i]`) and against `copy_first_stated` and `copy_last_stated` across all $N_{\text{disc}}$ rows (where copy controls score `false` on `abstain_target` rows and non-`exact` `answer_target` rows, and their exact recall result on `exact` rows).
   - `abstention_discriminates` is `true` iff:
     1. $N_{\text{abs}} > 0$ and $N_{\text{ans}} > 0$ (`applicable == true`),
     2. $\text{true\_abstain} > 0$ and $\text{true\_answer} > 0$,
     3. $\Delta_{\text{sel}} > 0$ (selective abstention rate strictly exceeds false-abstention rate on answerable rows),
     4. $C_{\text{model}}$ strictly beats `best_constant` by two-sided exact McNemar $p < 0.05$, and
     5. $C_{\text{model}}$ strictly beats `best_control_or_copy` ($\max(\text{best\_constant}, \text{copy\_first\_stated}, \text{copy\_last\_stated})$) by two-sided exact McNemar $p < 0.05$.

---

## 3. Host Re-Score of Archived Reply Artifacts & Null Controls (`0` GPU Hours)

### 3.1 Null-Control Verification (`check_only_controls` & Unit Test `r14_abstention_as_discrimination_scoring_and_controls`)

| Control Policy | Panel | $N_{\text{abs}}$ / $N_{\text{ans}}$ ($N_{\text{disc}}$) | `true_abstain` / `false_answer` | `true_answer` / `false_abstain` / `wrong_answer` | $\Delta_{\text{sel}}$ | $C_{\text{policy}}$ vs. `best_control_or_copy` (McNemar $p$) | `abstention_discriminates` |
|---|---|---|---|---|---|---|---|
| `constant[0]` (`"I'm not sure..."`) | `conversational-v5` (`64` rows) | `24` / `40` (`64`) | `24` / `0` | `0` / `40` / `0` | $24 \times 40 - 40 \times 24 = 0$ | `24` vs. `24` ($p = 1.0$) | **`false`** |
| `constant[1]` (`"That sounds nice!..."`) | `conversational-v5` (`64` rows) | `24` / `40` (`64`) | `0` / `24` | `0` / `0` / `40` | $0$ | `0` vs. `24` ($p = 1.19 \times 10^{-7}$) | **`false`** |
| `copy_last_stated` | `conversational-v5` (`64` rows) | `24` / `40` (`64`) | `0` / `24` | `20` / `0` / `20` | $0$ | `20` vs. `24` ($p = 0.652$) | **`false`** |
| `copy_first_stated` | `conversational-v5` (`64` rows) | `24` / `40` (`64`) | `0` / `24` | `20` / `0` / `20` | $0$ | `20` vs. `24` ($p = 0.652$) | **`false`** |
| Synthetic oracle (`abstain` on `unk` + `copy_last + copy_first` on `mem`) | `conversational-v5` (`64` rows) | `24` / `40` (`64`) | `24` / `0` | `40` / `0` / `0` | $+960$ | `64` vs. `24` (`+40`, $p = 1.82 \times 10^{-12}$) | **`true`** |

### 3.2 Archived Model Replies Re-Scored under `checks=v2`

All 13 archived `replies.json` sets from `reply-panel-open-2026-10-09.tar`, `mixture-dose-2026-10-10.tar`, and `ladder.tar` (`tiered-v2-2026-10-05`) were re-scored on the host with `chat-grade grade-replies ... checks=v2`:

| Archive Artifact | Panel | $N_{\text{abs}}$ / $N_{\text{ans}}$ ($N_{\text{disc}}$) | `true_abstain` / `false_answer` | `true_answer` / `false_abstain` / `wrong_answer` | $\Delta_{\text{sel}}$ | $C_{\text{model}}$ (`true_abstain + true_answer`) | `best_control_or_copy` (`constant[0]`) | Paired Discordant (`model_only` vs. `ctrl_only`, `both`, McNemar $p$) | `abstention_discriminates` |
|---|---|---|---|---|---|---|---|---|---|
| **Parent `29m`** (`replies-29m.json`) | Open `232` (`v2`) | `32` / `28` (`60`) | `0` / `32` | `3` / `2` / `23` | $-64$ | **`3 / 60`** | `32 / 60` | `3` vs. `32` (`both = 0`, $p = 4.18 \times 10^{-7}$) | **`false`** |
| **`rp-D0`** (`0%` M-world dose) | Open `232` (`v2`) | `32` / `28` (`60`) | `1` / `31` | `1` / `2` / `25` | $-36$ | **`2 / 60`** | `32 / 60` | `1` vs. `31` (`both = 1`, $p = 1.54 \times 10^{-8}$) | **`false`** |
| **`rp-D10`** (`10%` M-world dose) | Open `232` (`v2`) | `32` / `28` (`60`) | `0` / `32` | `2` / `1` / `25` | $-32$ | **`2 / 60`** | `32 / 60` | `2` vs. `32` (`both = 0`, $p = 6.94 \times 10^{-8}$) | **`false`** |
| **`rp-D25`** (`25%` M-world dose) | Open `232` (`v2`) | `32` / `28` (`60`) | `0` / `32` | `1` / `1` / `26` | $-32$ | **`1 / 60`** | `32 / 60` | `1` vs. `32` (`both = 0`, $p = 7.92 \times 10^{-9}$) | **`false`** |
| **`v5-D0`** (`0%` M-world dose) | `conversational-v5` (`64`) | `24` / `40` (`64`) | `0` / `24` | `5` / `0` / `35` | $0$ | **`5 / 64`** | `24 / 64` | `5` vs. `24` (`both = 0`, $p = 5.46 \times 10^{-4}$) | **`false`** |
| **`v5-D10`** (`10%` M-world dose) | `conversational-v5` (`64`) | `24` / `40` (`64`) | `0` / `24` | `20` / `0` / `20` | $0$ | **`20 / 64`** | `24 / 64` | `20` vs. `24` (`both = 0`, $p = 0.652$) | **`false`** |
| **`v5-D25`** (`25%` M-world dose) | `conversational-v5` (`64`) | `24` / `40` (`64`) | `15` / `9` | `20` / `1` / `19` | $+576$ | **`35 / 64`** | `24 / 64` | `20` vs. `9` (`both = 15`, $p = 0.0614$) | **`false` ($p = 0.0614 \ge 0.05$)** |
| **`replies-v2-29m-B`** | `conversational-v2` + `everyday` (`160`) | `24` / `39` (`63`) | `3` / `21` | `2` / `0` / `37` | $+117$ | **`5 / 63`** | `24 / 63` | `2` vs. `21` (`both = 3`, $p = 6.60 \times 10^{-5}$) | **`false`** |
| **`replies-v2-100m-C`** | `conversational-v2` + `everyday` (`160`) | `24` / `39` (`63`) | `0` / `24` | `4` / `1` / `34` | $-24$ | **`4 / 63`** | `24 / 63` | `4` vs. `24` (`both = 0`, $p = 1.80 \times 10^{-4}$) | **`false`** |
| **`replies-v2-balp-s1`** | `conversational-v2` + `everyday` (`160`) | `24` / `39` (`63`) | `14` / `10` | `11` / `6` / `22` | $+402$ | **`25 / 63`** | `24 / 63` | `11` vs. `10` (`both = 14`, $p = 1.0$) | **`false`** |
| **`replies-v3-29m-B`** | `conversational-v3` + `everyday` (`160`) | `24` / `36` (`60`) | `3` / `21` | `2` / `0` / `34` | $+108$ | **`5 / 60`** | `24 / 60` | `2` vs. `21` (`both = 3`, $p = 6.60 \times 10^{-5}$) | **`false`** |
| **`replies-v3-100m-C`** | `conversational-v3` + `everyday` (`160`) | `24` / `36` (`60`) | `0` / `24` | `1` / `1` / `34` | $-24$ | **`1 / 60`** | `24 / 60` | `1` vs. `24` (`both = 0`, $p = 1.55 \times 10^{-6}$) | **`false`** |
| **`replies-v3-balp-s1`** | `conversational-v3` + `everyday` (`160`) | `24` / `36` (`60`) | `13` / `11` | `7` / `2` / `27` | $+420$ | **`20 / 60`** | `24 / 60` | `7` vs. `11` (`both = 13`, $p = 0.481$) | **`false`** |

### 3.3 Key Diagnostic Findings

1. **Instrument non-vacuity on abstention-as-discrimination:** Every null control (`constant[0]`, `constant[1]`, `constant[2]`, `copy_first_stated`, `copy_last_stated`, `echo_last_user`, `echo_all_user_turns`) scores `abstention_discriminates = false` ($\Delta_{\text{sel}} = 0$), while a selective model that abstains on unknowable rows and answers answerable rows scores `abstention_discriminates = true`.
2. **Model behavior on archived runs:**
   - On the 232-row open panel (`60` discrimination rows under `v2`: `32` `abstain_exact` + `28` `answer_target`), all four archived arms (`29m`, `rp-D0`, `rp-D10`, `rp-D25`) have negative selectivity ($\Delta_{\text{sel}} \in [-64, -32]$) and score `1/60` to `3/60` vs. `32/60` for `constant[0]`.
   - On `conversational-v5` (`64` discrimination rows: `24` `abstain_exact` + `40` `exact`), `v5-D25` exhibits positive selectivity ($\Delta_{\text{sel}} = +576$: `15/24` `true_abstain` vs. `1/40` `false_abstain`, and `20/40` `true_answer`) and reaches `35/64` vs. `24/64` for `constant[0]` (`+11` rows, `20` model-only vs. `9` constant-only), but its two-sided exact McNemar $p = 0.0614$ falls just short of $\alpha = 0.05$ (`abstention_discriminates = false`).
3. **PM / Owner Boundary:** All three technical requirements of R14 (`v2` check table with frozen positive/negative controls, per-category best-constant and copy-control pairing, and abstention-as-discrimination scoring) are now implemented, unit-tested, and run against archived replies in [`crates/uor-r4-training/src/bin/chat-grade.rs`](../../crates/uor-r4-training/src/bin/chat-grade.rs). Per the item's done-when, the PM rules whether the instrument passes R14, and any reply-quality target remains **SUSPENDED** until set by PM/owner decision card.

---

## 4. Verification Commands

```sh
~/.cargo/bin/cargo test -p uor-r4-training --bin chat-grade --features native-rung0
~/.cargo/bin/cargo clippy --no-deps -p uor-r4-training --bin chat-grade --features native-rung0 -- -D warnings
(cd data/panels && shasum -a 256 -c MANIFEST.sha256)
uor-harness ratchet d23
uor-harness ratchet d25
uor-harness ratchet d32
```
