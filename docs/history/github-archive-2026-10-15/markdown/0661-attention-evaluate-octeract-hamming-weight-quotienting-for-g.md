# #661 — attention: evaluate Octeract Hamming-weight quotienting for geometric route attention

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-13T12:20:17Z
- **Updated:** 2026-08-15T03:08:28Z
- **Closed:** 2026-08-15T03:08:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/661
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

Two supplied research papers propose an 8-bit binary Kaprekar/“Octeract” map built from Hamming weight, complement symmetry, canonical shells, and idempotent coarse-graining. This issue converts that material into a bounded, falsifiable R4 investigation.

R4 already uses Hamming geometry. The question is therefore not whether `popcount` can be introduced, but whether the papers’ complement-folded quotient or related weight bounds improve a declared transformer step without destroying attention-relevant information.

# Source inputs

- **The Octeract Cypher: Coarse-Graining and Categorical Adjunctions in 8-Bit Phase Space Collapses**, supplied out of band; SHA-256 `44bab09a20253437aeef43057ae316fcded5b00fd9f6b180f83843f06d2bbb3c`.
- **Validating Octeract Cypher Mathlib**, supplied out of band; SHA-256 `5322c519fa872ca836e2ad23d523ecf655defedd3dd17589ba290dec62a93a5e`.

The source PDFs are research inputs, not repository artifacts. Record provenance and licensing before copying any source text or figures into the repository; derived definitions, tests, and independently written analysis may proceed.

For an 8-bit value with Hamming weight `k`, the paper defines

```text
K8(k) = 257 - (2^(8-k) + 2^k)
k:       0    1    2    3    4    5    6    7    8
K8(k):   0  127  189  217  225  217  189  127    0
```

The map factors completely through `popcount` and satisfies `K8(k) = K8(8-k)`. Its information-bearing class is therefore the five-shell quotient `s = min(k, 8-k)`; the particular integers `0/127/189/217/225` have no established metric meaning for attention.

Applied as an R4 hypothesis to `z = q XOR key`, this gives a projective Hamming relation:

```text
d = popcount(z)
s = min(d, B - d)
```

where `B` is the number of active bits in the block. Under bipolar `{-1,+1}` encoding, this preserves the magnitude of binary correlation while discarding its sign. That is plausibly useful as a multiplication-free coarse prefilter, but it also makes a key and its bitwise complement equally close. That collision is the primary falsifier.

The source claims require independent validation before adoption:

- the main paper calls a complement-folded map injective even though `K8(k) = K8(8-k)`;
- the validation document presents Lean declarations with `sorry`, so it is a proof roadmap rather than completed mechanized verification;
- the displayed Galois statement is not valid over unrestricted natural numbers and requires bounded domains plus explicit endpoint treatment;
- the reported `6.1824`-bit information loss is for uniformly distributed bytes and is not evidence about R4 route traces;
- self-correction, error-correction, and attention/LLM benefits are hypotheses, not results established by the papers.

# Current evidence

- [#602](https://github.com/UOR-Foundation/uor-r4/issues/602) defines the versioned attention vocabulary: projections, positional action, compatibility relation, selector, aggregation, output projection, state, and tie behavior.
- [#603](https://github.com/UOR-Foundation/uor-r4/issues/603) provides deterministic teacher Q/K/V and support traces for replacement studies.
- [#604](https://github.com/UOR-Foundation/uor-r4/issues/604) specifies dormant `r4-route-attention/1`. Its packed relation uses 288-bit route codes and `sum(popcount((q XOR key) AND mask))`, followed by deterministic bounded top-M selection and ScoreQ aggregation. See [format](https://github.com/UOR-Foundation/uor-r4/blob/main/crates/uor-r4-graph-format/src/route_attention.rs), [runtime](https://github.com/UOR-Foundation/uor-r4/blob/main/crates/uor-r4-graph-runtime/src/route_attention.rs), and [reference/certificate](https://github.com/UOR-Foundation/uor-r4/blob/main/crates/uor-r4-graph-certify/src/route_attention.rs).
- [#605](https://github.com/UOR-Foundation/uor-r4/issues/605) owns `route-fit/1`, its nulls, progressive head→layer→range→model ladder, and the existing teacher-forced metrics. The synthetic instrument produced evidence, but real-teacher and real-corpus quality arms remained unavailable; no real baseline should be invented here.
- [#606](https://github.com/UOR-Foundation/uor-r4/issues/606) owns the composed target-operator certificate and replay contract.
- Current [`route-fit/1`](https://github.com/UOR-Foundation/uor-r4/blob/main/crates/uor-r4-graph-compiler/src/route_fit.rs) uses cyclic expansion, bucket averaging, per-bit lower-median binarization, and a full mask. It is the primary fit seam for this comparison.
- [#310](https://github.com/UOR-Foundation/uor-r4/issues/310) already found that a learned power-of-two weighted-Hamming arm recovered only about 5% of its measured f32 gap, below its 25% shipping bar. Octeract must compare with that negative evidence, not rerun unweighted/weighted Hamming under a new name.
- [#395](https://github.com/UOR-Foundation/uor-r4/issues/395) evaluated E8/icosian 8-dimensional constructions. An 8-bit Boolean quotient is not E8 or octonionic geometry merely because each uses “eight”; any bridge needs an explicit map and tested invariants.
- Open [#643](https://github.com/UOR-Foundation/uor-r4/issues/643) owns the MSA-structured alternate selector. Open [#647](https://github.com/UOR-Foundation/uor-r4/issues/647) owns the reference-frame consistency guard, and [#648](https://github.com/UOR-Foundation/uor-r4/issues/648) reserves two-tier route codes for demonstrated unresolved mass. This issue must not duplicate those owners.
- [#515](https://github.com/UOR-Foundation/uor-r4/issues/515) requires unpromoted mechanisms and negative results to remain registered and default-off.
- Other Hamming consumers exist in the geometric fallback and legacy transformerless retrieval/index paths. They belong in the mapping audit, but this issue may prototype only one primary seat unless a predeclared attention arm clears its gate.

# Problem

Unary weight `wt(x)` is not an attention compatibility relation. R4 currently evaluates masked pairwise distance `wt((q XOR key) AND mask)`, and a complete attention operator must also declare selection, aggregation, positional behavior, state, output behavior, and ties.

The papers do not establish where their quotient belongs in those steps. At least three materially different interpretations are possible:

1. transform query and key encodings independently, then compare them;
2. apply the quotient to the pairwise XOR/distance relation;
3. use shell/weight information only as a first-stage bound or bucket, then refine with exact current R4 Hamming semantics.

They are not silently interchangeable. Complement folding may provide useful projective geometry or table sharing, or it may erase the polarity and locality that teacher attention needs. Raw attractor integers can also create apparent results that depend only on an arbitrary label encoding.

# Goal

Determine whether a precisely specified Octeract-derived Hamming-weight mechanism can improve R4 route attention or a directly adjacent Hamming-based engine mechanism under the repository’s CPU-first, table-native, deterministic, no-float/no-multiply hot-path constraints.

Completion means producing a reproducible `adopt`, `revise`, `retain-dormant`, or `reject` record. A well-instrumented negative result completes this issue.

# Scope

1. Add `docs/octeract_attention_661.md` with a page/claim/algorithm map. Classify every relevant source proposal as `directly-applicable`, `adaptable-hypothesis`, `already-covered`, or `incompatible`, and name its R4 owner.
2. Independently specify and validate the byte map with a direct sort/subtract oracle, the closed form, endpoint behavior, complement symmetry, idempotency, and the exact five-shell quotient. Reproduce all published byte examples.
3. Correctly scope any formal statement used by the implementation. A theorem shown with `sorry`, a finite computation, or an unrestricted-Nat statement is not proof. Reuse the repository proof/conformance owner (and [#653](https://github.com/UOR-Foundation/uor-r4/issues/653) if that integration is available) rather than introducing a parallel proof registry or required toolchain.
4. Produce a math-to-operator mapping for every [#602](https://github.com/UOR-Foundation/uor-r4/issues/602) field and for the geometric/legacy Hamming consumers. State explicitly whether each candidate is encode-first, relation-first, or prefilter-and-refine.
5. Run the cheap collision/reachability instrument on pinned [#603](https://github.com/UOR-Foundation/uor-r4/issues/603) traces before fitting or packed implementation. Report empirical shell entropy, class occupancy, mutual information with teacher support, exact-vs-complement collisions, and the oracle maximum support recall/Jaccard achievable after each proposed quotient.
6. Predeclare and compare these semantic arms at matched block/bit budgets:
   - current `r4-route-attention/1` as the primary baseline;
   - ordinary per-block Hamming weight (`weight9`) to isolate any value from complement folding;
   - `octeract-fold5`: the canonical class `min(d, B-d)`, not the raw anchor integer;
   - `octeract-oriented`: the folded class plus a polarity/hemisphere bit;
   - `octeract-prefilter`: folded/block-shell bucketing followed by exact V1 refinement;
   - the safe weight lower-bound prefilter `abs(wt(q AND m) - wt(key AND m)) <= d_H(q,key)`;
   - occupancy-matched random five-class and shuffled-block nulls.
7. Use the cheap instrument to select at most two Octeract candidates for the one-head A/B. Do not tune or add candidates after reading held-out labels.
8. For candidates that clear the one-head gate, reuse the #605 progressive replacement ladder and #606 certificate path. Keep candidate fitting and operator identities versioned separately from `route-fit/1` and `r4-route-attention/1`.
9. If every attention arm fails, perform one bounded classification-only screen of canonical cache keys/table sharing in the existing geometric and transformerless Hamming consumers. Do not expand into open-ended mechanism fishing or alter those engines in this issue.
10. Append source audit, fixtures, measurements, nulls, identities/CIDs, and final disposition to the evidence document; do not rewrite away negative rows.

# Non-goals

- No claim that the papers prove an attention mechanism, transformer equivalence, language-model quality, self-correction, or project completion.
- No serving activation, default-engine change, or change to [#655](https://github.com/UOR-Foundation/uor-r4/issues/655). Any positive candidate remains dormant until a separate promotion issue clears product gates.
- No use of `0/127/189/217/225` as a numeric similarity score without an independently justified and falsified metric. Relabeling the five anchors while preserving shell identity must not change a class-only mechanism.
- No assumption that complement invariance is desirable. Exact matches and complements must remain distinguishable unless the declared operator intentionally uses projective/antipodal geometry and evidence supports it.
- No reuse of unchanged Q/K/V parameters under an incompatible relation without a declared fit/replacement arm.
- No conflation of Boolean octets, E8/icosian geometry, octonions, or Cayley–Dickson constructions.
- No repeat of #310, #395, #643, or #648 under different terminology.
- No weakening of P-4 operation rules, steady-state allocation requirements, `no_std`, determinism, artifact/κ ownership, typed errors, or fail-closed unsupported behavior.
- No corpus-scale or long experiment before the exhaustive byte checks, reachability ceiling, nulls, anti-vacuity, and frame-consistency controls pass.
- No redistribution of the supplied PDFs until provenance and license permit it.

# Dependencies and blockers

- #602–#606 provide the operator schema, traces, baseline fit, runtime, and certificate path.
- #647 or its equivalent matched-frame control is required before a quality verdict. A failed/unavailable frame reports `UNAVAILABLE`, not a negative candidate result.
- #310 and #395 are required comparison evidence; #643 and #648 remain separate active owners.
- #515 owns dormant registration and activation gates.
- A formalization is optional unless a proof claim is used for conformance or promotion. If retained, it must use a pinned toolchain, compile without `sorry`, pass the relevant axiom/root audit, and avoid a second truth registry.
- Missing real teacher/corpus fixtures are reported as `UNAVAILABLE`; synthetic evidence must not be relabeled as real-model evidence.

# Proposed implementation shape

## Phase A — source and mathematical conformance

- Bind the two source digests and record license status.
- Implement a small scalar reference from the defining sort/subtract construction, not only the closed form.
- Exhaust all 256 byte inputs and all 65,536 full-mask byte pairs for pairwise candidates; property-test partial masks and wider/block-composed forms.
- Record corrections and unresolved claims explicitly. Do not make issue completion depend on proving unrelated claims from the papers.

## Phase B — cheap trace ceiling

- At natural 8-bit boundaries in the existing 288-bit route code, calculate `weight9`, `fold5`, oriented-fold, and null codes without changing the runtime operator.
- Measure how quotient collisions constrain the best possible teacher support and identify whether complement pairs are semantically interchangeable in actual traces.
- Stop before fitting if the oracle ceiling cannot meet the locked promotion floor or if the candidate is indistinguishable from its occupancy-matched null.

## Phase C — one-head semantic A/B

- Freeze the fixture, model/tokenizer identities, seed, block layout, masks, candidate definitions, nulls, and thresholds before running.
- Compare direct replacement and prefilter/refine claims separately. A prefilter succeeds only if exact-refinement behavior remains within its fidelity bound and actual candidate work/bytes fall materially.
- Apply #647 frame consistency and #605 anti-vacuity/deranged-support controls.

## Phase D — packed prototype only after promotion gate

- If a candidate clears Phase C, add a separately versioned reference and packed implementation with deterministic fitting, explicit witness fields, operation census, and independent replay.
- Differential-test reference versus packed behavior, then advance only one scope at a time through the #605 ladder.

## Phase E — disposition

- Positive: register the candidate as dormant and file a separate operator/activation issue with the real-teacher/corpus certificate requirements.
- Negative or unavailable: append the complete result and falsifier to the evidence record and ledger; preserve current behavior.

# Acceptance criteria

- [ ] Both papers are bound by exact title/file identity, content digest, provenance, and license state; the repository does not silently republish unlicensed source material.
- [ ] The evidence record classifies every relevant claim/mechanism and identifies the exact R4 owner and transformer/operator step it could affect.
- [ ] A scalar oracle reproduces the published map and exhaustive tests cover all 256 byte inputs, all 65,536 full-mask byte pairs for binary semantics, endpoints, ties, complements, idempotency, and representative masked/block composition.
- [ ] Any relied-upon theorem is correctly bounded and either proved without `sorry` under a pinned/audited toolchain or explicitly recorded as open and excluded from conformance claims.
- [ ] The issue distinguishes shell class identity from the arbitrary five anchor integers and distinguishes encode-first, relation-first, and prefilter/refine semantics.
- [ ] The trace ceiling reports empirical entropy/occupancy, teacher-support mutual information, quotient collision classes, exact/complement collision rates, and oracle support recall/Jaccard on pinned fixtures.
- [ ] Comparisons include current V1, `weight9`, oriented folding, #310 weighted-Hamming evidence where applicable, matched-budget random/shuffled nulls, and the #395 eight-dimensional result as a non-equivalence control.
- [ ] The #647 matched-frame control is materially nonzero; frame failure or missing real fixtures reports `UNAVAILABLE` rather than `FAIL` or success.
- [ ] A deterministic one-head report includes support recall/Jaccard, top-1/top-k agreement, bits/token, output/layer error where available, candidate count, false-positive rate, operations, bytes read, artifact bytes, and bit-flip stability.
- [ ] The current `r4-route-attention/1` regression arm remains byte-for-byte unchanged.
- [ ] At most two candidates advance from the cheap screen, and later replacement scopes run only after the preceding locked gate clears.
- [ ] If a packed candidate is built, reference-versus-packed differential tests, independent witness replay, allocation/operation census, P-4 scan, `no_std`, wasm where required, and deterministic repeated bytes/CIDs all pass.
- [ ] A positive result produces a separately versioned dormant proposal; a negative result preserves the fixtures, table, falsifier, and decision without post-hoc tuning.
- [ ] No serving path, default engine, historical artifact, κ pin, or current runtime behavior changes in this investigation.

# Verification

- Direct sort/subtract oracle versus closed form and lookup table.
- Exhaustive byte and pair-domain checks plus masked/wider property tests.
- Double-run equality for trace-derived parameters, report bytes, artifact identities, witnesses, and CIDs.
- Null, derangement, anti-vacuity, frame, tie, mask, saturation, complement-adversary, and boundary tests.
- Current V1 regression plus candidate reference/packed differential and certificate replay if Phase D is reached.
- `cargo fmt --all -- --check`.
- `cargo test --workspace --all-features --offline`.
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`.
- Graph-format `--no-default-features` and the repository alloc/no-std ladder.
- Required wasm target when `uor-r4-core` or `uor-r4-router` is touched.
- `python3 scripts/check_claim_wording.py`, repo-conformance, deterministic rebuild/κ, and allocation/P-4 gates as touched.

# Experimental run contract

```text
metric to move:
  Primary: teacher attention-support recall/Jaccard versus r4-route-attention/1.
  Guardrails: teacher-forced top-1/top-k, bits/token, output error, candidate work,
  bytes read, artifact bytes, and deterministic runtime operations.

current baseline:
  Re-measure r4-route-attention/1 on the same pinned #603 fixture and frame.
  #605 synthetic rows and #310/#395 results are prior evidence/controls, not a
  substitute for a real baseline and not permission to invent unavailable values.

promotion rule (locked before held-out labels are read):
  Direct relation: at least +0.03 absolute support overlap/Jaccard over V1,
  teacher-forced top-1 no worse by more than 1 percentage point, and bits/token
  no worse than 1.01x.
  Prefilter/refine: retain at least 95% of V1 support, keep top-1 within 1
  percentage point and bits/token within 1.01x, while reducing evaluated
  candidates or bytes read by at least 25%.

reachability ceiling:
  Compute the oracle best support attainable after quotient collisions, empirical
  shell entropy, and exact/complement collision rate. Do not launch fitting when
  the ceiling cannot reach the applicable promotion rule.

cheap binding instrument:
  Paper-conformance oracle + exhaustive byte/pair checks + one pinned one-head
  trace collision/ceiling run with weight9, oriented, random-five-class, and
  shuffled-block controls before any packed code or long run.

null baseline:
  Current V1, weight9, occupancy-matched random five-class mapping, shuffled
  8-bit block partition, deranged teacher support, and #310 weighted Hamming
  where the budgets are comparable.

falsifier:
  Stop if complement folding merges semantically distinct exact/opposite blocks
  enough to miss the ceiling; if it does not beat occupancy-matched nulls; if a
  class-only result changes when anchor integers are relabeled; if the frame
  control fails; or if matched-budget runtime violates P-4/allocation constraints.

exit rule:
  Stop at the first conformance, ceiling, null, frame, anti-vacuity, fidelity, or
  runtime-feasibility failure. No threshold/candidate changes after held-out
  observation and no corpus-scale run before the one-head gate.

positive next action:
  Advance one scope at a time through #605/#606, register a separately versioned
  dormant candidate, and file a separate promotion issue for real evidence.

negative next action:
  Append the negative table and falsifier, classify the candidate dormant/reject
  under #515, preserve current V1, and close this issue successfully.

cost estimate:
  Cheap offline source audit, exhaustive byte tests, and one-head trace screen
  first; medium fit/certificate work only after the ceiling clears; no long
  corpus-scale experiment in the initial investigation.
```

# Compatibility and migration

This issue is additive and research-only. Existing graph formats, route-fit/operator identifiers, κ pins, model bundles, server behavior, and default engine selection remain unchanged. Any promoted representation requires a new versioned schema, explicit artifact migration/re-pin review, compatibility tests, and a separate activation decision.

# Claim status

- **Definition:** the Octeract byte map, shell quotient, candidate placements, masks, block layout, and nulls must be exact and versioned.
- **Implementation criterion:** executable conformance, determinism, reference/packed agreement, witness replay, and repository runtime gates.
- **Empirical criterion:** the predeclared trace ceiling and promotion thresholds above on pinned fixtures.
- **Not established:** paper correctness, categorical language, uniform-byte entropy, or finite computation does not establish attention quality, transformer equivalence, geometric-attention success, self-correction, production readiness, or language-model quality.
- **Disposition:** positive, negative, and unavailable outcomes all remain evidence. Only a separately reviewed promotion can change an active/default mechanism.

## Comments (3)

### comment 5286844890 — 2026-08-13T21:54:54Z — markrnd87-cmd

Hell yeah 

### comment 5299486467 — 2026-08-15T00:25:57Z — Casey-allard

Phase A is now active as #720 and assigned to Casey: https://github.com/UOR-Foundation/uor-r4/issues/720

Dependency/source audit corrections before implementation:

- #647 and #648 are closed (the issue body still calls them open); the #647 frame guard is available for the later screen.
- #603-#606 provide the trace/baseline/certificate surfaces, but no pinned real-checkpoint trace artifact was delivered.
- #531 closed without producing the past-knee saturation-corpus artifact/CID, so a missing real arm must remain typed `UNAVAILABLE`, never zero or `FAIL`.
- Neither supplied PDF contains a detected license or redistribution grant. Phase A records exact filename/size/SHA-256 with `license: NOASSERTION` and `redistribution: not-authorized`; no PDF, figure, or extended source prose will be committed.

The ordered gate is now explicit:

1. #720 freezes the direct oracle, closed form, corrections, exhaustive byte/pair tests, and source/claim map.
2. After #720 merges, a separate #661-B child will run the predeclared V1/weight9/fold/oriented/prefilter/lower-bound/null collision and reachability screen with the #647 control.
3. A one-head child is created only if that cheap ceiling clears; packed/runtime work remains conditional after the one-head gate.

No long run or operator/default change is authorized by Phase A. A stopped negative or typed unavailable disposition will be preserved as evidence under the parent contract.


### comment 5300241866 — 2026-08-15T03:08:19Z — Casey-allard

Closing #661 as a completed bounded investigation with disposition **retain-dormant / UNAVAILABLE**—not an empirical negative and not a rejection.

Phase A completed through #720 / PR #721, merge `598cba902c6d16fb9d7687601744728856a4197f`, binding both supplied sources and exhaustive byte/pair conformance.

Phase B completed through #722 / PR #723, merge `3941aa53c06e8101ce1a19fbddf1b343a010faf5`. No registered pinned-real `full/1` observation/records/trace/fit bundle existed, so the locked missing-prerequisite branch produced the canonical typed report:

- 6,587 bytes
- payload kappa `blake3:8b8c3bdc41f04ac2d6b9a15ef843f5064fae92e8b6bfe57cafbc6803eca7c5a2`
- envelope kappa `blake3:eab7b1bb12d9508d9815da0c4fbac248eab8b93b8258e717829542e41ac75e5e`
- disposition `UNAVAILABLE`

Synthetic data established instrument conformance only and produced no empirical PASS/FAIL or attention-quality claim. Per the preregistered unavailable branch, no #661-C, classification fallback, packed prototype, operator/default change, or serving change is created. Existing `r4-route-attention/1` remains unchanged.

The append-only evidence is in `docs/octeract_attention_661.md`; the full local and merge-queue verification ladders passed, including merge-group run [31860200118](https://github.com/UOR-Foundation/uor-r4/actions/runs/31860200118). If an exact real evidence bundle is delivered later, it requires a separately reviewed follow-up with new contract/report versions rather than keeping this bounded investigation parked open.



---
_Comment/review ids in this file: 5286844890, 5299486467, 5300241866_

