# #722 — research/#661-B: run Octeract route-trace collision and reachability screen

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-15T01:15:21Z
- **Updated:** 2026-08-15T03:08:30Z
- **Closed:** 2026-08-15T03:07:18Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/722
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

Parent #661 asks whether the complement-folded Octeract byte quotient has a useful, falsifiable seat in R4 route attention. #720, merged via PR #721, froze the supplied-source identities, independent direct oracle, closed form, finite corrections, and exhaustive byte/pair conformance before any trace labels were inspected.

This child owns Phase B only: one deterministic, predeclared collision and reachability screen over the existing #603/#605 route-trace surface. It does not fit a new operator, replace a head, build packed code, or change a serving path.

The existing owners are reused:

- #603: versioned `full/1` teacher traces, aligned q/k and attention-support lanes, typed provenance, `RouteTraceCorpus`, and deterministic sidecar identities;
- #604: dormant `r4-route-attention/1`, including reference/packed selection semantics and witness/census seams;
- #605: `route-fit/1`, `FittedRouteCodes`, causal-prefix candidates, stable top-M behavior, null/anti-vacuity conventions, and PASS/FAIL/UNAVAILABLE/NOT_RUN states;
- #606: later composed quality certificates; this child creates no competing certificate;
- #647: `frame_control_default`, under which a false-zero or mis-framed instrument is UNAVAILABLE rather than a negative candidate result;
- #720: bounded exact block distance, folded/oriented classes, source identities, and the safe masked-weight lower bound.

No pinned real-checkpoint `full/1` trace artifact was delivered with #603-#606. A missing trace remains a typed `UNAVAILABLE` prerequisite, not a synthetic substitute, zero row, FAIL, or vacuous PASS. The absent #531 saturation corpus is not required for this cheap screen; it remains a later quality-stage prerequisite.

# Goal

Freeze and execute one cheap, deterministic, one-head screen that determines whether either lossy Octeract attention candidate can possibly justify a later one-head replacement experiment.

Completion produces one canonical report with a reproducible `advance`, `stop-negative`, or `UNAVAILABLE` disposition. A deterministic unavailable record is a valid completion when the real trace prerequisite remains absent.

# Fixed evaluation scope

The contract is serialized before inspecting teacher-support labels:

- source: one explicitly supplied pinned real `full/1` #603 trace bundle carrying source, tokenizer, adapter, source-operator, corpus, profile, record, and trace identities;
- fitted representation: unchanged `route-fit/1`;
- operator baseline: unchanged `r4-route-attention/1`;
- head: layer `0`, head `0`;
- code width: 288 bits, interpreted as 36 natural 8-bit blocks in byte order;
- mask: current V1 full mask;
- candidates: causal prefix positions in ascending candidate-index order;
- selection width: `M = min(8, trace support cap)`;
- eligible steps: `candidate_count > M`;
- teacher target: recorded top-M attention-support positions for the same `(story, layer, head, step)`;
- aggregation: fixed-order arithmetic and mean per-step metrics; no sampling, head search, seed search, or threshold tuning.

A real trace lacking the declared head, required q/k and support lanes, complete identities, or at least one eligible step is `UNAVAILABLE`.

# Arms

For query byte `q_b`, candidate byte `k_b`, and full-byte mask, define:

```text
d_b = popcount(q_b XOR k_b)
D   = sum_b d_b
s_b = min(d_b, 8 - d_b)
F   = sum_b s_b
o_b = d_b > 4
```

The screen contains exactly these arms:

1. **V1 baseline.** Use existing packed/reference `r4-route-attention/1` selection by ascending `(D, candidate_index)`. An independent 36-block scalar sum is a cross-check and must select exactly the same candidates.
2. **`weight9` control.** Preserve the 36 exact `d_b in 0..=8` values and decode `sum(d_b)`. Scores and selections must equal V1 exactly. This is not a promotion candidate.
3. **`octeract-fold5` direct relation.** Preserve the 36 `s_b in 0..=4` classes and select by ascending `(F, candidate_index)`. Raw anchors `0/127/189/217/225` never enter scoring.
4. **`octeract-oriented` control.** Preserve `(s_b,o_b)`, reconstruct every `d_b`, and sum. Scores and selections must equal V1 exactly. This is a representation control, not a promotion candidate.
5. **`octeract-prefilter`.** For `N` candidates set `P=floor(3N/4)`. A step enters the work/fidelity metric only when `P>=M`. Rank by `(F,candidate_index)`, keep the first `P`, refine with exact V1 `(D,candidate_index)`, and select top-M. Report exact-refinement candidates avoided and fidelity to full V1. Because computing `F` still scans the flat table, this arm reports only a theoretical exact-refinement ceiling, not deployed byte/op/latency savings.
6. **Safe lower-bound control.** For each candidate compute `L=sum_b abs(popcount(q_b)-popcount(k_b))` and verify `L<=D`. Process candidates in ascending index order. After exact top-M is full, a later candidate may skip exact distance only when `L>=current_worst_selected_distance`; equal distance is safe because the later index loses the V1 tie. Final selection must equal V1. This non-Octeract control cannot independently advance an Octeract candidate.

# Nulls and controls

All null identities and seeds are serialized:

- **Occupancy-matched fold null**, seed `0x661B0001`: within each `(story,step,block)`, deterministically permute observed fold5 labels across causal candidate indices, preserving five-class occupancy while breaking candidate/support association.
- **Shuffled-block null**, seed `0x661B0002`: derive one fixed non-identity permutation of the 36 key-byte indices and compare each query block with the permuted key block.
- **Deranged-support null**: reuse #605's cyclic one-position support derangement within each sequence.
- **Prior weighted-Hamming row**: cite #310 where budgets are comparable; do not rerun or relabel it as an Octeract arm.
- **Anchor relabel control**: every bijective relabeling of the five shell identities must leave class-only results unchanged.

# Measurements

For each arm/null, report fixed-order finite counts for:

- eligible stories, steps, candidates, and teacher-support entries;
- exact block-weight and folded-shell occupancy, pooled and per block;
- empirical Shannon entropy, with no uniform-byte assumption;
- empirical discrete mutual information with teacher-support membership (`Y=1` exactly for recorded teacher top-M), both `I(score;Y)` and pooled per-block `I(class;Y)`, with no smoothing;
- mean teacher-support recall and Jaccard under the locked selection/tie rule;
- exact/complement collision counts;
- candidate groups sharing a complete 36-element fold signature while differing in weight9/oriented signature, including size distribution and within-group teacher-membership disagreement;
- fold5 shortlist false positives/negatives;
- prefilter V1-selection recall/Jaccard and exact-refinement fraction;
- lower-bound tightness and exact evaluations avoided.

## Collision-constrained oracle ceiling

For each eligible step, partition candidates by their complete 36-element fold5 signature. Within a signature, retain ascending candidate index. Compute the maximum teacher-support count any signature-only selector could place in M slots, allowing an oracle ordering of distinct signatures but no within-signature distinction beyond the locked tie. Use exact bounded DP/enumeration (`M<=8`, `N<=64`) over complete groups plus a possible final partial group.

Report mean maximum recall and Jaccard. This is an optimistic reachability ceiling, not a realizable fitted result or quality measurement.

# Frame and instrument validity

Before accepting a candidate or null verdict:

1. independently score V1 as the sum of 36 exact block distances;
2. compare it with existing `r4-route-attention/1` reference/packed selection on the same candidates;
3. require exact candidate IDs, distances, and tie order;
4. pass mean matched-pair Jaccard through `frame_control_default`;
5. require the deranged-support arm to be observably distinct from the matched row.

A non-finite/near-zero frame control, identity mismatch, invalid trace alignment, or vacuous null instrument makes the screen `UNAVAILABLE (frame/instrument mismatch suspected)`, never candidate FAIL.

# Locked advance and stop rules

Only direct `octeract-fold5` and `octeract-prefilter` can advance.

## Direct fold5 advances only when all hold

- prerequisite, conformance, frame, and anti-vacuity controls pass;
- collision-constrained oracle ceiling is at least V1 mean Jaccard plus `0.03`;
- realized mean teacher-support Jaccard is at least V1 plus `0.03`;
- realized Jaccard is strictly greater than every applicable occupancy-matched, shuffled-block, and deranged-support null.

## Fold5 prefilter advances only when all hold

- prerequisite, conformance, frame, and anti-vacuity controls pass;
- mean recall of full V1 selection is at least `0.95`;
- exact V1 refinement evaluates at most `0.75` of candidates over work-eligible steps;
- teacher-support Jaccard is strictly greater than applicable nulls;
- at least one work-eligible step exists.

## Controls

- `weight9` and `octeract-oriented` must be exactly equivalent to V1 or the instrument stops structurally.
- the lower-bound arm must select exactly V1. Positive pruning is adjacent performance evidence, not Octeract promotion.

# Verdict discipline

- `PASS`: a valid, framed real-trace arm clears its locked reachability gate;
- `FAIL`: a valid, framed real-trace arm runs and misses a locked gate;
- `UNAVAILABLE(reason)`: a required real trace/lane/identity is absent/incomplete, the declared domain is absent, or the frame/instrument is invalid;
- `NOT_RUN(reason)`: an arm is skipped after an earlier structural stop.

Synthetic #605 traces may test implementation, nulls, determinism, and anti-vacuity, but cannot produce an empirical PASS/FAIL for #661. They are labeled `instrument-conformance`; the real row remains `UNAVAILABLE`.

If the real trace is absent at delivery, emit canonical report bytes with typed absent identities and the exact missing-prerequisite reason, append the disposition to `docs/octeract_attention_661.md`, and do not create #661-C.

# Implementation shape

Add a certifier-side screen, for example:

```text
crates/uor-r4-graph-certify/src/octeract_trace_screen.rs
crates/uor-r4-graph-certify/tests/octeract_trace_screen_661.rs
```

It must:

- consume `RouteTraceCorpus` and `FittedRouteCodes`, not create a parallel reader/fitter;
- reuse #720 bounded primitives, #604 reference/packed V1 semantics, #647 frame handling, and #605 verdict/absence conventions;
- define `uor-r4-octeract-trace-screen-contract/1` and `uor-r4-octeract-trace-screen/1`;
- serialize the full contract, arms, seeds, fixture identities, metrics, stage verdicts/reasons, disposition, and report kappa into deterministic canonical bytes;
- bind both #720 source digests, observation identity-bundle digest, records kappa, trace kappa/profile digest, `route-fit/1` digest, fitted-parameter kappa, source-operator digest, and report kappa;
- accept the trace input explicitly; never search for or silently select a fixture.

Append the locked contract and final table to `docs/octeract_attention_661.md`; preserve Phase A and unavailable/negative history.

# Decision branches

- **At least one Octeract arm clears:** create #661-C with only clearing arms, at most two, for a one-head semantic replacement under #605/#606. Remain dormant.
- **Valid screen, no arm clears:** preserve the negative table, create no one-head child, and file only the bounded classification/cache-sharing fallback required by #661 scope item 9.
- **Real screen unavailable:** preserve the typed unavailable report; create neither #661-C nor the negative-only fallback; retain the mechanism dormant/unavailable.
- **Lower-bound control alone is promising:** preserve it and route indexed-representation/runtime-cost work to a separate performance issue; do not call it an Octeract result.

# Non-goals

- No PDF redistribution, new formal toolchain, or proof claim.
- No new trace pipeline, fit method, route-code encoding, mask, operator version, packed instance, artifact section, serving path, default, or kappa re-pin.
- No raw Octeract anchor integer as score.
- No post-hoc head, block layout, seed, null, metric, or threshold selection.
- No synthetic value presented as real-model evidence.
- No live checkpoint generation or corpus-scale generation run.
- No deployed performance claim from a flat-scan theoretical prefilter.
- No duplication of #606's certificate or repeat of #310/#395/#643/#648 under new terminology.

# Acceptance criteria

- [ ] #720 source/conformance identities are bound into the report.
- [ ] Contract, arms, null seeds, head, layout, tie rule, thresholds, and decision branches are serialized before real-trace inspection.
- [ ] V1 existing operator seam agrees exactly with the independent 36-block scalar scorer.
- [ ] `weight9` and oriented controls reproduce V1 scores/selections exactly.
- [ ] Fold5, prefilter/refine, lower bound, and all three nulls follow the definitions above without tuning.
- [ ] Report includes occupancy, entropy, MI, support recall/Jaccard, collision groups, complement disagreement, oracle ceiling, prefilter fidelity/work, and lower-bound tightness.
- [ ] Collision oracle matches brute force on exhaustive small fixtures and is bounded for `M<=8`, `N<=64`.
- [ ] #647 frame control is materially nonzero before accepting a null/FAIL.
- [ ] Missing/incomplete real inputs and frame faults are named `UNAVAILABLE`, distinct from FAIL/zero/NOT_RUN.
- [ ] Synthetic fixtures cannot construct a real empirical PASS/FAIL row.
- [ ] At most direct fold5 and prefilter can advance.
- [ ] Report bytes/kappa are identical across repeated runs.
- [ ] Evidence record is appended with identities, contract, table, nulls, and disposition.
- [ ] Current fit/operator/runtime/artifact/serving/default/historical bytes remain unchanged.

# Verification

Focused tests:

```bash
cargo test -p uor-r4-graph-certify --offline --test octeract_trace_screen_661
cargo test -p uor-r4-graph-certify --offline --test octeract_attention_661
cargo test -p uor-r4-graph-certify --offline --test route_attention_604
cargo test -p uor-r4-graph-certify --offline --test route_fit_605
```

Synthetic fixtures cover fold/complement adversaries, exact/tied V1 selection, weight9/oriented identity, prefilter pass/fidelity fail, lower-bound equality/safe skip, occupancy nulls, block permutation, frame mismatch, all four verdict states, brute-force oracle comparison, and deterministic report bytes/kappa.

Run the full repository fmt, clippy, workspace tests, graph-format no-std/alloc, claim wording, model/register/deferral/limit audits, cargo-deny, and diff checks. Wasm is required only if the final diff reaches a wasm-built surface.

# Experimental run contract

```text
metric to move:
  Direct: mean teacher-support Jaccard versus V1.
  Prefilter: V1-selection recall and exact-refinement candidate fraction.

fixed scope:
  pinned real full/1 trace; route-fit/1; layer 0/head 0; 288 bits as
  36 natural bytes; full mask; causal prefixes; M=min(8,support cap);
  candidate_count>M; lowest-index ties.

reachability ceiling:
  exact quotient-signature oracle under stable within-class order.
  Direct must reach V1 Jaccard +0.03.
  Prefilter must retain >=0.95 of V1 with <=0.75 exact refinement.

cheap instrument + binding verdict:
  #720 conformance + V1 scalar/operator identity + weight9/oriented identity
  + fold5/prefilter/lower-bound + occupancy/block/support nulls + #647 frame.
  A real framed trace is required for empirical PASS/FAIL.

exit rule:
  stop at the first prerequisite, conformance, identity, frame, anti-vacuity,
  reachability, null-separation, fidelity, or work-gate failure. Never tune
  the candidate, seed, threshold, head, or layout after observation.

if positive:
  create #661-C with only clearing arms, at most two, under #605/#606.

if negative:
  preserve the negative report; create no one-head child; trigger only the
  bounded classification/cache-sharing fallback.

if unavailable:
  preserve typed UNAVAILABLE with the missing trace/lane/identity or frame
  reason; create no one-head or negative-only fallback.

cost estimate:
  bounded certifier work plus O(steps*candidates*36) scan; seconds to minutes
  once a fixture exists; no checkpoint generation, fit redesign, packed work,
  or corpus-scale run.
```

# Compatibility and claim status

**Definition:** the contract, arms, nulls, identities, scope, metrics, gates, and report format.

**Guarantee** Status: **Structural** for executable V1/weight9/oriented identity, safe lower bound, deterministic null generation, collision-oracle correctness, and report identity.

**Empirical Criterion** Status: **Unproven** until a valid pinned real trace runs the locked reachability and null-separation gates.

This child does not establish attention quality, language-model quality, deployed savings, transformer equivalence, the source papers' categorical claims, production readiness, or promotion.

References #661.


## Comments (1)

### comment 5300241784 — 2026-08-15T03:08:17Z — Casey-allard

Delivered and merged through PR #723 at merge commit `3941aa53c06e8101ce1a19fbddf1b343a010faf5`.

The binding merge-group run is [31860200118](https://github.com/UOR-Foundation/uor-r4/actions/runs/31860200118); every required queue job passed: full tests/nextest/BDD/docs/no-std/deterministic rebuild/canonical kappa, all-features dormant tests, Gate C trend, wasm-pack, fuzz smoke, conformance/cargo-deny, and audit.

Final canonical missing-real result:

- report bytes: 6,587
- payload kappa: `blake3:8b8c3bdc41f04ac2d6b9a15ef843f5064fae92e8b6bfe57cafbc6803eca7c5a2`
- envelope kappa: `blake3:eab7b1bb12d9508d9815da0c4fbac248eab8b93b8258e717829542e41ac75e5e`
- disposition: typed `UNAVAILABLE`

No registered pinned-real `full/1` evidence bundle existed. Synthetic/manual evidence remained instrument-conformance-only and produced no empirical PASS/FAIL. Per the preregistered branch, this creates neither #661-C nor the negative-only fallback and changes no deployed operator, artifact format, or serving path.



---
_Comment/review ids in this file: 5300241784_

