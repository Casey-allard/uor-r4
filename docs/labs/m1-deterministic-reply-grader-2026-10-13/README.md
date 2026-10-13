# M1 Deterministic Reply Grader & Diagnostic Re-Score (D28 / D31 / R14) — 13 October 2026

- **Lab:** Antigravity (`antigravity/ag1`)
- **Milestone:** M1 — Conversational & Grounded Reply (`#2029`)
- **Line & D21 Count:** `m1-deterministic-reply-grader`, `0/3` (evaluation tooling; zero model compute, zero GPU)
- **Work Card & Governance:** Card [#2029 comment 6153554602](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6153554602), PM acceptance [#2029 comment 6153595823](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6153595823), Owner audit ruling [#2029 comment 6153686986](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6153686986), Claude independent audit [#2029 comment 6154573629](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6154573629) / [PR #2365 comment 6154574078](https://github.com/UOR-Foundation/uor-r4/pull/2365#issuecomment-6154574078) (`F1`–`F6`) & [PR #2365 comment 6154783818](https://github.com/UOR-Foundation/uor-r4/pull/2365#issuecomment-6154783818) (`F7`–`F10`), PM routing [#2029 comment 6154614848](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6154614848), PM ruling **R14** ([#2029 comment 6154755982](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6154755982), PR `#2373`)
- **Audit Verdict:** Grader code **FIT** (`23/23` unit tests pass; every headline count reproduces byte-for-byte from the archived replies).
- **Headline Status (R14):** Under [R14](../../harness/pm-rules.md#r14--no-target-is-set-or-restated-off-a-vacuous-instrument), **the M1 reply-quality headline is SUSPENDED, with no target.** The deterministic re-score below is recorded as a **diagnostic measurement whose raw headline is vacuous**: on the 88-row checked open panel, `grade-replies`' own built-in default constant `"I'm not sure. Can you tell me more about what you mean?"` scores **`40/88`** (passing all 40 `abstain_exact` rows; paired McNemar $p = 3.8 \times 10^{-11}$ against `D10`'s `1/88`, `discriminates: false`, and `categories_model_beats_best_constant` is empty `[]` for every open-panel category) and **`24/24`** on the `conversational-v5` unknowable rows (`F3`); on the 40-row `conversational-v5` memory panel, `copy_last_stated` and `copy_first_stated` each score **`20/40`**, tying `D10` (`20/40`, `14/20` last-stated vs. `6/20` first-stated, discordant rows `6 vs 6` against `copy_last_stated`) and `D25` (`20/40`, `14/20` last-stated vs. `6/20` first-stated) (`F4`); and under [D31](../../integration/DECISIONS.md#d31---no-numeric-check-rules-out-a-mechanism-instrument-first-solvability-before-authority), `clarify` (`0/27`) and `abstain_exact` (`0/40`) contain **instrument false negatives** (`7` target-behaviour clarification/request rows and `6` bare-salutation rows on `D10`) that are never reported as model incapacity (`F5`).

---

## 1. Summary & Governance Boundary

Under [D28](../../integration/DECISIONS.md#d28---no-neural-comparisons-or-graders-until-owner-declares-end-to-end), all neural graders (specifically the `qwen2.5:7b` Ollama reply grader previously invoked by `chat-grade` and named in `docs/harness/panels/v5-reply.json`) are prohibited in the active project, and any test-set row that cannot be scored by a deterministic check must be marked `BLOCKED` rather than silently passed or proxied by a heuristic. Under [D31](../../integration/DECISIONS.md#d31---no-numeric-check-rules-out-a-mechanism-instrument-first-solvability-before-authority) and [R14](../../harness/pm-rules.md#r14--no-target-is-set-or-restated-off-a-vacuous-instrument), zeros caused by the instrument are never reported as model incapacity, and no M1 reply-quality target exists until the follow-up versioned `v2` check table lands with frozen positive controls, per-category best-constant/copy-control pairing, and abstention-as-discrimination scoring. Under [D32](../../integration/DECISIONS.md#d32---discrete-geometric-substrate-no-floating-point-in-model-computation-including-learning), grading stored replies on the laptop uses zero model compute and zero GPU and is unaffected by the discrete model-compute boundary.

This deliverable:
1. Re-enables `crates/uor-r4-training/src/bin/chat-grade.rs` as an active Rust binary (`--bin chat-grade`, `required-features = ["native-rung0"]`) decoupled from the `#[cfg(any())]`-quarantined pre-D29 CPU/Candle modules (`stack_dialogue`, `stack_tracking`, `baseline_protocol`, `geometric_stack`).
2. Removes the `Grader` Ollama `curl` / `qwen2.5:7b` neural judge and all `gemm` calls (`candle_core::cuda::set_gemm_reduced_precision_f32`), and enforces hard D28/D24/D29 refusals if `grader=`, `ollama_url=`, `chat-grade grade`, or `chat-grade reply` is invoked.
3. Embeds all 5 disjoint frozen deterministic check tables (`conversational-v2-checks.tsv`, `conversational-v3-checks.tsv`, `conversational-v4-checks.tsv`, `conversational-v5-checks.tsv`, and `reply-panel-checks.tsv` — 302 checked row IDs total) and grades every panel row deterministically as `PASS`, `FAIL`, or `BLOCKED` (`reason: "open_ended_no_exact_anchor"`), with `BLOCKED` rows never counted as `PASS`.
4. Preserves `data/panels/reply-panel-checks.tsv` (`sha256: aba96550fb89cbb063b68bae409421b03d2c248e53874e3168d6d7ef40b7ed0c`) and `data/panels/MANIFEST.sha256` byte-identical so the lexical novelty gates (`scripts/check_panel_v4_novelty.py` and `scripts/check_panel_v5_novelty.py`) pass without modification.
5. Records the audited diagnostic measurement of archived `replies.json` artifacts (`D10`, `parent 29M`, `D0`, `D25`, plus the `pointer-ft arm-control` report mislabeled `rpg-D55`) alongside the constant and copy controls (`F1`–`F10`), with zero model compute and zero GPU.

---

## 2. Deterministic Grader Architecture (`chat-grade.rs`)

| Component | Implementation in `crates/uor-r4-training/src/bin/chat-grade.rs` | Rule / Verification |
| :--- | :--- | :--- |
| **D28 neural-grader refusal** | `refuse_neural_grader(&args)` rejects any non-empty `grader=` or `ollama_url=` flag; `chat-grade grade` returns a hard `D28/D29` error. | Tested in `tests::d28_refuses_neural_grader_and_cpu_reply` |
| **D24/D29 CPU-model refusal** | `chat-grade reply` returns a hard `D24/D29` error directing reply generation to the admitted native GPU path. | Tested in `tests::d28_refuses_neural_grader_and_cpu_reply` |
| **D23 / D25 ratchets** | Removed `candle_core::cuda::set_gemm_reduced_precision_f32(false)` and all neural grader code; zero prohibited squashing or dense-product tokens. | `uor-harness ratchet d23` & `uor-harness ratchet d25` PASS |
| **Per-row deterministic verdict** | `Checks::verdict(id, users, reply)` returns `RowVerdict { status: "PASS" \| "FAIL" \| "BLOCKED", check_kind, check_pass, reason }`. | Tested in `tests::deterministic_judge_panel_and_blocked_rows` |
| **BLOCKED accounting** | Unchecked rows receive `status: "BLOCKED"`, `check_kind: None`, `check_pass: None`, `reason: "open_ended_no_exact_anchor"`, `acceptable: None`. `Tally` records `checked_rows`, `check_pass`, `check_fail`, `blocked_rows`, and `blocked_reason`. | `BLOCKED` rows never increment `acceptable` or `check_pass` |

---

## 3. Open Reply Panel (`232` Rows) & `conversational-v5` (`64` Rows) Frozen V1 Classification

### 3.1 Panel Breakdown by Category and Check Kind

| Panel | Category | Total Rows | Checked Rows | Check Kind(s) | `BLOCKED` Rows (`open_ended_no_exact_anchor`) |
| :--- | :--- | ---: | ---: | :--- | ---: |
| **Open Reply (`everyday-32.json`)** | `smalltalk` (`talk-01..08`) | 8 | 0 | — | 8 |
| **Open Reply (`everyday-32.json`)** | `simple_question` (`ask-01..08`) | 8 | 7 | `6 any`, `1 reply_exact` (`ask-06`) | 1 (`ask-04`) |
| **Open Reply (`everyday-32.json`)** | `simple_instruction` (`do-01..08`) | 8 | 0 | — | 8 |
| **Open Reply (`everyday-32.json`)** | `follow_up` (`multi-01..08`) | 8 | 0 | — | 8 |
| **Open Reply (`heldout-200-a/b.json`)** | `heldout_first_turn` (`heldout-000..199`) | 200 | 81 | `10 any`, `4 reply_exact`, `27 clarify`, `40 abstain_exact` | 119 |
| **Open Reply Subtotal** | **All open-reply categories** | **232** | **88** | **`16 any`, `5 reply_exact`, `27 clarify`, `40 abstain_exact`** | **144** |
| **Frozen `conversational-v5`** | `multi_turn_memory` (`conv-v5-mem-01..40`) | 40 | 40 | `40 exact` (recall + forbidden distractor) | 0 |
| **Frozen `conversational-v5`** | `unknowable_or_impossible` (`conv-v5-unk-01..24`) | 24 | 24 | `24 abstain_exact` | 0 |
| **`conversational-v5` Subtotal** | **All v5 categories** | **64** | **64** | **`40 exact`, `24 abstain_exact`** | **0** |

### 3.2 Structure of the Frozen `88 / 144` V1 Split (`F2`, `F6`)

As documented in `data/panels/reply-panel-checks-provenance.tsv` and `docs/labs/reply-panel-deterministic-checks-2026-10-09/README.md`:
- **67 ill-posed held-out rows (`data/panels/heldout-ill-posed-v3-ids.txt`):**
  - **27 `clarify` rows:** requests whose missing referent has a specific name (e.g., `heldout-001` `which recipe`, `heldout-013` `which city`, `heldout-015` `which city`). The reply must contain `?` and a row-specific clarification phrase.
  - **40 `abstain_exact` rows:** requests with unannounced missing material, dangling salutations (`Dear Dr. Patel,`), or constraints without a query (`heldout-016`). The reply must contain a genuine abstention phrase without asserting after `but` or fabricating specifics.
- **21 content-anchored rows (`16 any` + `5 reply_exact`):**
  - **16 `any` rows:** `6` in `simple_question` (`ask-01` chlorophyll/pigment/sunlight, `ask-02` honey/spoil/archaeolog, `ask-03` hydrat/water helps/fluid/energy/temperature, `ask-05` caffeine/relax/breathing/routine/schedule/screen, `ask-07` sun/star/fusion/orbit/light/heat/gravity, `ask-08` breathe/breathing/relax/exercise/talk/slow) and `10` in `heldout_first_turn`.
  - **5 `reply_exact` rows (`F2`):** `ask-06` (farm animals `cow|chicken|pig|horse|sheep|goat|duck|animal` vs. wild-animal distractors `lion|tiger|elephant|penguin|shark|dolphin`), `heldout-022` (Python Min heap `heapq|heapify|heappush|heappop|push` vs. `max heap|maxheap`), `heldout-043` (`96` positive divisors of `27720` vs. half-count distractor `48`), `heldout-103` (`28` sum of primes 1–12 vs. distractors `17|55`), and `heldout-126` (C program `printf|stdio|include|main(` vs. C++/Java/JS/Python distractors `cout|System.out|console.log|print` and `java|javascript|python|cpp`). *(Note: `heldout-120`, `heldout-168`, and `heldout-193` are `BLOCKED` open prompts in the archived panel — a wedding design, meditation, and an astronaut role-play — not `reply_exact` rows.)*
- **144 `BLOCKED` rows (`open_ended_no_exact_anchor`):**
  - `25` rows in `everyday-32.json` across the everyday categories (`smalltalk`: `8`, `simple_question`: `1` [`ask-04` *"What makes a good friend?"*], `simple_instruction`: `8`, `follow_up`: `8`) and `119` open-ended prompts in `heldout-200-a/b.json`.
  - **Audit finding `F6` (input to the `v2` check table follow-up):** While `data/panels/reply-panel-checks.tsv` (`aba96550…`) is kept byte-identical in this PR, Claude's audit identified `BLOCKED` rows in the v1 split that have exact answers or are ill-posed (`heldout-122` $f - g = 8x^2 + x - 10$, `heldout-125` $x = (3 \pm \sqrt{33})/2$, `heldout-092` one of three fixed strings, `heldout-173` story of exactly six words, `heldout-112` no data given / belongs with clarify/abstain) as well as inconsistent anchoring (`heldout-020` pollination, `heldout-033` Robben Island, and `heldout-077` star formation are `BLOCKED` while equally anchorable `ask-07` sun and `heldout-153` elephant trunks are checked). See §6 for the `v2` check table specification.

---

## 4. Control Sets — Zero-Scoring Negative Controls & High-Scoring Default Constant (`F3`, `F8`, `F9`)

Tested in `crates/uor-r4-training/src/bin/chat-grade.rs` (`tests::panel_v5_and_reply_panel_negative_controls`) and `judge_panel` (`F3`, `F8`, `F9`):

| Control Set | Source | Target Rows | Measured Score | Audit / Non-Vacuity Reading |
| :--- | :--- | ---: | ---: | :--- |
| **13 canned replies (`F8`)** | `data/panels/reply-panel-canned.txt` (13 non-blank lines) | 88 checked open-reply rows | **`0/88`** (all 13) | Unit-test negative control (**PASS**). |
| **4 generic control replies (`F8`)** | `data/panels/reply-panel-controls.txt` (4 lines, including `?`) | 88 checked open-reply rows | **`0/88`** (all 4) | Unit-test negative control (**PASS**). |
| **3 adversarial abstentions (`F8`)** | `ADVERSARIAL_ABSTENTIONS` (`chat-grade.rs`, 3 entries) | 24 `conv-v5-unk-*` rows & 40 open `abstain_exact` rows | **`0/24`** & **`0/40`** (all 3) | Unit-test negative control (**PASS**). |
| **40 v5 binding-swap replies** | `data/panels/conversational-v5-swaps.tsv` | 40 `conv-v5-mem-*` rows | **`0/40`** | Unit-test negative control (**PASS**). |
| **40 v5 expected-value oracle replies** | `data/panels/conversational-v5-checks.tsv` | 40 `conv-v5-mem-*` rows | **`40/40`** | Unit-test positive oracle (**PASS**). |
| **Built-in default constant (`F3`, `F9`)** | `"I'm not sure. Can you tell me more about what you mean?"` (`chat-grade.rs` `CONSTANTS[0]`) | 88 checked open-reply rows & 64 `conversational-v5` rows | **`40/88`** on open panel (`40/40 abstain_exact`); **`24/64`** on v5 (`24/24 conv-v5-unk-*`) | **Exposes raw-count vacuity (`F3`, `F9`):** A single canned abstention clears all 40 open-panel `abstain_exact` rows (`40/88` vs. `D10`'s `1/88`; paired McNemar $p = 3.8 \times 10^{-11}$, `discriminates: false`) and all 24 v5 unknowable rows (`24/24`). `categories_model_beats_best_constant` is **empty (`[]`)** for every open-panel category. |
| **Recency / first-stated copy controls (`F4`)** | `copy_last_stated` & `copy_first_stated` (`chat-grade.rs`) | 40 `conv-v5-mem-*` rows | **`20/40`** each (`0/24` on `conv-v5-unk-*`) | **Exposes memory tie (`F4`):** Both copy controls score `20/40`, tying `D10` (`20/40`) and `D25` (`20/40`). See §5.3. |

---

## 5. Diagnostic Measurement of Archived `replies.json` Artifacts (`F1`–`F5`, `F7`, `F10`)

> [!IMPORTANT]
> **Governance & Provenance (`R14`, `D31`, `F1`):**
> - Under **R14** (`#2029` comment `6154755982`), the M1 reply-quality headline is **SUSPENDED** with **no target**. The numbers below are diagnostic measurements only; a raw `x/88` open-panel sum is vacuous because a single constant reply scores `40/88` (`F3`), and `0/27` / `0/40` on `clarify` / `abstain_exact` include instrument false negatives (`F5`, **D31**).
> - **Archive provenance (`F1`):** All graded `replies.json` files come from the iCloud store under `results/deepseek/` (not `docs/history/branch-archive/`):
>   - `results/deepseek/mixture-dose-2026-10-10.tar` — `md5: d31b25ddabaa6c1a161cfd7ae3de0104` (matching `index/mixture-dose-2026-10-10.tsv`)
>   - `results/deepseek/reply-panel-open-2026-10-09.tar` — `md5: 1740b21f181779221458bb8c32008f4f`
> - **Per-file `sha256` digests (`F1`):**
>   - `rp-D10` (`score/rp-D10/replies.json`): `fdb654457e1814a54494ef1532b01bbca2376c8766106df34cb0e9dba1b32510`
>   - `rp-D0` (`score/rp-D0/replies.json`): `abe2aff8460853fc9dc91bece8cba82091339cc98cec395b2420392262183230`
>   - `rp-D25` (`score/rp-D25/replies.json`): `9ddf6b4a72ab0cb57c9ea912cc226a2a1f13d02672dde078505cf00b1e4780aa`
>   - `replies-29m.json` (parent `chat-29m-B-lr5e-4`): `5c4ef2afae44c6061f668368ec0fd98e7e2b163988777c25b2c864060fbac906`
>   - `v5-D10` (`score/v5-D10/replies.json`): `6d5f9e981bfdee2611e797bfcd434b16357d6fa025617f8ff5b2a0c7dbd3be11`
>   - `v5-D0` (`score/v5-D0/replies.json`): `0b1007b24a2cb24558a544381f152b34b4cac828c6daff56568df825ea3ab4dc`
>   - `v5-D25` (`score/v5-D25/replies.json`): `427eed5e59d13ae3824a31c5e031af4109eac12b97002d06b56d94a7f2ee7807`

### 5.1 Open Reply Panel (`232` Rows = `88` Checked + `144` `BLOCKED`) — Headline Suspended (`R14`, `F2`, `F3`, `F7`)

- **Mandatory `F3` Control Context:** Next to every open-panel figure below, `grade-replies`' default constant `"I'm not sure. Can you tell me more about what you mean?"` scores **`40/88 PASS`** (all `40/40 abstain_exact` rows; paired McNemar $p = 3.8 \times 10^{-11}$ vs. `D10`, `discriminates: false`), and `categories_model_beats_best_constant` is **empty (`[]`)** for every open-panel category.

| Checkpoint / Control | Archived Artifact & `sha256` (`F1`) | Checked Rows | **`PASS` (`x/88`)** | `FAIL` (`x/88`) | `BLOCKED` (`x/232`) | Breakdown by Check Kind (`any:16`, `reply_exact:5`, `clarify:27`, `abstain_exact:40`) (`F2`) | Best Constant (`40/88`) & Headline Discrimination (`F3`) |
| :--- | :--- | ---: | ---: | ---: | ---: | :--- | :--- |
| **`Constant: "I'm not sure..."` (`F3`)** | Built-in `CONSTANTS[0]` (`chat-grade.rs`) | 88 | **`40/88`** | `48/88` | `144/232` | `any`: **0/16**, `reply_exact`: **0/5**, `clarify`: **0/27**, `abstain_exact`: **`40/40`** | Reference constant (`40/88`); `24/24` on `v5` unknowable rows |
| **`D10` (active M1 base)** | `score/rp-D10/replies.json` (`fdb65445…b32510`) | 88 | **`1/88`** | `87/88` | `144/232` | `any`: **1/16** (`ask-08`), `reply_exact`: **0/5**, `clarify`: **0/27**\*, `abstain_exact`: **0/40**\* | Loses to constant (`1/88` vs. `40/88`, McNemar $p = 3.8 \times 10^{-11}$, `discriminates: false`, `categories_model_beats_best_constant: []`) |
| **`parent` (`chat-29m-B-lr5e-4`)** | `replies-29m.json` (`5c4ef2af…fbac906`) | 88 | **`2/88`** | `86/88` | `144/232` | `any`: **1/16** (`ask-03`), `reply_exact`: **1/5** (`ask-06`), `clarify`: **0/27**\*, `abstain_exact`: **0/40**\* | Loses to constant (`2/88` vs. `40/88`, `discriminates: false`, `categories_model_beats_best_constant: []`) |
| **`D0` (0% M-world dose)** | `score/rp-D0/replies.json` (`abe2aff8…183230`) | 88 | **`1/88`** | `87/88` | `144/232` | `any`: **1/16** (`ask-05`), `reply_exact`: **0/5**, `clarify`: **0/27**\*, `abstain_exact`: **0/40**\* | Loses to constant (`1/88` vs. `40/88`, `discriminates: false`, `categories_model_beats_best_constant: []`) |
| **`D25` (25% M-world dose)** | `score/rp-D25/replies.json` (`9ddf6b4a…e4780aa`) | 88 | **`0/88`** | `88/88` | `144/232` | `any`: **0/16**, `reply_exact`: **0/5**, `clarify`: **0/27**\*, `abstain_exact`: **0/40**\* | Loses to constant (`0/88` vs. `40/88`, `discriminates: false`, `categories_model_beats_best_constant: []`) |

\* *Under **D31** (`F5`), the `0/27` (`clarify`) and `0/40` (`abstain_exact`) counts include instrument false negatives and are not model negatives; see §5.2.*

- **Correction on `score/rpg-D55/report.json` (`F7`):** The earlier table included a `"D55"` row citing `score/rpg-D55/report.json` (`sha256: 9bd774677e9ee22deae24496e29b9ecf30f8020484bbf422712e427f25e4a7ed`) inside `results/deepseek/reply-panel-open-2026-10-09.tar`. Claude's audit (`F7`) established that `score/rpg-D55/report.json` is a retired `qwen` report — not a `replies.json` — and its embedded `replies` metadata block actually names `pointer-ft-20261009/score/rp-control/replies.json` (`sha256: 1d13d105…2584d`, model `pointer-ft-20261009/.../arm-control/model`). Furthermore, `chat-grade grade-replies` cannot read `report.json` directly because it lacks top-level `panel` and `record` fields; our earlier `2/88` count (`1/16 any`, `1/40 abstain_exact`) was produced by extracting the `rows` array out of `score/rpg-D55/report.json` into a scratch `replies.json` wrapper, which therefore graded **`pointer-ft arm-control`**, not `D55`. The mislabeled `D55` row is dropped from the mixture-dose comparison above until the real `D55` `replies.json` artifact is located.

### 5.2 Diagnostic Breakdown of the `87/88` `FAIL`s on `rp-D10` & Instrument False Negatives (`D31`, `F2`, `F5`, `F10`)

Under **D31**, zeros caused by the instrument are never reported as model incapacity:

| Check Kind | Rows | `PASS` | `FAIL` | Exact Failure Reasons (`chat-grade`) | Audited Attribution (`D31`, `F2`, `F5`, `F10`) |
| :--- | ---: | ---: | ---: | :--- | :--- |
| `any` | 16 | 1 (`ask-08`) | 15 | `15 missing_required_term` | `D10` passes `ask-08` (`breathe\|breathing\|relax\|exercise\|talk\|slow`, `F2`/`F10`). On the 15 failing content questions, `D10` repeats the prompt phrasing (`heldout-064` recursion), emits ungrounded text (`heldout-081` colostrum -> `"natural mammalue located in the American South Wales"`), or omits the required anchor. |
| `reply_exact` | 5 | 0 | 5 | `5 missing_expected_value` | Across the 5 `reply_exact` rows (`ask-06`, `heldout-022`, `heldout-043`, `heldout-103`, `heldout-126`; `F2`/`F10`), `D10` emits bare code fences ```` ``` ```` (`heldout-022` Min heap), repeats the question without computing `96` (`heldout-043` divisors of 27720) or `28` (`heldout-103` sum of primes 1–12), or omits the required answer token (`ask-06` farm animals, `heldout-126` C hello world). |
| `clarify` | 27 | 0 | 27 | `27 missing_question_mark` | **Contains instrument false negatives (`D31`, `F5`):** `CheckKind::Clarify` requires both `?` *and* the exact literal phrase (e.g., `"which recipe"`), rejecting valid probes such as `"What recipe do you mean?"` and `"Could you share the recipe first?"`. On `D10`'s actual replies, **across `clarify` and `abstain_exact`, 7 rows perform the target behaviour (requesting the missing material) and fail on wording only**: `heldout-037` (`"I do not have access to … Please provide me with more specific information"`), `heldout-059`, `heldout-069`, `heldout-073`, `heldout-162` (`"Please provide me with the name of the supplies"`), `heldout-190`, and `heldout-191` (`"Please go ahead and share the text/essay…"`), with `heldout-144` arguable. These are **instrument false negatives**, not model failures. On other rows (e.g., `heldout-015` `"The weather in [city] is quite pleasant..."`, `heldout-049` `"visit the [place]"`), `D10` fills in bracket placeholders instead of clarifying. |
| `abstain_exact` | 40 | 0 | 40 | `40 no abstain phrase` | **Contains instrument false negatives (`D31`, `F5`):** `CheckKind::AbstainExact` rejects valid probes including `"Don't know, sorry."`, `"Nope, I don't know."`, and `"Honestly, I don't know."` (rejected as `"names a specific"` because `Don't`, `Nope`, and `Honestly` are absent from the closed `SENTENCE_OPENERS` list), `"I'm sorry, I don't have that information."` (`"no abstain phrase"`), and `"I don't know, but you could check your coat."` (`"asserts after but"`). On `D10`'s actual replies: (1) the target-behaviour requests for missing material noted above (`heldout-059`, `heldout-069`) fail only because imperative requests are not in `ABSTAIN_PHRASES`; and (2) on **6 bare-salutation rows** (`heldout-006`, `heldout-009`, `heldout-019`, `heldout-102`, `heldout-131`, `heldout-196`), `D10` replied `"Hello! How can I help you today?"`, a defensible response to a greeting that `abstain_exact` cannot accept by construction. Both groups are **instrument false negatives (`D31`)**, not model failures. |

### 5.3 Frozen `conversational-v5` Panel (`64` Rows = `40` `exact` + `24` `abstain_exact`, `0` `BLOCKED`) (`F3`, `F4`)

- **Mandatory `F4` Memory Control Context:** Next to every `multi_turn_memory` figure below, `copy_last_stated` and `copy_first_stated` each score **`20/40`**. Both `D10` (`20/40`) and `D25` (`20/40`) pass **`14/20`** rows where the asked value was stated last and **`6/20`** where it was stated first. Paired against `copy_last_stated`, the discordant rows are **`6 vs 6`** — statistically indistinguishable from recency-copying. Because `grade-replies`' current headline rule pairs only against the three constant controls (which score `0/40` on memory), `"multi_turn_memory beats best constant"` carries this explicit caveat: neither `D10` nor `D25` beats the `copy_last_stated` (`20/40`) or `copy_first_stated` (`20/40`) control.
- **Mandatory `F3` Unknowable Control Context:** Next to every `unknowable_or_impossible` figure below, `grade-replies`' default constant `"I'm not sure. Can you tell me more about what you mean?"` scores **`24/24 PASS`** on `conv-v5-unk-01..24`.

| Checkpoint / Control | Archived Artifact & `sha256` (`F1`) | `multi_turn_memory` (`40 exact`) & Position Split (`F4`) | `unknowable_or_impossible` (`24 abstain_exact`) (`F3`) | Total Deterministic `PASS` (`x/64`) | Paired vs. `copy_last_stated` (`20/40`) (`F4`) |
| :--- | :--- | :--- | :--- | ---: | :--- |
| **`Control: copy_last_stated` (`F4`)** | Built-in `copy_last_stated` (`chat-grade.rs`) | **`20/40 PASS`** (`20/20` last-stated, `0/20` first-stated) | **`0/24 PASS`** | **`20/64`** | Reference recency-copy control (`20/40`) |
| **`Control: copy_first_stated` (`F4`)** | Built-in `copy_first_stated` (`chat-grade.rs`) | **`20/40 PASS`** (`0/20` last-stated, `20/20` first-stated) | **`0/24 PASS`** | **`20/64`** | Reference primacy-copy control (`20/40`) |
| **`Control: Constant "I'm not sure..."` (`F3`)** | Built-in `CONSTANTS[0]` (`chat-grade.rs`) | **`0/40 PASS`** (`40 missing_expected_value`) | **`24/24 PASS`** | **`24/64`** | Reference constant abstention control (`24/24` on unknowable) |
| **`D10` (active M1 base)** | `score/v5-D10/replies.json` (`6d5f9e98…3be11`) | **`20/40 PASS`** (`14/20` last-stated, `6/20` first-stated; `17 contains_forbidden_term`, `3 missing_expected_value`) | **`0/24 PASS`** (`24 no abstain phrase`) | **`20/64`** | Ties `copy_last_stated` (`20/40` vs. `20/40`; discordant rows `6 vs 6`, indistinguishable from recency-copying) |
| **`D0` (0% M-world dose)** | `score/v5-D0/replies.json` (`0b1007b2…b4dc`) | **`5/40 PASS`** (`7 contains_forbidden_term`, `28 missing_expected_value`) | **`0/24 PASS`** (`23 no abstain phrase`, `1 contains_forbidden_term`) | **`5/64`** | Below `copy_last_stated` (`5/40` vs. `20/40`) |
| **`D25` (25% M-world dose)** | `score/v5-D25/replies.json` (`427eed5e…7807`) | **`20/40 PASS`** (`14/20` last-stated, `6/20` first-stated; `15 contains_forbidden_term`, `5 missing_expected_value`) | **`15/24 PASS`** (`9 no abstain phrase`) | **`35/64`** | Memory ties `copy_last_stated` (`20/40` vs. `20/40`, `14/20` last vs. `6/20` first); unknowable `15/24` vs. constant `24/24` |

---

## 6. Follow-Up Work: Versioned `v2` Check Table (`F5`, `F6`, `R14`)

Per **R14** (`#2029` comment `6154755982`) and PM routing (`#2029` comment `6154614848`), after `#2365` merges, Antigravity will draft (and Claude will review) a separate follow-up PR introducing a versioned `v2` check table while keeping `data/panels/reply-panel-checks.tsv` (`sha256: aba96550fb89cbb063b68bae409421b03d2c248e53874e3168d6d7ef40b7ed0c`) byte-identical:
1. **Frozen must-pass positive-control set (`F5`):** Legitimate abstentions (e.g., `"Don't know, sorry."`, `"Nope, I don't know."`, `"Honestly, I don't know."`, `"I'm sorry, I don't have that information."`), imperative requests for missing material (`heldout-037`, `-059`, `-069`, `-073`, `-162`, `-190`, `-191`), bare-salutation greetings (`heldout-006`, `-009`, `-019`, `-102`, `-131`, `-196`), and natural clarifications (`"What recipe do you mean?"`, `"Could you share the recipe first?"`), mirroring the must-fail adversarial set.
2. **Wider accepted behaviour class (`F5`):** Expand `clarify` and `abstain_exact` in the `v2` table to pass the frozen positive-control set while continuing to reject the adversarial and canned controls.
3. **`F6` row-classification corrections in `v2`:**
   - `heldout-122`: exact algebraic answer ($f - g = 8x^2 + x - 10$).
   - `heldout-125`: exact quadratic roots ($x = (3 \pm \sqrt{33})/2$).
   - `heldout-092`: exact membership in one of three fixed strings.
   - `heldout-173`: exact six-word length check.
   - `heldout-112`: no data given (ill-posed; reclassify from `BLOCKED` to clarify/abstain).
   - Consistent factual anchoring across `heldout-020` (pollination), `heldout-033` (Robben Island), and `heldout-077` (star formation) relative to checked `ask-07` (sun) and `heldout-153` (elephant trunks).
4. **Non-vacuous headline rule (`F3`, `F4`, `R14`):** Score per category against the best of **both the constant controls AND the copy controls** (`copy_last_stated`, `copy_first_stated`), with abstention scored as **discrimination** (abstaining on ill-posed rows AND answering well-posed ones), and re-run every constant and copy control against the `v2` table.

---

## 7. Verification Commands Run

```sh
CARGO_TARGET_DIR=local/grader-target ~/.cargo/bin/cargo test -p uor-r4-training --bin chat-grade
CARGO_TARGET_DIR=local/grader-target ~/.cargo/bin/cargo clippy -p uor-r4-training --bin chat-grade -- -D warnings
uor-harness ratchet d23
uor-harness ratchet d25
python3 scripts/check_panel_v4_novelty.py
python3 scripts/check_panel_v5_novelty.py
uor-harness onboard --repo "$PWD"
uor-harness check
```
