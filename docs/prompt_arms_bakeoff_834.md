# Prompt-conditioned evidence arms — bake-off instrument (#834)

- **Issue:** #834 — "research/#822-B: fit prompt-conditioned evidence arms against
  causal and equal-budget controls" (item B of S1 tracker #822, programme #820).
- **Parent tracker:** #822 (S1 — persistent prompt-conditioned predictive state).
- **Date:** 2026-08-20.
- **Status:** The **binding cheap instrument** and its five Verification items are
  frozen and green (reference-only). A **teacher-grounded run** on the #833 canonical
  bundle has now been executed (§6.1): for the arms fittable against the current
  artifact (current-scoring, longer-local-context) the verdict is
  **`NO PROMPT-CONDITIONING ARM ESTABLISHED`** — the deployed model is suffix-local.
  The three #835 Ψ-family arms are not lowered into the deployed artifact (built by #836).
  A follow-up **Ψ segment-lane reference-arm re-test** (§6.2) then found that whole-prompt
  content **is** predictive beyond the suffix (Ψ **+17.5‰**, CI [15.9, 19.0]; follows
  **10/4,722** minimal pairs where the suffix baseline follows 0) — a modest but real
  positive that warrants building #835/#836. This record is append-only.
- **Claim language:** normative per [`docs/formal_vocabulary.md`](formal_vocabulary.md).
  Every labeled statement carries one claim class (**Definition**, **Objective**,
  **Guarantee**, **Assumption**, **Empirical Criterion**) and, for **Guarantee** /
  **Empirical Criterion**, a status (**Structural**, **Witnessed**, **Empirical**,
  **Assumed**, **Unproven**).
- **Evidence files:** the instrument `crates/uor-r4-api/tests/prompt_arms_bakeoff_834.rs`
  (default `cargo test -p uor-r4-api --test prompt_arms_bakeoff_834`); the teacher-grounded
  run harness `crates/uor-r4-api/tests/causal_prompt_run_834.rs` (ignored) and its
  CID-bound record `docs/causal_run_834_result.json` (§6.1).

## 1. Problem and scope

#784 measured **0.0% full-depth context-code collisions** while 11/15 distinct rows
still favored the same continuation, and historical sampled canaries were only about
5–6/15 distinct — **diversity diagnostics, not causal-relevance results**. The open
question of S1 is *causal*: does prompt meaning move prediction beyond suffix/exact-
context memory? A single bidirectional-anchor implementation would confound four
distinct mechanisms — persistent-state discrimination, candidate availability,
conditional evidence, and decoding — so #834 fits and ablates **several** artifact-only
arms against strong causal and equal-resource controls before selecting at most one.

**Definition (execution scope).** This deliverable is a **reference-only /
off-serving-path** instrument in the sense of
[`docs/conformance_execution_scope_830.md`](conformance_execution_scope_830.md). It
provides (a) an offline, integer, deterministic **bake-off harness** over the arms and
controls #834 enumerates, built on the frozen #835 reference model and the frozen #832
evaluation vocabulary, and (b) the **binding cheap instrument** the run contract
requires to pass *before* any long run may launch. It is **not** deployed-serving
evidence, and it is **not** the S1 causal verdict — that measurement is the
maintainer-gated long run of §5. Tuning any deployed decision on the harness fixture is
a #834 non-goal ("Tune on the held-out intervention suite").

## 2. Arms and controls

**Definition (the five arms).** Each arm is a pure, integer, decode-independent reader
over a candidate set (`crates/uor-r4-api/tests/prompt_arms_bakeoff_834.rs`):

| Arm | What it reads | Isolates |
|---|---|---|
| `current-scoring` | suffix-local score (the deployed baseline) | the no-whole-prompt floor |
| `longer-local-context` | a longer suffix window | context length ≠ whole-prompt state |
| `persistent-state` | the whole-prompt fold (#835 `Ψ`) | state discrimination |
| `conditional-residuals` | persistent state minus the corpus marginal | evidence beyond the marginal |
| `candidate-support-expansion` | persistent-state scoring over an expanded candidate set | candidate availability |

**Definition (the controls / nulls).** Mapped to the frozen `ControlKind` vocabulary
(#832): `prompt-swap`, `suffix-only`, `shuffled-state`, and `trivial-prior`
(constant/root prior), plus the equal-budget controls the run contract names —
equal-candidate, equal-table-read, and a decoder held constant (a single fixed argmax
decoder across every arm and control). Each null is a real transform; none may pass
through zero variance or identical outputs.

## 3. The binding cheap instrument (Verification)

**Guarantee (the instrument operates and has teeth). Status: Structural** (machine-
checked; the named tests in `prompt_arms_bakeoff_834.rs`). The five Verification items
of #834 are each an executable, falsifiable test:

- **Interventions / attribution / shuffles operate** — `fixture_interventions_attribution_and_shuffles_operate`:
  a committed, document- and template-disjoint paired fixture across two domains with
  all six intervention families (paraphrase, subject, relation, negation, role,
  constraint); the shuffled-state control provably corrupts the recovered meaning; the
  attribution decomposition credits gains.
- **Power fixes n and the MDE before fitting** — `power_fixes_n_and_mde_before_fitting`:
  the sample count and minimum decision-relevant effect are compile-time constants set
  before any fitting; the design resolves the MDE.
- **Double-run / reordered-shard determinism** — `double_run_and_reordered_shard_determinism`:
  the CID-bound per-pair record reduces identically under repetition and under
  reversed/sharded order (a stable-id keyed, ordered reduction).
- **Planted negatives fail the primary gate** — `planted_negatives_fail_the_primary_gate`:
  a **prompt-insensitive** model and a **diversity-only** model (distinct output, no
  per-intervention relevance) are each flagged degenerate against the prompt-swap null
  and cannot reach the causal arm's separation — the #834 non-goal "promote distinctness
  without relevance", made a falsifier.
- **Independent report recomputation** — `independent_report_recomputation_from_cid`:
  the report CID recomputes from the canonical record bytes.

**Guarantee (candidate availability and score conditioning are separately identified).
Status: Structural** (`candidate_and_score_conditioning_separately_identified`). On
in-vocabulary pairs the persistent-state gain attributes to state discrimination and
never to candidate availability; on needs-expansion pairs the gain attributes to
candidate availability and never to state discrimination.

**Guarantee (every control is non-degenerate). Status: Structural**
(`every_control_is_non_degenerate`, `causal_arm_separates_from_all_nulls`). A genuinely
causal arm separates from every null, and each null changes at least one prediction —
so a zero reading means "no effect", not "broken harness" (the anti-vacuity property the
#835 planted-negative teeth already established for the state model, extended here to the
full arm/control matrix).

## 4. Reachability and power (projected, before fitting)

**Objective (reachability ceiling).** For each arm the harness computes, separately, the
fraction of pairs whose **candidate set** it can change (candidate-recall ceiling) and
whose **scorer** it can move (scoring ceiling); headline movement over the baseline
cannot exceed their sum (`reachability_ceiling_bounds_headline_movement`). This is the
run contract's reachability arithmetic instantiated on the instrument fixture.

**Empirical Criterion (power design). Status: Structural** (the design; not a corpus
result). The instrument's sample count and MDE are fixed before fitting; the powered `n`
of the real S1 study is set by its own protocol on #833 and is **UNAVAILABLE** here.

## 5. Run contract and entry-gate assessment

The #834 run contract, with the availability of each required input recorded (a missing
input is **UNAVAILABLE**, never a vacuous pass):

- **Metric / current value:** primary paired `causal-influence-delta` on EXCT-disabled,
  document-disjoint pairs; the historical surface diagnostics (11/15 newline argmax rows;
  ~5–6/15 distinct canaries) are diversity, not causal relevance, and are to be
  remeasured on #833.
- **Reachability ceiling:** the per-arm candidate and scoring ceilings above; on the
  real corpus these are computed from the #833 attribution, **UNAVAILABLE** here.
- **Pinned identities:** the #835 state-spec revision and the #832 suite vocabulary are
  **AVAILABLE and frozen** (`docs/prompt_state_spec_835.md`,
  `docs/capability_suites_832.md`, `s1-causal-prompt-pairs`). The canonical
  teacher / tokenizer / corpus / partitions / **bundle from #833** and the fixed decoder
  identities for the real evaluation are **UNAVAILABLE** in this offline instrument.
- **Nulls / falsifier:** implemented and non-degenerate (§3).
- **Binding cheap instrument:** **PASS** — the committed fixture's state variance,
  intervention direction, candidate change, and every null are non-degenerate, and the
  reachability arithmetic exceeds the effect floor (§3, §4).
- **Exit rule:** select only when the primary causal-relevance lower confidence bound
  clears the frozen floor on both domains while relevance, bits/accuracy, and resource
  budgets satisfy non-regression. Evaluated on #833, **UNAVAILABLE** here.
- **If positive:** open the selected layout/runtime lowering in sibling item **#836** and
  ledger all other arms as rejected/dormant with evidence.
- **If negative:** publish the falsifier, keep current serving semantics, and return S1
  to representation/compiler redesign — not code-space or sampling tuning.
- **Cost estimate (projected for the long run, not incurred here):** the real bake-off
  runs every arm × control on the #833 canonical bundle across ≥2 domains with a powered
  `n`; wall time, peak RSS/storage, teacher-evaluation count, and per-arm artifact bytes
  are set by that protocol and are posted to #834 before the run is launched.

**Guarantee (no long run launched here). Status: Structural.** This deliverable launches
**no** hours-scale run. It lands the binding cheap instrument the contract requires
first; the real multi-arm study on #833 is a **maintainer-gated** decision (post exact
run values, obtain go/no-go) with programme-wide blast radius, because only a predeclared
positive causal verdict may trigger deployed lowering (#836) and gate the S1→S7 lane.

## 6. Decision status

**Empirical Criterion (S1 causal verdict). Status: Unproven / UNAVAILABLE.** No
`SELECT` / `REVISE` / `NO PROMPT-CONDITIONING ARM ESTABLISHED` verdict is asserted by this
deliverable. The harness's `verdict_vocabulary_is_complete` and
`instrument_selects_causal_arm_and_rejects_planted_negatives` tests demonstrate the
decision function can produce all three verdicts and, on the controlled fixture, selects
a genuinely causal arm while rejecting planted non-causal negatives — this validates the
**instrument**, and is explicitly **not** the S1 result.

## 6.1 Recorded teacher-grounded run (2026-08-20)

**Empirical Criterion (S1 causal verdict, deployed / current-scoring arm).
Status: Empirical.** A teacher-grounded run was executed on the #833 canonical bundle
`smollm2-360m-broad-clean`, reusing the teacher argmax already recorded in the bundle's
corpus and the deployed EXCT-disabled engine — no live teacher forward. Harness:
`crates/uor-r4-api/tests/causal_prompt_run_834.rs` (ignored; run with
`cargo test -p uor-r4-api --release --test causal_prompt_run_834 -- --ignored`). Record:
`docs/causal_run_834_result.json`. Protocol: n = 24,044 held-out real Simple-Wiki
positions (in-story windows, mean 7.8 tokens, 98% longer than the 2-token suffix), top-1
agreement with the recorded teacher argmax, EXCT disabled.

- Pinned identities: **artifact_cid** `blake3:bc2366f1…`, **corpus_meta_cid**
  `blake3:aa9d1767…`, **result_cid** `blake3:aad1511d…`.
- full-context 269.3‰ (95% CI [263.6, 274.9]); suffix-only 270.9‰;
  **causal-influence-delta −1.6‰ (paired 95% CI [−2.4, −0.8])** — full context does not
  beat a 2-token suffix, and the interval excludes a positive effect.
- context-saturation sweep (top-1‰ vs teacher): k1 258.5, k2 270.9, k3 270.0, k4 269.8,
  k6 269.6, full(8) 269.3 — **flat**; context beyond ~2 tokens adds no predictive signal.
- nulls non-degenerate: prompt-swap 21.6‰, trivial-prior 49.8‰ (both far below
  full-context); attribution context-helped 27 / 24,044.
- **minimal pairs** (same 2-token suffix, different story, different teacher argmax):
  1,460 pairs, the model **follows 0**; the suffix-only control is degenerate-by-
  construction (1,460 / 1,460 identical), so a zero reading is "no effect", not a broken
  harness.

**Verdict: `NO PROMPT-CONDITIONING ARM ESTABLISHED`** for the arms fittable against the
current artifact (current-scoring, longer-local-context): the deployed model is
suffix-local. Per the #834 run contract's *if negative* branch, current serving semantics
are kept and S1 returns to representation/compiler redesign rather than code-space or
sampling tuning; this corroborates #784's continuation-convergence finding, now
quantified and teacher-grounded.

**Scope of this verdict (do not over-read). Status: Assumption.** The three arms
`persistent-state`, `conditional-residuals`, and `candidate-support-expansion` are the
#835 Ψ-family mechanisms that are **not lowered** into the artifact (the engine's public
surface returns the selected token, not a candidate score vector), so they are
**UNAVAILABLE** here and can be fitted only after #836 builds them. The measured negative
therefore establishes that the current mechanism has no prompt-conditioning beyond a short
suffix — making #836 the prerequisite for any positive S1 conditioning claim — and does
**not** assert those unbuilt mechanisms would also fail.

## 6.2 Ψ segment-lane reference-arm re-test (2026-08-20)

**Empirical Criterion (whole-prompt segment lane vs suffix baseline). Status: Empirical.**
Following the §6.1 negative for the *deployed* arms, the #835 **segment lane** —
whole-prompt content → candidate-support contributions — was built as an offline reference
arm and tested teacher-grounded (harness `crates/uor-r4-api/tests/psi_arm_run_834.rs`;
record `docs/psi_arm_834_result.json`). From 288,794 TRAIN positions (document-disjoint by
story) two co-occurrence tables were built — a 2-token suffix→teacher-argmax table (the
baseline) and a content-token→teacher-argmax table (the segment lane, bounded to top-64 per
key) — and scored on all 72,130 held-out positions. λ was fixed at 1.0 before evaluation.

- suffix baseline top-1 vs teacher **246.6‰**; Ψ (suffix + whole-prompt content)
  **264.1‰** → **Ψ-delta +17.5‰ (paired 95% CI [+15.9, +19.0])**; the interval excludes
  zero. Exploratory λ-sweep (not the verdict): 0.5→264, 1→264, 2→261, 4→248, 8→223 ‰ — the
  effect is not an artifact of over-weighting content.
- **minimal pairs** (same 2-token suffix, different story, different teacher argmax): Ψ
  **follows 10 / 4,722** (95% lower bound 0.8‰) where the suffix baseline — a pure function
  of the suffix key — follows **0 / 4,722** by construction.

**Verdict: `SELECT` (positive signal), per the pre-registered rule.** Whole-prompt content
carries real predictive signal for the teacher's answer beyond the 2-token suffix. Read
together with §6.1: the deployed artifact is suffix-local and **discards** this signal, but
the signal is present and a bounded segment-lane mechanism recovers it — so building the
#835 persistent-state mechanism (**#836**) is warranted (the run contract's positive that
legitimately triggers #836).

**Effect size — do not over-read. Status: Empirical.** The gain is **modest**: +1.75pp
overall, and only **10 of 4,722** hardest minimal pairs are resolved. The segment lane helps
on average (candidate-support widening plus scoring) but rarely resolves genuine same-suffix
context dependence. #836 should be pursued with calibrated expectations, not as a decisive
fix. This is a reference-only result; a deployed lowering must re-clear the causal gate on
the packed production path (#836's own acceptance).

## 6.3 Conditional-residuals reference-arm run (2026-08-21)

**Empirical Criterion (conditional residuals vs corpus marginals). Status: Empirical.**
Following the 2026-08-21 maintainer decision on #834, the last unmeasured arm of the §2
five-arm scope — `conditional-residuals`, persistent state minus the corpus marginal —
was built as an offline reference arm and run teacher-grounded on the #833 canonical
bundle (harness `crates/uor-r4-api/tests/conditional_residuals_run_834.rs`; record
`docs/conditional_residuals_834_result.json`; run contract posted to #834 before the
run). Construction identical to §6.2 where shared: the same suffix and content tables
from the 288,794 document-disjoint TRAIN positions plus the UNCAPPED corpus marginal
(16,524 entries over 288,794 TRAIN targets; argmax token 260), scored on all 72,130
held-out positions with λ = 1.0 fixed before evaluation and a fixed argmax decoder
(score desc, id asc) across every arm and control. The arm scores
`suffix_rate + λ·(content_rate − marginal_rate)` over the §6.2 Ψ-widened candidate set,
so CR-vs-Ψ is a pure scoring-rule ablation. Before the reading was accepted, the harness
reproduced §6.2 to the digit (suffix 246.6‰; Ψ 264.1‰, +17.5‰ CI [15.9, 19.0]; 4,722
minimal pairs, Ψ-follow 10, baseline-follow 0) — the harness-correctness gate — and
passed the in-harness 2,000-position double-run determinism check. Identities:
corpus_meta_cid `blake3:aa9d1767…`, result_cid `blake3:feece1b3…`.

- **CR (primary): 262.8‰ vs suffix baseline 246.6‰ → +16.2‰ (paired 95% CI
  [+14.6, +17.9])** — the interval excludes zero AND its upper bound sits below the
  frozen 20‰ `CAUSAL_FLOOR_PERMILLE`: the arm is real but **unconditionally sub-floor**,
  the same reachability arithmetic that made the #836 segment lane dormant.
- **CR vs Ψ: −1.3‰ (paired 95% CI [−2.4, −0.1])** — subtracting the corpus marginal
  *slightly hurts* relative to §6.2's raw content evidence on identical candidates and
  tables: the mechanism-specific increment of the conditional-residual transformation is
  non-positive at this scale.
- **Residual-shuffle null: +16.0‰ [14.4, 17.6]**, statistically indistinguishable from
  the real arm (+16.2‰) while changing 14,478 individual predictions — the specific
  marginal alignment contributes nothing; what carries the arm is the §6.2 content
  evidence, not the subtraction. Together with CR-vs-Ψ this is the falsifier for the
  arm's distinctive mechanism.
- **Attribution (candidate availability vs score conditioning):** `cr-narrow` (the same
  residual scoring over the suffix candidate set only) reaches +3.1‰ [2.3, 4.0]; the
  widened arm reaches +16.2‰ — most of the arm's movement is candidate availability,
  consistent with §6.2's segment-lane structure.
- **Controls non-degenerate:** prompt-swap −7.3‰ [−8.7, −5.9] (different-story content
  hurts; 27,758 predictions changed) — the gain is prompt-specific; trivial-prior 49.1‰;
  λ-sweep flat (262.1 / 262.8 / 262.0 / 259.7 / 245.8‰ at λ = 0.5/1/2/4/8) — not a
  weighting artifact.
- **Minimal pairs:** CR follows 13 / 4,722 (2.8‰, 95% lower bound 1.3‰) vs Ψ 10 and
  baseline 0 — a minute real hard-pair signal.

**Verdict: `REVISE`, per the pre-registered rule posted to #834 before the run** (SELECT
required the paired-delta lower bound ≥ 20‰; a minimal-pairs bound alone can no longer
produce SELECT, per the #887 governance verdict). The signal is real and sub-floor with
its CI upper bound below the floor, so **no lowering track opens** — by the #887
calibration a deployed lowering re-faces integer-`ScoreQ` quantization and the top-8
candidate ceiling and cannot exceed an off-serving ceiling that is already below the bar.

**Arm-space disposition (closing the §2 scope). Status: Empirical.** With this run every
arm of the §2 five-arm scope carries an evidence-backed disposition: `current-scoring`
and `longer-local-context` — negative (§6.1; the deployed model is suffix-local);
`persistent-state` (segment lane) — positive-sub-floor off-serving (§6.2), lowered
end-to-end (#836), then retired from the promotion track with the 20‰ bar standing
(#886 lowering-fidelity gap 1/10; #887 governance); `conditional-residuals` — REVISE,
real-but-sub-floor with a non-positive mechanism-specific increment (this section);
`candidate-support-expansion` — closed not-planned (#888, its own gates failing and
superseded by #887). **No arm cleared the frozen causal floor.** Per the run contract's
negative branch: current serving semantics are kept, no lowering opens, and S1 meets the
#822 kill/redesign criterion (two-plus independently motivated arms fail the causal
gate) — the S1 stage verdict against the full child set is the maintainer's call on
#822, and further movement at S1 is representation/compiler redesign, not code-space or
sampling tuning.

## 7. Repository conformance

**Definition (RF mapping).** This instrument extends the evidence of existing capability
IDs and introduces **no new built RF capability**; following the #835/#832 spec-leaf
precedent it adds **no** `model/ids.toml` row and triggers **no** `CONFORMANCE.md`
regeneration (generated conformance is never hand-edited):

- **RF-27 / RF-28** (semantic state / typed transitions; state–emission separation,
  reference/f32): the arms are prompt-conditioned instantiations, scored decode-
  independently.
- **RF-01** (unsupervised intervention and counterfactual behavioral probes): the paired
  interventions and the planted negatives.
- **RF-21 / RF-22** (R4G1 compilation quality gates / pathology filter): the equal-budget
  and shuffled controls.

**Definition (built-capability order for #836).** When #836 lowers the *selected* arm
into a deployed capability it follows the required order — `model/ids.toml` row → tagged
Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md` —
and either extends an existing suite or justifies a new capability then. Unselected arms
remain ledgered with activation or retirement gates.

## 8. Claim status and next action

**This deliverable freezes #834's binding cheap instrument; it does not establish that
any arm has prompt-causal relevance.** The next action is the **maintainer-gated long
run**: post the exact #833-scoped run values to #834, obtain go/no-go, then run the
multi-arm bake-off and record a `SELECT` / `REVISE` / `NO PROMPT-CONDITIONING ARM
ESTABLISHED` verdict with CID-bound records. A positive verdict opens #836; a negative
verdict narrows or retires the S1 conditioning claim under the #822 kill/redesign
criterion and is first-class completion evidence, not an implementation backlog.
