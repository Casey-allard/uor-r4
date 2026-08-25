# ADR-0001: R4G1Runtime owns normative serving selection

- **Status:** Accepted
- **Date:** 2026-08-20
- **Issue:** #831 (item C of S0 tracker #821, programme #820)
- **Scope of this decision:** Normative `R4G1Runtime` candidate and token
  selection. `R4Engine` / `GraphScorer` is token-free D4 policy plus an
  explicitly named reference / certifier scope; evidence from it is not
  credited as a deployed-serving result.
- **Implementation evidence status:** **Current and empirically bound at the
  #933 scope.** #831 accepted the semantic owner; #933 subsequently established
  SKMX/PSIB consumption, common production-adapter ownership, zero-mismatch
  cross-surface evidence, and a strict schema-2 full-census admission envelope.
  RF-31 is RATIFIED only for that exact measured artifact/population/decode.
- **Relation to prior records:** Extends `docs/inference_contract.md` (RF-10) and
  `docs/scoring_semantics.md`; does not retract them. Claim language follows
  `docs/formal_vocabulary.md` (normative). **Definition.** This record designates
  the semantic owner and scopes the other scorers. Structural properties are
  labeled separately below; this decision is not model-quality evidence.

## Context

The repository carries several scoring surfaces that were each motivated separately and whose
production-versus-reference roles were **not expressed as one unambiguous normative production
semantics** (the #831 problem statement):

- `uor-r4-graph-runtime` — the `no_std`, allocation-free deployed runtime
  (`R4G1Runtime::step` / `predict_distribution*` / `predict_candidates*`), plus its
  `scoring` (fixed-point accumulator, `OrderedScore` ordering, canonical selection),
  `patch_chain`, and witness surfaces. This is what selects the **served token** on the
  deployed path (`src/chat.rs`, `src/tless_uor.rs`, `src/server.rs`).
- `uor-r4-graph-format::scoring_semantics` (v1.0.0) — the machine-readable **normative
  specification** of the fixed-point scoring semantics: `ScoreQ` (Q16.16), the seven typed
  residual contribution kinds, saturating accumulation, the no-double-counting rule, and the
  deterministic tie-break. It is a specification, not a second serving implementation.
- `uor-r4-graph-certify::score_runtime::GraphScorer` (the "Phase-4 reference scorer") — a
  witness-replayable, integer-accumulation reference/certifier scorer (Rule 2 exact-context
  precedence, Rule 1 chain-telescoped graph residuals). It is also loaded on the served path
  **as the D4 abstention-policy resolver** through `uor-r4-api::R4Engine`.
- `uor-r4-graph-certify::score` — the offline Gate C measurement harness.
- `GraphScorer::score_candidates_legacy` — the retired Σ-over-cloud formula (confirmed double
  counting; kept only for the Gate C side-by-side).
- `uor-r4-router` — the exploratory `f64` geometric retrieval router, off the P-4 kernel and
  off the serving-selection path by design.

**The concrete drift this ADR was intended to remove.** On the deployed path the served token is chosen by
`R4G1Runtime`, while whether to serve or abstain is chosen by `R4Engine`/`GraphScorer`
resolving the *same* position. These are two implementations, and they are **not exactly
equivalent**: for example the Cayley–Dickson `syntactic_morphism_score` term runs only in the
`R4G1Runtime` multi-candidate generation slice (recorded in `crates/uor-r4-graph-certify/tests/r4g1_cd_ab.rs`).
Quality measured on one surface (Gate C over `GraphScorer`) was therefore discussed as though
it applied to what the other surface serves. Without a single named owner, such divergence is
silent and would invalidate every downstream S1–S7 gate that cites "the scorer".
The #933 audit later established that #831's implementation evidence was
partial: some production adapters and the SKMX/PSIB lane did not reach the
designated owner. The accepted ownership decision remains in force; the
historical reachability conclusion does not.

## Decision

**Definition.** The one normative candidate and token selector for deployed
inference is `uor-r4-graph-runtime`'s
`R4G1Runtime::predict_served_candidates*` path. Its base
`step` / `predict_distribution*` / `predict_candidates*` machinery and
`scoring` module feed that selector; its state, tie-break, saturation, and
no-double-counting rules are specified normatively by
`uor-r4-graph-format::scoring_semantics` (v1.0.0) and constrained by the operation contract in
`docs/inference_contract.md`. The served token is, by definition, the token this path selects.

The normative semantics are fixed as follows (all already realized in code; this ADR names
them as the single owner):

- **Definition.** **Score domain:** `ScoreQ` = signed Q16.16 integer. `ScoreQ::MIN`/`MAX` are the
  saturated-low/high sentinels.
- **Definition.** **Accumulation:** pre-quantized, already-signed residuals combined with
  **saturating** integer add (`i32::saturating_add`); positive/negative overflow clamps to
  `MAX`/`MIN`. No float, no multiply, no divide, no runtime rescaling on the hot path.
- **Guarantee. Status: Structural.** **No-double-counting:** each canonical evidence id contributes at
  most once per candidate evaluation.
- **Guarantee. Status: Structural.** **Deterministic tie-break:** primary key `ScoreQ` descending,
  secondary key candidate id (token/node) ascending — a total order across x86_64, aarch64,
  and portable scalar targets.
- **Definition.** **Resolution status / decline:** the single **scorer resolution-status**
  space is `ScoreStatus` (`uor-r4-api::ResolutionStatus` is a type alias of it — there is no
  second definition of the *scorer* status space). This is distinct from, and must not be
  conflated with, `uor-r4-graph-runtime::status::ResolutionStatus`, which classifies ROUT
  routing margin (Supported / Boundary / BackedOff / Novel / Contradictory) and is a routing
  concern, not the scorer's resolution class. The deployed D4 `StatusPolicy`
  (`serve` / `widen_once` / `abstain`) is the decline policy; abstention is a typed outcome,
  never a guessed token.
- **Patch precedence (owner):** `uor-r4-graph-runtime::patch_chain`
  (`R4G1Runtime::try_push_patch`) is the single normative owner of patch/delta interpretation.
  Bytes that are not a valid, compatible R4G1 artifact **fail closed** (a rejection reason is
  returned; no partial patch is applied).
- **Witness (owner):** the R4G1 witness / replay surface is the single normative owner of
  witness semantics; a witness whose token, region, depth, length, or contribution ids do not
  reproduce the artifact replay **fails closed** (typed `ReplayError` / witness-rejection
  reason), and no unverified witness is credited.

**2. The other scorers are explicitly scoped (distinct identities, distinct claim
boundaries).** Convergence of the implementations is a non-goal; clear scoping is sufficient
(#831 non-goals). No scorer other than the one above may be cited as deployed-serving
evidence.

| Surface | Crate / symbol | Role | Execution scope |
|---|---|---|---|
| **Normative deployed scorer** | `uor-r4-graph-runtime` `R4G1Runtime` + `scoring` | Selects the served token; owns patch + witness | `normative-runtime` (served) |
| Normative specification | `uor-r4-graph-format::scoring_semantics` v1.0.0 | The rules the deployed scorer realizes | `normative-runtime` (spec) |
| Reference / certifier scorer | `uor-r4-graph-certify::score_runtime::GraphScorer` | Witness-replayable reference; **and** the deployed D4 abstention-policy resolver via `R4Engine` | `certifier-instrument` (+ policy-only on the served path) |
| Gate C harness | `uor-r4-graph-certify::score` | Offline held-out measurement | `certifier-instrument` |
| Legacy scorer | `GraphScorer::score_candidates_legacy` | Retired Σ-over-cloud formula (double counting) | `reference-only` (retired) |
| Geometric router | `uor-r4-router` | Exploratory `f64` retrieval | off-serving (out of scope) |

**3. Bounded, named non-equivalence.** `R4G1Runtime` (served token) and `GraphScorer`
(reference / abstention policy) are **not asserted exactly equivalent** — they are distinct
implementations with distinct identities. They **share** the normative tie-break, saturating
accumulation, no-double-counting rule, and the single `ScoreStatus` status space. Their known
divergence is bounded to the `R4G1Runtime` multi-candidate generation slice (the CD term). The
decline rule that keeps this fail-closed: on the served path, a position whose reference
resolution status and normative served status disagree is resolved **conservatively** — the
D4 policy already routes anything not resolved `ExactContext`/`Graph` through `WidenOnce` and
then abstains, so a drifted position abstains rather than silently serving a divergent token.

**4. Per-token attribution.** The normative scorer already carries per-token **resolution
attribution** through `ScoreStatus` (ExactContext / Graph / Novel …), the typed residual
contribution kinds (RootPrior / ChildCorrection / InteractionResidual / GoalReward /
ConstraintPenalty / UncertaintyPenalty / TokenEmission), and the witness (region, depth,
edges, contribution ids). This ADR names that surface as the attribution owner. Committing the
**CID-bound per-token attribution capability suites** on top of it is the explicitly scoped
deliverable of sibling item **#832** (documented exception, not omitted).

## Intended reachability and #831 evidence boundary

**Definition.** Every production entry point that selects a token is required
to reach the normative `R4G1Runtime` selector; the reference scorer is permitted
only in its named token-free policy and reference/certifier roles. The table
below records that accepted architecture. #831 did not mechanically execute
every row; its reachability evidence remains historical/partial. #933 supplies
the missing behavior tests and content-bound cross-surface artifact, with zero
mismatches on the exact admitted evidence generation.

| Entry point | Path | Reaches |
|---|---|---|
| Local chat / generation | `src/chat.rs` → `NormativeStepAdapter` → `R4G1Runtime::predict_served_candidates_with_signature_lanes` | Normative deployed selector |
| Transformerless serve | `src/tless_uor.rs` → `NormativeServingEngine` → `R4G1Runtime::predict_served_candidates_with_signature_lanes` | Normative deployed selector |
| HTTP / WS / WASM server | `src/server.rs` → the shared production composition above | Normative deployed selector |
| Library façade | `uor-r4-api::NormativeServingEngine` / `NormativeStepAdapter` | `R4G1Runtime` selects candidates/token; `R4Engine` resolves token-free D4 policy only |
| Certification (Gate C, replay) | `uor-r4-graph-certify` `score` / `score_runtime` / `verify_witness_replay` | Reference / certifier scope |
| Proof obligations | `uor-r4-proof-model` (`ExecutableSpec`, Kani surface) | Structural obligations over the runtime + format |

The historical #831 harness,
`crates/uor-r4-graph-certify/tests/normative_scorer_831.rs`, checks scorer
semantics and a limited fail-closed/reachability surface. It does not establish
all local, HTTP, library, beam, sampled, and WASM adapters and must not be cited
as complete deployed-serving reachability evidence.

## Consequences

- Downstream reports (S1–S7) must name the normative scorer identity when citing scorer
  quality, and must not transfer a reference/certifier measurement to the served path without
  the reachability shown here.
- Adding a third production scoring implementation is prohibited by this ADR; extend the
  normative path or add an explicitly scoped reference with its own claim boundary.
- The `scoring_semantics` module remains the single normative spec: a change to tie-break,
  saturation, accumulation order, or the no-double-counting rule is an ADR-superseding change
  and versions the spec (§`ScoringSemanticsVersion`).
- No format or ABI bytes change in this ADR (convergence is a non-goal). If a future change
  alters bytes or semantics, version the relevant section/operator and publish a migration
  decision.

## Conformance mapping

Reuses/extends existing capability rows (no new RF id required):

- **RF-23** `r4g1_runtime` (`normative-runtime`, `deployed-serving`) — R4G1Runtime selection
  and fallback: the normative deployed scorer designated here; evidence extended to cite this
  ADR.
- **RF-10** `inference_contract` (`normative-runtime`, `deployed-serving`) — the operation
  contract the normative path obeys.
- **RF-09** `graph_invariant_ownership` (`normative-runtime`, `deployed-serving`) — loader
  validation that makes incompatible artifacts fail closed.
- **RF-22** `r4g1_quality` (`certifier-instrument`, `off-serving-path`) — the certifier
  instrument scope kept distinct from the served scorer.
- **RF-15** / **RF-21** (`offline-compiler`, `off-serving-path`) — compile-time quality /
  reproducibility, distinct from serving.

`CONFORMANCE.md` is regenerated (never edited directly) after the RF-23 evidence extension.

## Verification

- `crates/uor-r4-graph-certify/tests/normative_scorer_831.rs` — differential of the normative
  specification (`scoring_semantics`) against the deployed runtime `scoring` accumulator and
  selector; a **planted-negative** divergent scorer that the differential detects; the
  single-source status-space check; reachability of both the deployed and reference scorers
  from the same artifact bytes; and the incompatible-artifact fail-closed boundary.
- The four local gates + register conformance (R1–R6) + wasm-router lib build.

## Claim status

**Definition.** This record establishes semantic ownership, not complete
production reachability and not model quality. #831's reachability evidence is
historical/partial; #933 owns the missing mechanical and content-bound evidence.
The record does not claim behavioral equivalence between the two scorers, and it does not
establish any generation-quality result.

## Implementation-audit addendum — 2026-08-24 (#933)

This decision is reaffirmed, not superseded: `R4G1Runtime` remains the sole
normative production candidate and token selector, and `R4Engine` / `GraphScorer`
remains tokenless D4 policy plus reference/certifier scope on production paths.

The #933 audit found a post-ADR implementation divergence. #908 measured
SKMX/PSIB through `R4Engine::predict_decision_candidates_with_skipmix`; #910
activated that `R4Engine` reroute and compiler emission but did not add SKMX/PSIB
consumption to `R4G1Runtime`. Local chat still selected the served token through
`R4G1Runtime`, while other greedy/sampled/API surfaces did not share one candidate
owner. The existing quality report likewise measured the certifier Rule 1+2 row,
not the exact normative selector with a bound production population.

**Empirical Criterion. Status: Empirical. Execution scope:
reference/off-serving.** #908's 29.702% top-1 and +28.45‰ paired delta remain
valid for that exact harness. They are not credited as normative
deployed-serving evidence. RF-31 is **NOT ESTABLISHED** at that scope until #933
proves exact absent-section identity, planted SKMX/PSIB reachability in
`R4G1Runtime`, common greedy/sampled/beam candidates across every production
surface, and production admission of a versioned full-census report bound to the
exact graph, artifact, corpus, tokenizer, partition, selector, and release
manifest. A missing or mismatched binding is a typed decline, never a PASS.

## Implementation and evidence resolution — 2026-08-25 (#933)

This append resolves the condition above without superseding the ADR.
`R4G1Runtime::predict_served_candidates*` remains the one candidate/token
authority; D4 remains token-free policy. Local ask/chat, greedy, pinned-seed
sampled, beam, HTTP response surfaces, the public library, and WASM all compose
through the shared normative adapter. The raw eight-row cross-surface artifact
and the full evaluator together record 72,138 checks with zero mismatches.

**Empirical Criterion. Status: Empirical. Execution scope: normative-runtime /
deployed-serving.** Graph/evaluator revision
`74ced4d12a84a176d73665106f88d0aab9407453` records 21,293 / 72,130 =
29.5203% through the normative selector, versus same-position TLA 20,284 /
72,130 = 28.1214% (+13.988‰, 95% CI [11.057, 16.919]) and same-generation
sections absent 18,806 / 72,130 = 26.0723% (+34.479‰ [31.681, 37.277]).
Internal absent identity is exact on 72,130 positions, the label-shuffled
control is negative, and witness replay is 64/64 with zero failures.

Hardened verifier revision
`f901cd97577da3117fd52c9b1c6dcf075cc4d3a2` strictly admits the exact
schema-2 envelope from an empty model store. The release manifest's hardened raw
BLAKE3 is
`c2025e9e507e8367993d78bd83ef099ce5851c838d3cc5cf01eda5560986ad33`
(SHA-256
`7572e07a1e3722f3ffc0ea749a67b4ac162221de79b5b4b8a315f4e4e6570fde`)
and it binds TLA comparator store CID
`c1749e62077758c4a098e2a02150b5455e1ca3c02c60b87e6d45fcbb9e2b4404`.
**Verdict: RF-31 RATIFY for this exact graph/report/population/decode.** This is
not scorer-wide equivalence, a universal quality floor, live-teacher
parity, instruction following, reasoning, or free-running coherence.
