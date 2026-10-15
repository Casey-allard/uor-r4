# #605 — compiler: fit R4RouteAttentionV1 from standard-teacher traces with predeclared exit rules

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T22:01:18Z
- **Updated:** 2026-08-19T05:25:55Z
- **Closed:** 2026-08-12T19:27:09Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/605
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

An R4-native operator is not a recompilation merely because unchanged source weights are loaded into a new runtime. The target relation must be fitted or compiled from teacher evidence under a declared objective, with nulls and progressive replacement.

# Current evidence

- `crates/uor-r4-model-source/src/lib.rs:802-840` exposes source representation/behavior and final hidden/top-k evidence, but no target-operator fitting boundary.
- `crates/uor-r4-graph-compiler/src/induction.rs` fits graph covers and emissions from observations; it does not fit a distinct attention/relation operator.
- `docs/reconstruction_null_456.md` records that reconstruction bits can discriminate relative covers but remain sub-unigram; this is a falsifier against treating a structural metric as model quality.
- #307 measures deployed teacher parity, while #531 owns the corpus-scale saturation question. Neither establishes progressive replacement of a source attention operator with `R4RouteAttentionV1`.
- #604 is the blocked reference/paked target-operator issue; #603 is the trace boundary.

# Problem

There is no bounded experiment that fits route prototypes/masks/radii/residuals/projections from standard-teacher traces, measures replacement at increasing scope, and records positive and negative exits without conflating compilation success with model quality.

# Goal

Fit `R4RouteAttentionV1` from pinned standard-teacher traces under a predeclared run contract and produce a decision record for one-head, one-layer, layer-range, and whole-model replacement.

# Scope

- Define compiler-side fitting inputs and outputs using #603 profiles and #604 semantics.
- Fit route prototypes, masks, radii, residuals, and projections as applicable; identify which parameters are learned/compiled versus source weights.
- Run null baselines and progressive replacement: one head, one layer, a layer range, then whole model where the source adapter exposes the needed boundary.
- Record top-1/top-k, bits/token, attention-support overlap where meaningful, layer/output errors, artifact bytes, runtime bounds, and witness replay.
- Record explicit positive/negative next actions and preserve failed fits as evidence.

# Non-goals

- Do not change the source teacher or call unchanged Q/K/V weights valid under a new equation.
- Do not enlarge corpus scale outside #531’s selected experiment.
- Do not promote an operator or alter serving defaults.
- Do not add cross-repository arithmetic/registry/formal integrations blocked by #589.

# Dependencies and blockers

- **BLOCKED BY #515** for dormant mechanism/activation-gate conventions.
- Depends on #597, #599, #602, #603, and #604; consumes #531’s corpus decision.
- Existing Gate C/teacher-parity certificates from #307 must be extended rather than duplicated.

# Proposed implementation shape

Implement an offline fitting harness in compiler/certifier code with deterministic sample selection and ordered reductions. Keep the target operator ID and all input CIDs in the result, and make the replacement ladder independently runnable so a negative one-head result stops before an unjustified whole-model run.

# Acceptance criteria

- [ ] A pinned trace/source/corpus run produces a complete fit manifest and deterministic output.
- [ ] Null baselines and one-head/one-layer/layer-range/full-model results are reported separately, with absent stages represented as absent.
- [ ] The report distinguishes source parity, target-operator fit, runtime contract, and model-quality measurements.
- [ ] Predeclared positive and negative exit actions are recorded; negative results remain linked from the plan/ledger.
- [ ] No target fit is called complete when only an artifact compiles.

# Verification

- Deterministic double-run of fit parameters/artifact bytes on a pinned mini-corpus.
- Progressive replacement and null-arm measurements with source/top-k/hidden traces.
- Extend Gate C/teacher-parity/certification reports, plus witness replay, op census, no-float/no-multiply/no-allocation checks, and workspace gates.

# Compatibility and migration

Fitted outputs must carry source snapshot, tokenizer, adapter, trace, geometry, operator, corpus, and compiler CIDs. Existing artifacts and fixtures remain untouched; any new artifact era requires a reviewed migration/re-pin decision.

# Claim status

Experimental implementation and empirical criteria. It may report measured behavior, never exact equivalence or proof of quality.

# Experimental run contract

metric to move: teacher-forced top-1/top-k agreement and bits/token after progressive target-operator replacement, with support/error metrics as diagnostics.
current baseline: awaiting #531/#603/#604 prerequisites; do not invent a source/target parity number.
reachability ceiling: the source teacher’s recorded top-1/top-k and bits/token on the pinned corpus; report the actual measured ceiling when the prerequisite run exists.
cheap instrument and required verdict: one-head fit on a deterministic mini-corpus with nulls, source parity preflight, and operation/witness checks.
exit rule: stop at the first replacement stage that fails source parity, target semantics, runtime bounds, or the predeclared quality margin; no whole-model run follows a failed gate.
if positive: advance one replacement scope and record the new evidence.
if negative: preserve the fit/null report, keep the operator dormant, and file no quality claim.
cost estimate: compiler-side fitting plus repeated short teacher-forced evaluations; full-model scope is deliberately deferred until cheap stages pass.
pinned source CID: from #597/#599.
pinned corpus CID: from #531.
target operator ID: `R4RouteAttentionV1` from #604.


## Comments (11)

### comment 5270063288 — 2026-08-12T17:17:29Z — Casey-allard

## Plan + pre-registered run contract (posted before any run)

**Prerequisite state.** #597/#599/#600/#602/#603/#604 shipped (source manifest κ, adapter validation, `bucket-average/1` geometry, attention registry incl. `r4-route-attention/1`, TeacherTrace profiles + `.trace` sidecars, R4RouteAttentionV1 reference + packed lowering + witness). #531 CLOSED: scale law + `recommend-scale` + `scale_sweep.sh` shipped; β provisional 0.5; the past-knee saturation runs are compute-bound and not yet executed → **no pinned real-corpus CID exists**. The pinned SmolLM2 snapshot is likewise absent from the build environment. Per the #599 three-state convention, real-teacher/real-corpus ladder stages will be reported **UNAVAILABLE with reasons — never vacuous-PASS** — and the executable evidence is this issue's own "cheap instrument": a deterministic synthetic mini-corpus + synthetic teacher.

**Deliverables.**
1. `uor-r4-graph-compiler/src/route_fit.rs` — versioned fit method **`route-fit/1`** (canonical pinned-line-format bytes + blake3 declared-identity digest; registry fn refusing unknown (id,version), per the #600–#604 convention). Inputs: #603 trace records (attention-support + layer profiles) + #600 `bucket-average/1` projection. Outputs: R4RouteAttentionV1 instance parameters + a **FitManifest** carrying the eight CIDs (source snapshot, tokenizer, adapter, trace, geometry, operator, corpus, compiler); absent stages/params represented as absent, never zero-filled.
2. `route-fit/1` method, declared: per (layer, head), binarize query/key features via the #600 bucket-average projection to D=288 buckets + per-bit **lower-median sign threshold** computed over the fit sample in fixed order (compile-side float math — P-4 binds only the deployed kernel); v1 mask = full (recorded, not learned); radii absent; residual/projection absent (recorded). Learned/compiled vs source-weight parameters explicitly labeled in the manifest.
3. graph-certify: replacement-ladder runner — **null → one-head → one-layer → layer-range → whole-model** — each stage a three-state record (PASS/FAIL/UNAVAILABLE + reason) with the RunContract serialized into the report (pre-registration machine-carried). Replaced-path evaluation calls the real packed runtime step (`route_attention_step` over `build_route_attention_instance`) — never a reimplementation — plus witness replay, op census, no-float/no-multiply/no-allocation checks. Extends the #307 Gate C / teacher-parity report surfaces rather than duplicating them. **Replacement semantics v1 (recorded in the contract): teacher attention restricted to the fitted-selected top-M support and renormalized — isolating the selection mechanism R4RouteAttentionV1 provides from value mixing.**
4. Tests (`route_fit_605.rs`): deterministic double-run (byte+κ equality of manifest and fitted params), one-head fit vs both nulls, anti-vacuity arm, ladder stop-at-first-failure, UNAVAILABLE representation.
5. Ledger dormant lane `route-fit-dormant` (activation gate: positive ladder result on pinned real teacher + #531 corpus) + CONFORMANCE.md regen; `docs/MODEL_LIFECYCLE.md` #605 section.

**Pre-registered criterion (binding; no run has occurred).**
- Metric to move: teacher-forced top-1/top-k agreement and bits/token at each replaced scope; support overlap (Jaccard) as diagnostic.
- Nulls: **N1** seeded-random route codes (same shape); **N2** deranged-trace fit (fit against a derangement of the query→support mapping).
- Advance gate per stage (synthetic): source-parity preflight PASS ∧ runtime checks PASS (witness replay, census, 0-alloc packed steps) ∧ fitted support overlap ≥ max(2× best null, 0.5) ∧ top-1 agreement ≥ 0.90 ∧ replaced bits/token ≤ 1.10× teacher.
- Anti-vacuity: N2 must land < 0.5× fitted overlap, or the instrument is vacuous and the run is **invalid** (instrument must be able to fail).
- Exit rule: stop at the first stage failing any gate; no later stage runs; the negative is preserved in the report and ledger; the operator stays dormant.
- If positive through the synthetic ladder: record "method viable on synthetic; real-teacher run blocked on pinned snapshot + #531 saturation corpus," file the follow-up, **no quality claim**.
- If negative: preserve fit/null report, dormant, no quality claim.
- Compilation success is never called fit success; a fit is never called model quality (per `docs/reconstruction_null_456.md`).

Method-choice note: `route-fit/1` is v1 of a versioned registry — the median-threshold binarization + full-mask + selection-only-replacement choices are pinned and auditable, and alternative methods land as `route-fit/2+` without disturbing this record.


### comment 5271166964 — 2026-08-12T18:37:43Z — Casey-allard

## Delivery record — PR #644 (pre-registration: comment 5270063288, posted before any run)

**Acceptance criteria:**
- ✅ *Pinned run → complete fit manifest, deterministic output.* FitManifest carries the eight identity fields (source snapshot, tokenizer, adapter, trace, geometry, operator, corpus, compiler) with **typed absence** (synthetic arm: tokenizer genuinely `None`, corpus the pinned mini-corpus κ); deterministic double-run tests pin byte+κ equality of manifest, fitted params, and the full report.
- ✅ *Nulls and one-head/one-layer/layer-range/full-model reported separately; absent stages absent.* Stage records are PASS/FAIL/UNAVAILABLE/NotRun — four distinct states, round-trip tested. Ladder stops at first failure (tested by injection).
- ✅ *Report distinguishes source parity / target-operator fit / runtime contract / model quality.* Separate record fields under separate verdicts; the existing #307 Gate C parity row type is **embedded** (GateCMetrics + Deserialize/PartialEq/serde(default), serialized bytes unchanged) rather than duplicated.
- ✅ *Predeclared positive/negative exits recorded; negatives preserved.* RunContract serialized into every report; decision record carries both branches; ledger lane `route-fit-dormant` preserves the reading.
- ✅ *No fit called complete on compilation alone.* Kernel-level runtime checks (witness replay, census, RouteState epoch discipline) and fit gates are separate; selection evidence comes from the deployed packed kernel (`build_route_attention_instance` → `route_attention_step`), never a reimplementation.

**Measured (synthetic cheap instrument — deterministic 2L/2H/d32/vocab64 fixture teacher through the production #603 trace surface, 1024-token mini-corpus):**

| stage | overlap | N1 | N2 | top-1 | bits ratio | verdict |
|---|---|---|---|---|---|---|
| one-head | 0.5718 | 0.2019 | 0.2353 | 0.9941 | 0.9999 | PASS |
| one-layer | 0.5730 | 0.2039 | 0.2312 | 0.9922 | 1.0000 | PASS |
| layer-range | 0.5872 | 0.2064 | 0.2537 | 0.9951 | 1.0001 | PASS |
| whole-model | 0.5801 | 0.2052 | 0.2424 | 0.9902 | 1.0002 | PASS |

Instrument **valid** (N2 < 0.5× fitted everywhere). Real-teacher and real-corpus stages: **UNAVAILABLE** (pinned SmolLM2 snapshot absent from build env; #531 saturation corpus not yet produced — compute-bound). Per the contract: this is a cheap-instrument reading — **no model-quality claim exists**, the operator stays dormant, and the activation gate binds only to the real arm.

**Honest notes (also in code/docs):** (1) the first fixture teacher measured VACUOUS under the pre-registered anti-vacuity rule (N2 0.5272 ≥ 0.5× fitted 0.7183 — norm-hub keys made supports query-independent); the fixture was rebuilt with per-head QK L2-normalization and the gates/nulls/margins were not touched; both measurements are recorded. (2) `bucket-average/1` refuses sources narrower than 288, so route-fit/1 declares cyclic tiling as a versioned parameter. (3) Null-stage overlap semantics and the 2-layer LayerRange definition (1..=1) are pinned in the contract as data.

**Discovered work → filing as issues:** a shared #603 trace-row *reader* in `observation.rs` (writer's lane layout currently has only the #605 consumer-side decoder).


### comment 5271794694 — 2026-08-12T19:27:09Z — Casey-allard

Closing: PR #644 merged (2026-08-12T19:14:51Z). Delivery record: https://github.com/UOR-Foundation/uor-r4/issues/605#issuecomment-5271166964 (pre-registration posted before any run: comment 5270063288). route-fit/1 + pre-registered replacement ladder shipped dormant (route-fit-dormant); synthetic cheap instrument valid and 5/5 stages passed pre-registered gates; real-teacher/real-corpus arms UNAVAILABLE with reasons — no model-quality claim. Discovered work: #645 (shared #603 trace-row reader).

### comment 5331999821 — 2026-08-18T17:52:20Z — Casey-allard

Pointer (2026-08-18): the #786 dormant reassessment shortlists this issue's real-teacher arm as **S1** and found prerequisite A (pinned snapshot) now met, while prerequisite B surfaced a governance gap: #531 closed COMPLETED without a closure record and its saturation corpus is unlocated. The (a) produce-corpus vs (b) amend-contract decision is written up in `docs/dormant_reassessment_786.md` (PR #794) and on the #786 thread; the draft S1 contract finalizes here once that call is made. No run is scheduled until then.

### comment 5334219251 — 2026-08-18T21:21:31Z — Casey-allard

Decision update (2026-08-18): the (a)/(b) corpus question is resolved — **(b), amend to the #509 smollm2-360m-broad corpus first** (cheap-check-first; the saturation corpus can still follow a positive). The full decision structure, amended pre-registered contract, and run plan now live in **#804**, which tracks the route-attention family end to end.

### comment 5335360268 — 2026-08-18T23:13:47Z — Casey-allard

## Declared contract amendment — real-corpus arm (S1), posted before the run per the pre-registration discipline

Per the #786 shortlist, the #804 decision record (D1 + the profile and backend decisions of 2026-08-18), and the maintainer approvals recorded there, the real-teacher/real-corpus stages' two UNAVAILABLE prerequisites are resolved as follows. **Every pre-registered gate is unchanged** (support overlap vs the N1/N2 nulls, teacher-forced top-1 ≥ 0.90, bits ≤ 1.10× teacher, anti-vacuity N2 < 0.5× fitted; stop at first failing stage).

1. **Corpus (amends the "#531 saturation corpus" prerequisite).** The pinned broad corpus `smollm2-360m-broad` (#516): Simple-Wiki `.uor-models/corpora/simple-wiki-20231101/articles.jsonl`, articles `0..2999`, seq-len 128, teacher-forced at every in-story position, D3 partition rule — re-observed under the trace profile below (the trace pass reproduces the same article set and partition rule; #516's census was 2,994 clean stories / 360,924 records). #531's own saturation corpus remains unproduced; a positive here may motivate it later — it is no longer a prerequisite.
2. **Teacher snapshot.** The pinned `smollm2-360m-instruct` source (κ `blake3:eb23c3e8…`, the same snapshot the #516 run used), 32L × 15H (5 KV), hidden 960.
3. **Trace profile (#603).** `full/1`, declared layers `[0, 4, 9, 13, 18, 22, 27, 31]`, support cap S=8 (`PRIMARY_TOP_K`). Layout-derived row width **93,440 B** (smoke-verified byte-exactly); ≈ 33.7 GB of sidecars for the full corpus. The fit consumes the q/k lanes + attention supports through `load_route_trace_corpus`; the whole-model ladder stage runs over the declared 8-layer set, and widening the layer list is a follow-up pass gated on a positive.
4. **Observation machinery.** PR #807 (traced teacher-forced text pipeline + `observe-text --trace-profile`), merged to `main`.
5. **Matmul backend — recorded, not hidden.** The observation pass runs under the #804 **measurement-only Accelerate exception** (PR #808): the exact-GEMM teacher (#655-B2) measured >18 min for ONE 128-position traced article on the pinned M1 (a ~100+ hour full corpus), vs **7 s** under the exception (~18.3 pos/s; ≈ 5.5 h single-stream). Every run log carries `matmul=Accelerate cblas (observation-only exception #804)`. Consequence for the ladder: teacher-side logits/supports carry Accelerate low-order accumulation bits; all fitted-vs-teacher comparisons in this arm run against traces produced under the SAME backend, and the produced corpus is never silently mixed with exact-GEMM corpora.
6. **Run shape.** `observe-text` with `--workers 2 --shards 8`, resume-daemon wrapper (crash-safe whole-article checkpointing per #807), output outside the repo tree; measured-rate projection ≈ 3–6 h wall.
7. **Known fit-side caveat, to resolve before the ladder stage runs (not affecting the observation):** `load_route_trace_corpus` reconstructs each story's fed sequence as `[bos, next…]`; for a teacher-forced TEXT corpus, position 0's true input is the article's first token, not BOS. Before the ladder consumes this corpus, the loader gets a declared handling (read the recorded input token or an explicitly-recorded position-0 approximation) posted on this thread.

Observation launches tonight on the maintainer's confirmed window; results and κ provenance will be posted here, then the ladder stages run per the unchanged gates.


### comment 5335971782 — 2026-08-19T00:28:15Z — Casey-allard

## Declared amendment 2 — real-arm stage ordering + two recorded facts (posted before the ladder runs)

Reading the ladder machinery ahead of today's run surfaced one gap that shapes the run order. Declared here per the pre-registration discipline; **every gate value is unchanged**.

**Fact 1 — the real-arm replaced-forward does not exist yet.** `run_route_fit_ladder` is typed to the synthetic teacher; the real-teacher/real-corpus stage kinds are carried as data and hardcoded to `UNAVAILABLE`, and the `support-restrict-renormalize/1` replacement semantics exist only on the synthetic executor — the real SmolLM2 executor has **no restricted-attention forward**. Implementing it (mask + renormalize inside the real attention loop, with parity falsifiers) is its own slice, named **S1-2**.

**Fact 2 — the fit and the overlap gates are teacher-agnostic and CAN run today.** `fit_route_codes` consumes only the trace corpus (q/k rows + supports); the deployed-kernel selection evidence (`packed_selection_evidence`: witness replay, closed-form census, certify cross-check, epoch discipline, #653 cost vector) and the overlap gates (fitted-vs-teacher support Jaccard vs the N1/N2 nulls + anti-vacuity) need no teacher rerun.

**Amended real-arm ordering (gates unchanged, stop-at-first-fail preserved):**

1. **S1-1 (today, on the corpus finishing this morning):** load the traced `smollm2-360m-broad` bundle → `fit_route_codes` → per-head deployed-kernel evidence + overlap gates: fitted support overlap must clear the pre-registered margins over N1 (permuted codes) and N2 (deranged supports), anti-vacuity N2 < 0.5× fitted. This is the cheapest falsifier and the heart of the question — *does packed route-attention fit the real teacher's routing structure at all*. A FAIL here ends the ladder with a negative record and S1-2 is never built (exactly the cheap-check-first spirit of D1).
2. **S1-2 (only on an S1-1 PASS):** implement restricted-forward on the real executor, then the replaced-forward parity gates exactly as pre-registered (teacher-forced top-1 ≥ 0.90, bits ≤ 1.10× teacher).
3. #606 certificate rows and the #643 A/B follow only a full PASS, per their own contracts.

**BOS handling (maintainer decision, 2026-08-19):** the loader's `[bos, next…]` position-0 approximation is **accepted and documented** for S1-1 (~1/128 of supports at seq-len 128, within the gates' tolerance); the exact first-token derivation lands before any #606 certificate goes real.

**Runner note:** S1-1 needs a small entry point (the overlap machinery is currently private to the synthetic runner) — a `route-fit-real` CLI/test wiring slice, landing today ahead of the run.


### comment 5335988956 — 2026-08-19T00:30:38Z — Casey-allard

**Amendment 2 addendum (still before the run):** two load-bound parameters for S1-1, from the machinery itself: **(1)** `fit_route_codes` enforces the deployed operator's instance bound `ROUTE_MAX_CANDIDATES = 64` — stories are evaluated on their **first-64-position prefix** (the exact window the packed operator could ever serve; supports are self-contained within a prefix). **(2)** the full 33.7 GB trace cannot load into the 16 GB measurement box, so S1-1 evaluates a **pre-declared story subset: the first 1,000 story ordinals** (~all their prefix positions ≈ 60k+ steps — per-head overlap SEs far below the pre-registered margins), streamed shard-by-shard; the subset's own κs plus the parent corpus identity-bundle digest go in the report. Gates and margins unchanged. A wider follow-up load (streaming full-corpus) is S1-2-adjacent work if wanted after the verdict.

### comment 5336088666 — 2026-08-19T00:45:41Z — Casey-allard

**S1-1 runner landed** (PR #812, auto-merge armed): subset loader (`load_route_trace_corpus_subset`, first 1000 story ordinals × first 64 positions, domain-tagged subset κs under `uor-r4-route-trace-subset/1`), `run_real_arm_overlap_gates` (pre-registered order: vacuity → deployed-kernel runtime checks → overlap bar `max(null_factor × best_null, floor)`; N1 permuted codes, N2 deranged supports), and the `transformerless route-fit-real` CLI entry point — exactly the entry point the amendment (issuecomment-5335971782) said would land ahead of the run.

Gates at landing: workspace clippy `-D warnings` clean, fmt clean, compiler lib 203 passed, certify 41 passed — including the synthetic PASS + determinism + both seeded-gate falsifiers (overlap floor 1.1 → FAIL, N2 vacuity 0.0 → FAIL vacuous) and subset-loader equivalence/truncation vs the full loader.

On corpus completion (observation currently healthy, ~24% at last check) the declared invocation runs:

```
./target/release/r4 transformerless route-fit-real \
  --corpus ~/obs-wiki-360m-traced \
  --source .uor-models/sources/smollm2-360m-instruct \
  --stories 1000 --positions 64 \
  --out ~/s1_1_overlap_report.json
```

Corpus-completion record (manifest trace profile, merged κs, disk) gets posted here first; then the S1-1 verdict lands here and on #804. PASS → S1-2 restricted-forward slice per the amendment; FAIL → negative record, operator retained, #784-contract work next per the standing decision.


### comment 5337863280 — 2026-08-19T05:19:33Z — Casey-allard

**S1-0 corpus record — the traced observation pass is COMPLETE.** Posting per amendment item 6 before the ladder runs.

## Run identity

- **Command (as declared):** `observe-text --input .uor-models/corpora/simple-wiki-20231101/articles.jsonl --source .uor-models/sources/smollm2-360m-instruct --sequence-length 128 --shards 8 --workers 2 --trace-profile full/1 --trace-layers 0,4,9,13,18,22,27,31 --trace-support 8 --out ~/obs-wiki-360m-traced`, resume-looped by the declared daemon until completion.
- **Backend provenance (amendment item 5):** the maintainer-approved measurement-only Accelerate exception (#804 decision, PR #808) — binary built with `--features observation-blas-exception`, teacher matmuls via Accelerate cblas, provenance line printed at startup. Observation-only; no serving or fit path touches this backend.
- **Wall-clock:** started ~23:23Z 2026-08-18, `observe-text complete` at ~05:11Z 2026-08-19 (**~5h48m**, in line with the ~5.75h projection from the live rate at launch).

## Corpus facts

- **3000 / 3000 articles**, **361,693 records**, 256/256 shards complete (`shard_bits = 8`), `done = true`.
- **Merged κ:** `blake3:523075f86f3a8dc8d631ebc818fa1b99445e3378970ec7b0fb4ab98cb394fc0c` (merged.bin).
- **Input CID:** `blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf` (articles.jsonl).
- **Partition (recorded rule):** held-out = `blake3(article id)[0] % 5 == 0` → **289,471 construction / 72,222 held-out** records (2404 / 596 articles).
- **Teacher-forced bits/token:** 3.395 across the pass.
- **Truncation note:** 2558 of 3000 articles truncated at the teacher sequence length (128) — expected at this sequence budget, recorded for the eligibility accounting (positions are within-window by construction).
- **Trace pins (manifest-verified):** `trace_profile = full/1` (top_k 8, layer_indices [0,4,9,13,18,22,27,31]), **`trace_row_bytes = 93440`** — exactly the declared TraceRowLayout for 360M @ 8 layers, S=8.
- **Tokenizer adapter:** hf-byte-bpe/1 with CID pinned in the manifest; attention operator `standard-source-attention/2` recorded.
- **Disk:** 33 GB total output; 20 GB headroom remained.

## Next (per the amendment + runner note)

S1-1 executes now on the declared subset (first 1000 story ordinals × first 64 positions, domain-tagged subset κs) via the landed `route-fit-real` runner (PR #812), built from current `main` with **default features** — the fit/gate path never touches the observation-exception backend; binary provenance goes in the verdict post. BOS position-0 approximation stands documented (amendment item 7; exact fix required before any #606 certificate). Verdict lands here and on #804.


### comment 5337903765 — 2026-08-19T05:25:55Z — Casey-allard

**S1-1 VERDICT: FAIL — instrument VACUOUS** (pre-registered gate order: vacuity → runtime → overlap bar; the vacuity gate fired first and is decisive by contract). Report: `uor-r4-real-arm-overlap/1`, written to `~/s1_1_overlap_report.json` on the run machine.

## Provenance

- **Binary:** current `main` @ `2ee0790a`, **default features** (the fit/gate path never touches the observation-only Accelerate exception; that backend exists solely in the corpus pass recorded above, issuecomment-5337863280).
- **Subset (as amended):** first 1000 story ordinals × first 64 positions → **62,875 records / 1000 stories**; records κ `blake3:dc6a7a19…015a31`, trace κ `blake3:13815189…691434` (domain-tagged `uor-r4-route-trace-subset/1`); geometry 32L/15H/5KV/960D, declared layers [0,4,9,13,18,22,27,31], S=8.
- **Fit:** 120 heads (15 heads × 8 layers), top_m 8, fitted-params κ `blake3:80b8cf13…44f7eed`. Wall-clock ≈ 4 min end to end.

## The numbers

| quantity | value |
|---|---|
| fitted support overlap (aggregate, 6,585,000 eligible steps) | **0.3961** |
| N1 — permuted route codes (seed 1615859713) | 0.1918 |
| N2 — deranged supports (next-position cyclic) | **0.2922** |
| vacuity bar (N2 must be < 0.5 × fitted = 0.1981) | **FAILED** — vacuous |
| overlap bar (max(2.0 × best_null, 0.5) = 0.5844) | would also have failed (0.3961 < 0.5844) |
| per-head fitted min / median / max | 0.2690 / 0.4026 / 0.5175 |
| per-head N2 min / median / max | 0.1530 / 0.2925 / 0.4269 |
| heads individually vacuous (n2 ≥ 0.5 × fitted) | **119 / 120** (sole exception: layer 9 head 0 — fitted 0.4315, n2 0.2015, n1 0.1928) |

**Deployed-kernel runtime checks: ALL PASS** across 7,545,000 packed steps — witness replay, census closed-form, reference cross-check, and caller-owned RouteState epoch discipline. Cost census recorded in the report (per-step maxima within the declared shapes: 2304 xors/popcounts, 64 candidates; the aggregate run-total exceeds `route-cost-bounds-v1`'s totals — a bounds-declaration artifact of this run's 7.5M-step scale, not a kernel property; execution `Accepted`).

## Honest reading

1. **The fitted codes do recover real structure over chance**: 0.396 vs the N1 permuted-code floor 0.192 (≈2.07×). The kernel, packing, and fit machinery all work as built.
2. **But the instrument cannot attribute that structure to genuine per-position routing.** The N2 null — teacher supports shifted one position within the sequence — reaches 0.292, because **teacher attention supports are temporally smooth on this corpus**: adjacent positions attend to heavily-overlapping token sets. Whole-support top-8 overlap therefore cannot separate "the codes learned where this position attends" from "attention barely moves between neighboring positions." Per the pre-registered contract, that makes the instrument VACUOUS and the run a FAIL regardless of the other numbers.
3. The aggregate fitted-minus-N2 excess is +0.104, and exactly **one head in 120** individually clears the vacuity bar. That is not nothing — but the contract's whole point is that a claim this small under a vacuous instrument is not evidence, and we honor it.
4. **The temporal smoothness itself is a real finding**: any future support-prediction instrument for this family must beat temporal persistence, not chance. N2-style nulls should be primary, not secondary, in any successor contract.

## Consequences (per the amendment's pre-declared tree)

- **S1-2 (restricted-forward parity) does NOT run** — it was gated on an S1-1 PASS.
- **The route-attention operator is RETAINED dormant** behind its unchanged ledger gate (`r4-route-attention-dormant`); nothing activates, nothing is deleted. The #643 msa A/B stays dormant with it (its contract referenced a fitted baseline that now doesn't exist).
- Any future real-arm attempt needs a **new pre-registered contract** with a vacuity-robust instrument — candidates: excess-over-N2 as the primary pre-registered metric with its own bar, or skipping straight to restricted-forward parity (which measures end-task effect and is immune to this vacuity mode). Neither is scheduled; either would need maintainer approval.
- The BOS position-0 approximation (amendment item 7) is moot for #606 certification — no certificate is being produced from a FAIL.

Decision record + close on #804 follows. The #784-family next step (held for this verdict per the standing maintainer decision) now returns to the maintainer.



---
_Comment/review ids in this file: 5270063288, 5271166964, 5271794694, 5331999821, 5334219251, 5335360268, 5335971782, 5335988956, 5336088666, 5337863280, 5337903765_

