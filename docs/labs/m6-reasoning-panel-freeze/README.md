# M6 Reasoning and Coding Panel & Oracles (`m6-reasoning-panel-freeze` #2866)

- **Status:** Staged and verified on finite host controls ([`crates/uor-r4-harness/src/panel_m6_reasoning.rs`](../../../crates/uor-r4-harness/src/panel_m6_reasoning.rs), [`crates/uor-r4-harness/tests/panel_m6_reasoning.rs`](../../../crates/uor-r4-harness/tests/panel_m6_reasoning.rs), [`docs/harness/panels/m6-reasoning.json`](../../harness/panels/m6-reasoning.json) with `"frozen": false` pending owner bar confirmation on [`#2028` comment `6344266280`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6344266280) and non-author `TEST FITNESS: FIT`); GPU qualification (`m6-qualify-reasoning-coding` `#2865`) remains HELD pending M1 trunk training (`m1-trunk-assembly-and-training` `#2845`) and Casey's typed D40 release.
- **Milestone:** M6 (`#2034`). **Item:** [`m6-reasoning-panel-freeze`](../../roadmap/items/m6-reasoning-panel-freeze.md) (`#2866`). **Lab:** antigravity.
- **Design note:** [`#2034` comment `6344264079`](https://github.com/UOR-Foundation/uor-r4/issues/2034#issuecomment-6344264079), [`#2866` comment `6344265209`](https://github.com/UOR-Foundation/uor-r4/issues/2866#issuecomment-6344265209). **Bar decision card:** [`#2028` comment `6344266280`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6344266280).

## 1. Seeded Panel Synthesis and Content Seal

The `m6-reasoning` acceptance panel is synthesized deterministically in pure Rust by `uor_r4_harness::panels::m6_reasoning::generate_frozen_m6_panel()` from canonical seed `M6_PANEL_FROZEN_SEED_V1 = 0x004D_365F_5245_4153` (`"M6_REAS"` in ASCII), wired into [`crates/uor-r4-harness/src/panels.rs`](../../../crates/uor-r4-harness/src/panels.rs). By generating the exact JSONL bytes in-process from the canonical seed and sealing their SHA-256 digests in [`docs/harness/panels/m6-reasoning.json`](../../harness/panels/m6-reasoning.json) and [`docs/reference/storage.md`](../../reference/storage.md), no bulk corpus file is committed to git (CHARTER §3.13):

| Stream | Generator call | Row count | Byte length | SHA-256 content hash |
|---|---|---:|---:|---|
| Unscored evaluation prompts (`M6UnscoredPromptRow`) | `generate_frozen_m6_panel().to_canonical_jsonl_bytes(false)` | `240` | `79,625 B` | `5cf15b8c97c7d6528e5dbff43f0b21df6de6b98e8845228bc10b7ea4439e7dc7` |
| Sealed oracle specification (`M6FrozenRow`) | `generate_frozen_m6_panel().to_canonical_jsonl_bytes(true)` | `240` | `206,197 B` | `a52f3b0333a2832e1e3371c09954703cf3fa6b4364c22f9d4f3606a0f3600f0f` |

## 2. Three Task Families and Deterministic Oracles (D28 Compliant)

Every row is graded by `grade_m6_row` / `grade_m6_panel` in [`crates/uor-r4-harness/src/panel_m6_reasoning.rs`](../../../crates/uor-r4-harness/src/panel_m6_reasoning.rs) with zero neural comparator, teacher, or judge (CHARTER §3.10, D28):

1. **`Compositions` (`80` rows, `m6-comp-000` .. `m6-comp-079`) — Unseen multi-step compositions:**
   - Synthesized from SplitMix64 stream `seed ^ 0x434F_4D50_5F56_3100` over 4- to 5-digit integers in `[1_200, 78_999]` with at least `2` decimal carries (`count_decimal_carries(a, b) >= 2`), at least `1` decimal borrow on the actual subtracted operand (`a + b` for register chains; `(a + b + step_delta) * scale` for versioned memory chains), and verified zero leakage of `expected_intermediate` or `expected_final` in the prompt text.
   - `40` rows (`m6-comp-000` .. `m6-comp-039`) are 3-step register chains (`T1 = A + B` into `R0`, `T2 = R0 - C` into `R1`, `OUT = R1 + D`).
   - `40` rows (`m6-comp-040` .. `m6-comp-079`) are 3-step versioned addressed-memory chains (`R0@initial = A + B`, `R0@previous = R0@initial + step_delta`, `R0@current = (R0@previous * scale) - C`, querying `R0@initial` or `R0@previous` alongside `R0@current` so even `Initial` queries require computing the multi-carry sum `A + B`).
   - **Oracle (`grade_composition_row`):** Requires exact canonical integer formatting (`parse_canonical_i64`, rejecting leading `+`, leading zeroes, duplicate keys, or trailing junk fields) on **both** the intermediate register/memory state (`INT=<i64>`) and the final output (`OUT=<i64>`). Corrupting `INT` while keeping `OUT` exact fails with `M6OracleDiagnostic::IntermediateStateMismatch`.
2. **`AlteredConstraints` (`80` rows, `m6-alt-000` .. `m6-alt-079`) — Altered constraints:**
   - Synthesized from SplitMix64 stream `seed ^ 0x414C_5443_5F56_3100` across 4 sub-families (`20` rows each), every row verified to satisfy `expected_answer != default_habit_answer`:
     - `NonDecimalRadix` (`20` rows): 3-digit addition in base $B \in \{7, 8, 9, 11, 12, 13, 16\}$ using digits `1..min(B, 10)` with a mandatory base-$B$ carry so a base-10 habit reading yields a distinct wrong numeral (`default_habit_answer`).
     - `ReboundPrecedence` (`20` rows): expressions `a [+] b [*] c` where `[+]` binds tighter than `[*]` (`(a + b) * c != a + (b * c)`).
     - `ModularStepCap` (`20` rows): 2-step modular transitions (`if S1_raw mod M == forbidden_rem, set S1 = S1_raw + fallback_delta, else S1 = S1_raw; OUT = (S1 + c) mod M`) balanced into `10` branch-taken rows (`(a + b) mod M == forbidden_rem`) and `10` branch-not-taken rows (`(a + b) mod M != forbidden_rem`), so both unconditional-add and unconditional-skip heuristics score only `10 / 20` and fail the other `10` rows with `HabitCaptureError`.
     - `FeasibilityConstraint` (`20` rows): balanced into `10` unsatisfiable rows requiring `ABSTAIN:UNSATISFIABLE` and `10` satisfiable rows with a unique integer solution, sharing identical prompt wording across both sub-templates (`unsigned register R0` and `even-lattice closure`) so a surface keyword rule (`prompt.contains("feasibility") -> ABSTAIN:UNSATISFIABLE`) scores only `10 / 20` on the sub-family (`0 / 10` on satisfiable feasibility rows, failing with `FalseAbstentionOnSatisfiableRow`).
   - **Oracle (`grade_altered_constraint_row`):** Requires exact `expected_answer` on the `70` satisfiable rows (explicitly classifying `default_habit_answer` as `M6OracleDiagnostic::HabitCaptureError` on the `60` arithmetic/precedence/modular rows and `ABSTAIN:UNSATISFIABLE` as `FalseAbstentionOnSatisfiableRow` on all `70` satisfiable rows), and exact `ABSTAIN:UNSATISFIABLE` on the `10` unsatisfiable rows.
3. **`CodeTasks` (`80` rows, `m6-code-000` .. `m6-code-079`) — Executable coding tasks:**
   - Synthesized from SplitMix64 stream `seed ^ 0x434F_4445_5F56_3100` across 4 parameterized integer algorithm families (`20` rows each): `AffinelyScaledClampLoop`, `RadixDigitWeightedSum`, `BoundedRingStep`, and `PiecewiseParityRecurrence`, verified so none of the 4 expected output integers appear as literal tokens in the prompt.
   - **Safe execution & oracle (`SandboxProgram`, `grade_code_task_program`, `grade_code_task_row`):**
     - Candidate programs execute inside `SandboxProgram::execute`, a total 8-register integer machine with a hard instruction cap (`MAX_SANDBOX_STEPS = 256`), checked integer arithmetic (`checked_add/sub/mul/div/rem`), zero heap allocation, and zero filesystem/network/process I/O.
     - Each row also provides `SandboxProgram::to_rust_source`, emitting a standalone Rust module (`pub fn solve(x: i64) -> i64` and `#[test] fn verify_vectors()`) for offline `rustc --test` / `cargo test` verification.
     - Passing requires matching all `4 / 4` held-out `(input, expected_output)` test vectors in exact canonical format (`V0=<y0>;V1=<y1>;V2=<y2>;V3=<y3>`) within `256` steps with zero runtime traps.

## 3. Overlap Exclusion and Lawful Count-Table Baseline Noise (CHARTER §3.10, §4.4)

- **Overlap exclusion (`M6ReasoningPanel::verify_zero_overlap` and `panel_overlap_exclusion_and_corpus_disjointness`):**
  1. `FORBIDDEN_CONTROL_PAIRS` (`18` pairs) and `FORBIDDEN_CONTROL_TRIPLES` (`30` triples) explicitly exclude every operand tuple used in the finite CPU controls of [`crates/uor-r4-integer/src/m6_exact_arithmetic.rs`](../../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs) and [`crates/uor-r4-integer/src/m6_ablation.rs`](../../../crates/uor-r4-integer/src/m6_ablation.rs).
  2. Every row carries the `[M6-PANEL-V1::<family>::<row_id>]` domain header, verified absent from all other panel registry files in `docs/harness/panels/*.json`, and every row is verified not to leak its target integers in the prompt text. (Full byte-level scan against external training corpora is required at M6 qualification time when a training corpus is bound.)
  3. Independent seeds (`M6_PANEL_FROZEN_SEED_V1` vs `M6_PANEL_FROZEN_SEED_V1 ^ 0xDEAD_BEEF_CAFE_BABE`) are verified to have `0 / 240` prompt overlap.
- **Lawful non-neural count-table baseline (`M6CountTableBaseline`, `evaluate_count_table_baseline_noise`):**
  - Keyed strictly on the unscored `M6UnscoredPromptRow` surface text (`context_key(&M6UnscoredPromptRow)`) so the predictor has zero type-level access to `M6RowSpec` or feasibility polarity, and evaluated across `8` independent full 240-row seeded training draws (`num_seeds = 8`, no subsampling):
    - **Unconditioned numeric count-table scores:** `[3, 3, 1, 2, 1, 1, 1, 1]` (`1..=3 / 240`, arising from small-modulus residue collisions on `ModularStepCap` rows; `0 / 80` on `Compositions` and `0 / 80` on `CodeTasks`).
    - **Prompt-surface context-keyed count-table scores:** `[11, 11, 10, 12, 10, 10, 11, 11]` (min `10 / 240`, max `12 / 240`, range `2` rows, measured half-band **`+-1 rows`**).
    - **Family-conditioned modal count-table / best-constant ceiling:** `10 / 240` (`4.17%`, hitting the `10` `ABSTAIN:UNSATISFIABLE` rows in `AlteredConstraints`, `0 / 70` satisfiable `AlteredConstraints`, `0 / 80` `Compositions`, `0 / 80` `CodeTasks`).

## 4. Research Notebook Record (Six-Column Format for `m6-reasoning-coding`)

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| 2026-10-22 | `m6-reasoning-panel-freeze` (`#2866`): deterministic 240-row `m6-reasoning` generator (`0x004D365F52454153`), 3-family D28 oracles (`Compositions`, `AlteredConstraints`, `CodeTasks`), overlap exclusion against `m6_exact_arithmetic.rs` controls (`18` pairs, `30` triples), and 8-seed prompt-only `M6CountTableBaseline` noise evaluation, CPU finite check. | `240 / 240` oracle positive controls (`1.0000`), `0 / 240` negative controls (`60 / 60` arithmetic/precedence/modular habit-capture errors + `10 / 10` false-abstention habit errors flagged); prompt-only count-table context baseline `10..=12 / 240` (`+-1 rows` noise), modal ceiling `10 / 240` (`0.0417`); unscored SHA-256 `5cf15b8c…`, sealed SHA-256 `a52f3b03…`. | Proposed bar `>= 168 / 240` (`>= 56 / 80` per family) across `>= 2` seeds ([`#2028` comment `6344266280`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6344266280)). | **pending** (staged for owner bar confirmation & non-author FIT) | [`panel_m6_reasoning.rs`](../../../crates/uor-r4-harness/src/panel_m6_reasoning.rs), [`m6-reasoning.json`](../../harness/panels/m6-reasoning.json) |

---

## 5. Pre-Registration and Fitness Review Status (`#2866` / `#2034`)

### Bar

- **Staged in [`docs/harness/panels/m6-reasoning.json`](../../harness/panels/m6-reasoning.json) (`"frozen": false`) and proposed on [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6344266280) for owner confirmation (CHARTER §5.2):**
  1. `overall-pass-240`: `>= 168 / 240` (`70.0%`, `+156` rows above the `12 / 240` prompt-only count-table max score, `156x` the measured `+-1` row noise band) across `>= 2` seeds.
  2. `compositions-80`: `>= 56 / 80` (`70.0%` exact canonical intermediate + final match on unseen 3-step compositions).
  3. `altered-constraints-80`: `>= 56 / 80` (`70.0%`, requiring `>= 46 / 70` on satisfiable altered-constraint rows plus `10 / 10` `ABSTAIN:UNSATISFIABLE` discrimination).
  4. `code-tasks-80`: `>= 56 / 80` (`70.0%` passing all 4 held-out test vectors in the 256-step sandbox and `rustc --test` verifier).

### Base artifact

- **Panel generator & deterministic grader:** `uor_r4_harness::panels::m6_reasoning` ([`crates/uor-r4-harness/src/panel_m6_reasoning.rs`](../../../crates/uor-r4-harness/src/panel_m6_reasoning.rs)) and [`docs/harness/panels/m6-reasoning.json`](../../harness/panels/m6-reasoning.json).
- **Candidate model artifact (for `#2865`):** D11 discrete artifact (`uor-r4.lut-discrete/1`) exported from the M1 trunk (`m1-trunk-assembly-and-training` `#2845`), evaluated with [`crates/uor-r4-integer/src/m6_exact_arithmetic.rs`](../../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs) and [`crates/uor-r4-integer/src/m6_ablation.rs`](../../../crates/uor-r4-integer/src/m6_ablation.rs).

### Data split

- **Evaluation split:** `240` rows (`80` `Compositions`, `80` `AlteredConstraints`, `80` `CodeTasks`) from canonical seed `0x004D365F52454153`:
  - Unscored evaluation prompt stream: `79,625` bytes, SHA-256 `5cf15b8c97c7d6528e5dbff43f0b21df6de6b98e8845228bc10b7ea4439e7dc7`.
  - Sealed oracle specification stream: `206,197` bytes, SHA-256 `a52f3b0333a2832e1e3371c09954703cf3fa6b4364c22f9d4f3606a0f3600f0f`.
  - Verified zero overlap (`verify_zero_overlap()`) with `FORBIDDEN_CONTROL_PAIRS` (`18` pairs), `FORBIDDEN_CONTROL_TRIPLES` (`30` triples), other `docs/harness/panels/*.json` registries, and independent seeds.

### Steps

1. Run `cargo test --locked -p uor-r4-harness --test panel_m6_reasoning` (`7/7` integration tests) and `cargo test --locked -p uor-r4-harness --lib panel_m6` (`3/3` unit tests) to verify byte-exact SHA-256 seals (`5cf15b8c…` and `a52f3b03…`), overlap exclusion, positive/negative oracle controls (`240/240` vs `0/240`), feasibility keyword-leakage defense (`10/20`), `ModularStepCap` branch balance (`10/10`), and the 8-seed prompt-only `M6CountTableBaseline` noise band (`+-1 rows`).
2. Obtain other-vendor non-author `TEST FITNESS: FIT` review on PR `#2977` and owner confirmation of the numeric bars on [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6344266280) before flipping `"frozen": true` and `status: done`.
3. Keep M6 GPU qualification (`m6-qualify-reasoning-coding` `#2865`) HELD until M1 trunk training (`#2845`), owner bar confirmation on `#2028`, PM dispatch, and Casey's typed D40 release.

### Stop conditions

- Any change to `unscored_sha256` (`5cf15b8c…`) or `sealed_oracle_sha256` (`a52f3b03…`), any overlap with `FORBIDDEN_CONTROL_PAIRS` / `FORBIDDEN_CONTROL_TRIPLES`, or any failure of the positive/negative oracle controls halts evaluation immediately.

### Cost and minimum test power

- **Cost:**
  - `preparation`: `\$0.00` (host-only deterministic Rust generator, oracles, and count-table baseline noise evaluation).
  - `run`: `\$0.00` in this PR (this item stages the panel and releases no GPU run).
  - `controls`: `240` positive oracle controls, `240` negative/perturbed/habit-capture controls, `240` constant-abstention controls, `240` keyword-abstention controls, `20` modular branch-balance controls, and `8` full seeded `M6CountTableBaseline` draws.
  - `evaluation`: deterministic integer and 256-step sandbox grading (`grade_m6_panel`).
  - `retries`: `0`.
  - `storage`: `0 B` bulk binary corpus committed to git; `79,625 B` unscored and `206,197 B` sealed JSONL streams reproduced in-process from seed `0x004D365F52454153`.
- **Minimum test power (D22 §3):**
  - `240` fresh rows (`>= 200` D22 §3 floor), `80` rows per family, evaluated across `>= 2` independent seeds against the measured `+-1 rows` noise band and paired McNemar discordant floor $F_I = \max(16, \lceil 2\sqrt{b + c}\rceil)$.

TEST FITNESS: PENDING (awaiting non-author review on PR #2977 and owner bar confirmation on #2028)
