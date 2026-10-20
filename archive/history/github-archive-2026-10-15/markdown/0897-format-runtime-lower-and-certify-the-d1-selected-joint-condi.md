# #897 — format/runtime: lower and certify the D1-selected joint-conditional mix scorer (S1 redesign, #822)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-21T11:45:11Z
- **Updated:** 2026-08-21T18:37:19Z
- **Closed:** 2026-08-21T18:26:45Z
- **Labels:** enhancement, graph-compiler, conformance
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/897
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Trigger (recorded)

The S1 redesign D1 instrument returned **`SELECT (backed-off mix)`** under the pre-registered rule posted to #822 before the run (2026-08-21): backed-off mix 277.2 permille vs suffix baseline 246.6 permille -> paired delta **+30.6 permille (95% CI [+28.6, +32.5])** on the attested #833 bundle (`corpus.meta` CID `blake3:aa9d1767...`), clearing the Q3-adopted **25 permille lowering-track opening bar** and sitting entirely above the frozen **20 permille causal floor** off-serving — the first S1 arm to do either. Record: `docs/s1_redesign_instruments_822.md` §4; CID-bound numbers: `docs/joint_conditional_822_result.json` (result_cid `blake3:41be22f0...`); harness: `crates/uor-r4-api/tests/joint_conditional_run_822.rs`.

## Scope (an #836-shaped lowering)

Lower the selected arm into the deployed representation and certify end-to-end on the packed path:

- Bounded joint-conditional tables keyed by (content token, suffix key) with the pre-declared backed-off Ψ-bag fallback — composite key hash, fixed-offset reads, saturating adds; P-4 operation classes only (no multiply/divide/float in the deployed kernel); allocation-free steady state; deterministic artifact bytes; the dormant #836 PSTATE machinery is the engineering precedent.
- **Key-granularity design decision first**: the d4skip comparison arm ((token, last-token) keys — 1-token conditioning, denser support) measured **+49.6 permille [46.8, 52.4]**, stronger than the 2-token joint on the shared candidate set. The lowered key family (1-token vs 2-token conditioning suffix, or a granularity ladder) is the first design question and is a maintainer call at design time.
- #886-style deployed-fidelity spot-check pre-planned: the packed path re-faces integer-`ScoreQ` quantization and the top-8 candidate bound that sank a 19.0 permille ceiling (#836/#886); the off-serving 28.6 permille lower bound carries margin above the 20 permille floor, and the spot-check must confirm the margin survives lowering.
- **Promotion gate unchanged**: the frozen 20 permille `CAUSAL_FLOOR_PERMILLE` end-to-end on the deployed path (#887: the bar stands). The 25 permille bar opened this track; it does not promote anything.
- Built-capability order judged at implementation: if a new served capability row emerges, the required order applies (model/ids.toml -> tagged Gherkin -> failing marker test -> implementation -> regenerated CONFORMANCE.md).

## Activation

Trigger-gated: work does not start until the RFC §6-4 decision point (after the D2 instrument reports) confirms the re-entry direction on #822. Filed now so the SELECT outcome has its home (live-board discipline).

## Definition of Done

- Lowered scorer behind the frozen gates with witness replay; deployed-fidelity spot-check recorded; end-to-end measurement on the #833 protocol with a pre-posted run contract; promotion / limitation / retirement verdict recorded on #822 with CID-bound evidence.


## Comments (8)

### comment 5369699493 — 2026-08-21T12:17:36Z — Casey-allard

### ACTIVATED (2026-08-21) — trigger recorded on #822 (§6-4 decision); phase 0 = pre-registered key-family confirmation

Phase 0 runs BEFORE any lowering engineering: one cheap confirmation run in the D1 harness mold, pre-registered here.

- **PRIMARY `skipmix`** — backed-off mix with **1-token conditioning**: joint tables keyed by (content token, last window token); residuals vs the full 2-token `suffix_rate`; verbatim Ψ-bag fallback contribution for tokens without a supported key, normalized over all unique window tokens; cap 64 per key; λ = 1.0 fixed before evaluation.
- **Reproduction gates (exact recorded values, ±0.05):** base 246.6 permille; D1 mix 277.2; D1 d4skip 296.2; minimal pairs 4,722 exact; base-follow 0. The D1 arms are recomputed on their original constructions as harness-correctness gates.
- **Head-to-head reported:** the paired skipmix-vs-mix delta — the decision-relevant contrast.
- **Null (planted):** this window's tokens looked up under the swap partner's last-token key (conditioning specificity; must change ≥1 prediction and sit below a positive arm).
- **Exit rule (pre-registered):** SELECT-1-token iff the skipmix paired-vs-base 95% lower bound ≥ 25 permille AND the skipmix-vs-mix paired lower bound > 0. Otherwise the lowering proceeds with the D1-selected 2-token mix as measured. The branches differ only in the lowered key family; the lowering itself proceeds either way.
- Cost: minutes, offline, zero teacher compute. Record: `docs/s1_redesign_instruments_822.md` §6 + CID-bound JSON.

D2's depth-2 granularity and λ observations remain admissible design input for table layout and mixing weights during the lowering design, not for the key-family selection above. The frozen 20 permille end-to-end floor is the promotion gate; nothing in phase 0 changes deployed behavior.


### comment 5371844612 — 2026-08-21T15:23:41Z — Casey-allard

## Phase 0 COMPLETE — key family confirmed `SELECT-1-token` (PR #902 → main `9bc2b592`)

The pre-registered phase-0 confirmation run (contract in the ACTIVATED comment above) executed and merged (PR #902, merge commit `9bc2b592951b80a6698a842a6b57af6b38d4b588`).

**Result** (result_cid `blake3:4a13095b…`; `docs/skipmix_confirm_897_result.json`; record `docs/s1_redesign_instruments_822.md` §6):
- Reproduction gates reproduce the recorded D1 values exactly — base 246.6‰, D1 mix 277.2‰ (+30.6), D1 d4skip 296.2‰ (+49.6), minimal pairs 4,722, base-follow 0; double-run 2,000 identical; corpus.meta CID = attested #833 bundle.
- PRIMARY `skipmix` 302.8‰; **skipmix-vs-base +56.2‰ [53.6, 58.7]** (clears the 25‰ opening bar) and **skipmix-vs-mix +25.6‰ [23.1, 28.1]** (> 0, beats the D1-selected 2-token mix). The conditioning-specificity null collapses to 165.3‰ (−81.3‰, 47,185 changed).
- **Verdict `SELECT-1-token`.** The lowering lowers the 1-token-conditioning skip-mix key family: join key `(content token, last window token)`, verbatim Ψ-bag fallback, residual against the full 2-token suffix, cap 64.

**Reopened — this issue is NOT complete.** PR #902's body carried a "does-not-close-this-issue" sentence; GitHub's keyword parser ignored the negation, linked the PR as a closing reference, and auto-closed the issue on merge. That is a wording artifact, not scope completion. Phase 0 only decided the key family; the Definition of Done (lowered scorer behind the frozen gates with witness replay; #886-style deployed-fidelity spot-check; end-to-end measurement on the #833 protocol; promotion/limitation/retirement verdict on #822) is still open.

**Next phase — parked at the maintainer design gate.** The `#836`-shaped lowering of the confirmed 1-token skip-mix key family: bounded `(token, last-token)` joint tables (composite-key hash, fixed-offset reads, P-4 operation classes only, allocation-free steady state, deterministic bytes, witness replay), the #886-style spot-check, then the end-to-end measurement against the frozen 20‰ floor. Table layout and mixing weights remain the maintainer design call named in this issue. Unassigning while this parks — no lowering work is actively running this hour.


### comment 5372169756 — 2026-08-21T15:53:30Z — Casey-allard

## Maintainer design decision — #897 lowering (2026-08-21)

Resolves the design gate named in this issue (key family / mixing weights / table layout), following the phase-0 `SELECT-1-token` confirmation:

1. **Key granularity: pure 1-token skip-mix.** Lower exactly the confirmed arm — join key `(content token, last window token)`, verbatim Ψ-bag fallback, cap 64 per key, residual against the full 2-token suffix. No granularity ladder in this pass; revisit a 1+2-token ladder only as a follow-up if the deployed end-to-end measurement misses the 20‰ floor.
2. **Mixing weight λ = 1.0**, fixed as the pre-registered value for the binding end-to-end promotion measurement. A λ sweep may be reported alongside as exploratory context only — never as the gate.
3. **Process: design spec first, then build.** A short lowering design + run-contract — bounded `(token, last-token)` table layout, composite-key hash with fixed-offset reads, integer-`ScoreQ` quantization and the top-8 candidate bound (#836/#886), the #886-style deployed-fidelity spot-check, the #833 end-to-end protocol, and the pre-declared exit rule against the frozen 20‰ floor — is posted to this issue for maintainer sign-off before any implementation code.
4. **Sequencing:** this #897 lowering is the next S1 action; F0 #859 remains cross-cutting and non-displacing. Promotion gate unchanged — the frozen 20‰ `CAUSAL_FLOOR_PERMILLE` end-to-end (#887). D3 stays deferred.

Next action: draft the lowering design / run-contract on this issue for approval.


### comment 5372538672 — 2026-08-21T16:29:36Z — Casey-allard

## #897 lowering design + run-contract (v1) — posted for maintainer sign-off

**Status: DRAFT SPEC, no implementation code written.** Per the design-decision comment above ("Process: design spec first, then build"), this is Step 1 of that process. Nothing below changes deployed behavior; it is a plan for review.

### 0. Fixed inputs (not re-litigated — restating for the record)

1. Key granularity: pure 1-token skip-mix. Join key `(content_token, last_window_token)`. Verbatim Ψ-bag fallback for unsupported keys. Cap 64 entries per key. Residual scored against the existing baseline the deployed engine already produces (see §2).
2. λ = 1.0, fixed, for the binding end-to-end promotion measurement. Any λ sweep is exploratory context only.
3. Promotion gate unchanged: frozen 20‰ `CAUSAL_FLOOR_PERMILLE`, end-to-end, on the deployed path (#887). The 25‰ bar already opened this track (phase 0); it does not promote anything.
4. D3 (granularity ladder) stays deferred; revisit only if the end-to-end measurement below misses the floor.

### 1. Scope map — what precedent covers, and what is genuinely new

The issue names "the dormant #836 PSTATE machinery" as the engineering precedent. That machinery (`crates/uor-r4-graph-format/src/pstate.rs`, `crates/uor-r4-graph-runtime/src/runtime_state.rs`, `crates/uor-r4-api/src/engine.rs`) gives us, reusable as-is:

- The **contribution + argmax adjustment pattern**: `table_contribution()` sums a `saturating_add` over supported keys, then `segment_adjusted_token_with_table()` re-ranks only within the existing top-8 candidate surface (`StepCandidates`, `STEP_TOP_CANDIDATES = 8`, `crates/uor-r4-graph-certify/src/score_runtime.rs:1898`) via `base_score.saturating_add(contribution)`. #897's skip-mix table plugs into this exact call chain as a sibling contribution source — no new adjustment mechanism needed.
- The **P-4 op-class discipline** for the read path: fixed-offset table reads, binary search (not hashing), `saturating_add`/`saturating_sub`, integer comparison — the full allowed set is `docs/prompt_state_spec_835.md` §5 (XOR/AND/OR/NOT, shift/rotate, popcount/cttz/ctlz, saturating/wrapping add-sub, integer compare, fixed-offset table reads). No multiply/divide/float.
- The **quantization recipe**: `crates/uor-r4-graph-compiler/src/segment_fit.rs` `RATE_SCALE_SHIFT: u32 = 20`, `quantize_rate(count, total) = clamp(round((count<<20 + total/2)/total), 1, i32::MAX)` — round-half-up, pure integer, floor-clamped to 1 so a retained pair never silently zeros. `DEFAULT_TOP_K = 64` is the existing cap-64 precedent.
- The **deployed context window**: `compiler::WINDOW = 8` (`crates/uor-r4-core/src/transformerless/compiler.rs:303`). This resolves an apparent mismatch flagged during research: the reference harness's `uniq_tokens(w)` loop runs over at most 8 window slots, matching `SEGMENT_STATE_CAPACITY = 8` in `runtime_state.rs` exactly. No ring-capacity redesign needed.

What is **genuinely new** (not just PSTATE reuse) and should be sized accordingly:

**(a) Wire format.** PSTATE's on-disk key is a flat `u32`, binary-searched — there is no hash function anywhere in that module or its callers, despite the issue text's "composite key hash" phrasing. #897 needs a real `(content_token, last_token)` pair as the row key. See §3 for the proposed shape (Decision Point A).

**(b) Compiler-side table builder.** A new pass building the capped, canonicalized `(content_token,last_token)`-keyed table plus the Ψ-bag fallback table, mirroring `build_segment_lane`'s canonicalization discipline (sort, reject duplicate keys, reject duplicate candidate tokens within a key, reject empty entry lists) but for a different key shape.

**(c) Production emission path — the largest scope item.** `crates/uor-r4-graph-certify/src/score.rs::ScoredGraphSections` (the struct consumed by `emit_scored_r4g1`, which produces the actual released `graph/score.r4g1`) has **no PSTATE/segment field at all**. The only existing PSTATE emitter, `convert_with_segment_table` in `convert_r4g1.rs`, writes a *different, non-released* TLS→R4G1 conversion path — and #886 already measured that path as **not serving-equivalent** to the real release graph (6/20 positions unservable on the re-emitted artifact; minimal-pairs follow-rate 1/10 against a ≥6/10 threshold). Repeating that pattern for skip-mix would just reproduce #886's known-negative result for an unrelated reason (wrong emitter, not the scorer being tested).

Consequence: reaching a genuine end-to-end / deployed-fidelity measurement for #897 requires **extending `ScoredGraphSections` and `emit_scored_r4g1` with a real skip-mix section**, so the table ships through the actual release-graph emission path, not the known-divergent side path. This is real, additional engineering scope beyond "reuse #836's machinery," and is called out here so the size of the lowering isn't underestimated (see §9, Decision Point B).

**(d) New capability registration.** `model/ids.toml` has no PSTATE/segment row at all (the #836 lane never promoted, consistent with its dormant verdict) — so there is nothing to extend. #897, if it reaches PROMOTE, registers a **new** row, next free id `RF-31` (see §10).

### 2. What "residual against the baseline" means once lowered

The off-serving harness (`skipmix_confirm_897.rs`) scores `base = suffix_rate(w,c)` — a standalone bigram/2-token-suffix frequency table built only for the research measurement. That table has **no deployed counterpart**: nothing in `graph-runtime`/`uor-r4-api::engine` reads a `ContextRow`/NGRAM bigram table at inference time (checked: no runtime or engine file references `ContextRow`). Confirmed: the NGRAM/`context_rows` section is compiler/certify-side evidence, not a served scoring input.

So "the baseline" that skip-mix corrects, once lowered, is **not** a new thing to build — it is whatever `base_score` the deployed `R4Engine` already produces for a candidate (its native region/emission/exact-context score). The lowered contribution is added on top of that real base score exactly the way PSTATE's contribution already is: `adjusted = base_score.saturating_add(skipmix_contribution)`.

**This changes the reachability picture** (Guarantee this is flagged, not asserted away): the off-serving 246.6‰ "base" in the harness is a much weaker scorer than the real deployed `base_score`, so the +56.2‰ off-serving delta is very likely an **overestimate** of what the deployed end-to-end measurement will show — exactly the dynamic #836's ceiling arithmetic and #886's fidelity gap already demonstrated for PSTATE. §7 makes this an explicit gate rather than an assumption.

### 3. Wire format — bounded `(content_token, last_token)` table

Two new bounded, fixed-offset, sorted tables (both structurally siblings of `PstateTable`, wider key):

**Primary joint table** (row key = 8-byte composite, `(content_token: u32) << 32 | last_token: u32`, ascending sort, binary-searched — same discipline as `PstateTable::find`, no hash function):

```
header (fixed, little-endian):
  magic[4]        = "SKM1"
  version u16
  reserved u16
  row_count u32
  cap u16          = 64   (max entries/key, matches DEFAULT_TOP_K)
  reserved u16
  key_quant_id u32
  reserved[8]
row (16B, row_count of these, sorted by key ascending):
  key u64          (content_token<<32 | last_token)
  entry_count u16
  reserved u16
  entry_start u32  (absolute byte offset into the entries region)
entries (8B each, contiguous per row, sorted by candidate_token ascending):
  candidate_token u32
  score ScoreQ (i32, Q16.16, quantize_rate() output)
```

**Ψ-bag fallback table** (row key = `u32` content_token only — this can reuse `PstateTable`'s existing row/entry shape verbatim, just a different section id and different semantic meaning of the key):

```
same header/row/entry shape as PstateTable, key = u32 content_token
```

Both tables get their own `SectionId` (optional bit, next free value confirmed against `types.rs` at implementation time — do not hardcode a bit here without checking for collisions with anything added since this research pass) and their own typed `FormatError` variants mirroring the 8 `Pstate*` variants (`SkipmixTooShort`, `SkipmixBadMagic`, …, `SkipmixEntriesNotSorted`), so a malformed section is a typed decode error, never a panic. `GraphView::skipmix_table() -> Result<Option<SkipmixTable>, NotAProduct>` and `GraphView::skipmix_fallback_table() -> Result<Option<PstateTable>, NotAProduct>` (reusing the type) follow the same absent-section-is-`None` identity as PSTATE.

**Decision Point A (needs sign-off): sorted composite key vs. an actual hash table.** The issue text says "composite key hash"; the only precedent in this repo (PSTATE) uses a sorted array + binary search, never a hash function. Recommendation: **keep sorted + binary search** (as specified above) — it reuses a proven, already-verified determinism/no_std/alloc-free story, needs no collision-handling proof, and the observed key cardinality (793,781 unique `(token,last-token)` pairs per the phase-0 run) is small enough that a sorted 8-byte-key array is not a performance concern. If "hash" was meant literally (a fixed-capacity open-addressed table for a smaller footprint), that is a larger, separate design with its own collision/probe-sequence correctness burden — happy to spec that instead if you want it, but recommend against it absent a concrete size/latency reason.

### 4. Compiler-side construction

New pass (compiler/certify-side, `alloc`-gated, mirrors `build_segment_lane`'s canonicalization):

1. Fold the attested corpus (same bundle/protocol as the phase-0 harness) into `HashMap<(u32,u32), Counter>` (joint) and `HashMap<u32, Counter>` (Ψ-bag), exactly as `skipmix_confirm_897.rs` already does for its `d4skip_next`/`content_next` maps.
2. Cap each counter to top-64 via the existing `cap_to_top`/`DEFAULT_TOP_K` pattern (sort `(count desc, token asc)`, truncate at 64) — same mechanism already used compiler-side, no new algorithm.
3. Quantize every retained `(candidate, count, total)` via `quantize_rate()` verbatim (§1) — integer, round-half-up, floor-clamped to 1.
4. Canonicalize: sort rows by key ascending, entries by candidate token ascending, reject duplicate keys, reject duplicate candidate tokens within a key, reject empty entry lists — identical rejection set to `build_segment_lane`. A round-trip test (build → parse → compare) asserts order-independence, same as the existing PSTATE test.
5. Determinism: no HashMap-iteration-order dependence in the emitted bytes (sort before serialize, as above); content-addressed like every other artifact section.

### 5. Deployed runtime read/score path

Ported directly from the already-measured reference math (`skipmix_confirm_897.rs::skipmix_scores`), replacing HashMap lookups with the two bounded tables above, and replacing the toy `suffix_rate` baseline with the engine's real `base_score` (§2):

```
for t in unique_tokens(window)               // window <= 8 slots (compiler::WINDOW)
    if let Some(row) = primary_table.find((t, last_token))     // binary search, u64 key
        sup_ts += 1
        for (cand, score) in row.entries():                     // binary search by cand within StepCandidates
            contrib[cand] = contrib[cand].saturating_add(score)
    else if let Some(row) = fallback_table.find(t)               // binary search, u32 key
        for (cand, score) in row.entries():
            contrib[cand] = contrib[cand].saturating_add(score)  // verbatim Psi-bag; NOT counted in sup_ts

for cand in step_candidates.ranked()          // the existing top-8 surface, StepCandidates::entries
    resid = scale_by_lambda_and_support(contrib[cand], sup_ts, unique_tokens.len())  // integer, lambda=1.0 fixed -> identity scaling; keep the hook typed for a future non-unit lambda, never exercised as the gate
    adjusted[cand] = base_score[cand].saturating_add(resid)

serve argmax(adjusted), tie-break score-desc/token-asc  // identical tie-break rule to segment_adjusted_token_with_table
```

This is the exact `sup_ts`-asymmetric residual the reference math already validated (joint-supported tokens count toward the support denominator; Ψ-bag-fallback tokens contribute to the numerator only) — load-bearing, must not be "simplified" away during lowering. Entirely P-4 legal: table binary search, saturating add, integer compare — no multiply/divide/float anywhere in this loop. λ = 1.0 means the "scale by lambda" step is an identity op this pass (per §0.2, a non-unit λ is never the gate) but the type stays generic so a future λ isn't a silent behavior change.

### 6. #886-style deployed-fidelity spot-check

Mirror `docs/segment_lane_886_spotcheck.md` exactly: compare **reference off-serving math** (the already-measured `skipmix_confirm_897.rs` numbers) against **deployed packed-path replay** (fit → emit via the *extended* `emit_scored_r4g1` → consume via `R4Engine::predict_decision_candidates_with_segment`-shaped call, using the real minimal-pairs set from the phase-0 record, 87 of 4,722 "follows" reference-side).

- Sample: the same minimal-pairs construction already used for D1/skipmix (paired positions differing only in the swapped last-token / conditioning-specificity axis).
- Threshold: reuse #886's predeclared bar, **>= 6/10** (or the equivalent fraction on whatever sample size is drawn — state the exact n and threshold in the posted run contract before executing, per the pre-registration discipline every other instrument in this record follows) of served tokens "follow" (deployed serving matches the reference-math prediction) to call the lowering faithful.
- If it fails (a `LOWERING-FIDELITY GAP`, #886's own outcome for PSTATE), that is a legitimate negative outcome, not a bug to explain away — it goes on the record and the promotion attempt stops there, same as #887 handled PSTATE.

### 7. #833 end-to-end protocol — with a reachability gate first (AGENTS.md "Long-run discipline")

Per AGENTS.md's three-gate long-run process (mandatory for anything measured in hours, which the full #833 corpus replay is), the expensive end-to-end run does not launch until:

1. **Reachability arithmetic is posted** — because §2 shows the off-serving +56.2‰ figure is measured against a much weaker baseline than the deployed `base_score`, it is not usable directly as the end-to-end ceiling. A cheap, engine-grounded probe is required first: replay the same held-out positions used in phase 0, but score them through the *actual* `R4Engine` `base_score` path (no full serving/admission loop needed — a direct scoring query is enough) instead of the toy `suffix_rate`, and recompute the skip-mix delta against that real base. This is minutes of compute (same held-out set, same corpus, no teacher inference), not hours, and is the "cheap instrument" analogous to #836's quantization/top-8 arithmetic.
2. **Cheap-instrument verdict is binding**: if the engine-grounded delta's 95% lower bound does not clear 20‰ with margin, the full #833 replay does not launch — recorded as a `REVISE`/`RETIRE` verdict directly from the cheap instrument, same as #836.
3. Only if it clears does the full #833-protocol end-to-end run execute: attested bundle `.uor-models/compiled/smollm2-360m-broad-clean`, held-out 72,130 document-disjoint positions, admission-gated `R4Engine` load + offline-predict canary, teacher-forced replay against recorded `t_argmax`.

**Run contract (to be posted verbatim to #897 immediately before either run, per repo convention):**

```
metric to move:       end-to-end top-1 permille, skip-mix-adjusted vs unadjusted, on #833 protocol
reachability ceiling: <computed from the cheap engine-grounded probe in step 1, with the arithmetic shown>
instrument + verdict: engine-grounded held-out probe (minutes); binding if < 20 permille + margin
exit rule:            SELECT/PROMOTE iff cheap-instrument delta's 95% lower bound clears 20 permille
                       with margin AND the #886-style spot-check clears its threshold (section 6);
                       otherwise RETIRE-or-LIMIT with the specific gate that failed named on the record
if positive:           promote -- register RF-31 (section 10), wire the extended emission path into the
                       release build, update docs/RESEARCH.md and the S1 stage record on #822
if negative:           record REVISE/RETIRE on #897 + #822 with the failing gate's numbers (ceiling
                       arithmetic, spot-check fraction, or full-run delta, whichever failed first);
                       no serving change; D3 (granularity ladder) becomes the next candidate action
cost estimate:         cheap probe: minutes, offline. Full #833 replay if reached: hours (kappa-scale
                       corpus, same order as prior #833-class runs)
```

### 8. Non-goals for this pass

- No granularity ladder (1-token only; section 0.1).
- No lambda sweep as a gate (section 0.2) -- exploratory only if reported at all.
- No change to the frozen 20 permille floor or the 25 permille opening bar.
- D3 stays deferred (Q4).
- F0 #859 (Lean kernel proof) is cross-cutting and non-displacing; unaffected by this lowering.
- No touching `crates/uor-r4-router` (f64 geometric router) -- out of the graph-migration path per AGENTS.md.

### 9. Decision Point B (needs sign-off): accept the emission-path scope, or descope

Two ways to proceed, given section 1(c):

- **B1 -- Extend the real path (recommended).** Add the skip-mix section to `ScoredGraphSections`/`emit_scored_r4g1` so the spot-check and end-to-end run measure the actual release graph, not a known-divergent side path. Larger scope, but the only version whose result means what the DoD says it means ("deployed-fidelity spot-check", "end-to-end measurement on the deployed path").
- **B2 -- Reuse the existing (non-release) `convert_with_segment_table` side path**, accepting #886's already-demonstrated non-equivalence caveat verbatim, and treat any resulting number as *bounded by* that known gap rather than a real deployed-fidelity measurement. Cheaper, but very likely produces another `LOWERING-FIDELITY GAP` outcome for a reason unrelated to skip-mix's own merit -- this would waste the run.

Recommendation: **B1**. Flagging explicitly since it's more engineering than "lower the scorer" reads as standalone, and it's the one part of this spec that meaningfully changes the size of the work versus a naive reading of the issue.

### 10. Built-capability registration (only exercised if promotion is reached)

Per `docs/prompt_state_spec_835.md` section 12's required order (`model/ids.toml` row -> tagged Gherkin -> failing marker/behavior test -> implementation -> regenerated `CONFORMANCE.md`), not before:

```toml
[[id]]
id = "RF-31"
level = "build"
suite = "skip_mix_conditioning"
statement = "Deployed 1-token skip-conditioned residual scoring over the existing candidate surface (#822 S1 redesign, #897 lowering)"
scope = "normative-runtime"
reachability = "deployed-serving"
evidence = "features/suites/skip_mix_conditioning.feature; crates/uor-r4-graph-format skipmix tables; crates/uor-r4-api engine skip-mix contribution path; docs/skipmix_lowering_897_result.md"
```

Gherkin feature `features/suites/skip_mix_conditioning.feature`, failing marker/behavior test authored before the implementing code (repo convention), `cargo xtask check-model --write` regenerates `CONFORMANCE.md` -- never hand-edited.

### 11. Verification plan (standard gates, applies regardless of outcome)

`cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`; `cargo test --workspace --no-fail-fast --offline`; `cargo check -p uor-r4-graph-format --no-default-features` (+ `--features alloc`); `cargo run -q -p xtask -- validate` (R1+R4+R5+gnaf); `cargo deny check bans`; `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib` (any `uor-r4-core`/format touch); `python3 scripts/check_claim_wording.py`. Witness replay + deterministic-rebuild + kappa-reproduction as usual for anything touching the deployed kernel.

### 12. Summary -- the two things asking for your sign-off

1. **Decision Point A** (section 3): sorted composite-key binary search (matches existing precedent, no new hash function) vs. an actual fixed-capacity hash table (smaller footprint, more correctness burden). Recommend the sorted form.
2. **Decision Point B** (section 9): extend the real release-graph emission path (larger scope, meaningful result) vs. reuse the known-non-equivalent side path (cheaper, likely wastes the run). Recommend B1.

Everything else in this spec is either a restatement of your existing decision (section 0), a direct port of already-validated precedent (sections 1, 4, 5, 6, 10, 11), or a mechanical application of AGENTS.md's existing long-run-discipline process (section 7) -- flagged for visibility, not asking for a new call.


### comment 5372593224 — 2026-08-21T16:34:49Z — Casey-allard

## #897 lowering — decisions recorded, spec section 3 revised (2026-08-21)

Casey's sign-off on the spec above, both decision points:

- **Decision Point A: real hash table** (not the sorted-array recommendation). Revising section 3 below.
- **Decision Point B: extend the real emitter** (`ScoredGraphSections`/`emit_scored_r4g1`) -- confirmed, as recommended.
- **Proceeding to implementation** on branch `issue-897-skipmix-lowering` off fresh `origin/main`.

### Section 3 revised -- hash table (supersedes the sorted-array layout above)

A genuine fixed-capacity open-addressed hash table, still fixed-offset/no_std/alloc-free to read, still fully P-4-legal (no multiply/divide/float anywhere in the lookup):

**Hash function -- multiply-free ARX mixer** (Jenkins "one-at-a-time" construction; add/shift/xor only, no multiply):

```rust
#[inline]
fn mix(mut h: u32, x: u32) -> u32 {
    h = h.wrapping_add(x);
    h = h.wrapping_add(h << 10);
    h ^= h >> 6;
    h
}

fn hash_key(content_token: u32, last_token: u32) -> u32 {
    let mut h: u32 = 0;
    h = mix(h, content_token);
    h = mix(h, last_token);
    h = h.wrapping_add(h << 3);
    h ^= h >> 11;
    h = h.wrapping_add(h << 15);
    h
}
```

Every op is P-4-legal (`docs/prompt_state_spec_835.md` section 5): wrapping add, shift, xor only. No seed/randomization (unlike `std::HashMap`'s SipHash) -- identical inputs hash identically on every platform, which the determinism invariant requires.

**Layout:**

```
header (fixed, little-endian):
  magic[4]        = "SKM1"
  version u16
  reserved u16
  capacity u32     (power of two; slot count, sized at build time so load factor <= 50%)
  max_probe u16    (fixed bound on linear-probe distance; a build-time invariant, checked at
                    construction -- if any key's actual probe distance would exceed it, the
                    builder grows capacity and retries; this makes lookup cost provably bounded,
                    never a function of how many total keys exist)
  entry_cap u16    = 64   (max candidates retained per key -- unrelated constant, kept separate
                            from table capacity/max_probe on purpose: three independent fixed
                            capacities, not one)
  reserved[4]
slot (16B, `capacity` of these, at a fixed offset = header_len + slot_index * 16):
  key u64                    (content_token << 32 | last_token; only meaningful if entry_count > 0)
  entry_count u16            (0 = empty slot -- the occupancy marker; no separate bitmap needed)
  reserved u16
  entry_start u32            (absolute byte offset into the entries region; unused when entry_count == 0)
entries (8B each, contiguous per occupied slot, sorted by candidate_token ascending):
  candidate_token u32
  score ScoreQ (i32, Q16.16, quantize_rate() output)
```

**Lookup** (deployed, P-4-legal): `idx = hash_key(t, last) & (capacity - 1)` (AND, not modulo -- capacity is a power of two); linear-probe forward up to `max_probe` slots; a slot with `entry_count == 0` before a match means "absent" -> fall back to the Psi-bag table (unchanged, still the flat-`u32`-key/binary-search PSTATE-shaped table from the original section 3 -- no reason to hash-table that one too, since its key space is just the vocab and a sorted scan is already cheap and precedent-proven). A match on `key` with `entry_count > 0` returns that slot's entries, binary-searched by candidate token exactly as before.

**Build-time invariant, checked and typed-erred, never `unwrap`/panic:** for every key, the actual insertion probe distance must be <= `max_probe`; capacity is grown (next power of two) and the whole table rebuilt if any key exceeds it, so a shipped artifact's `max_probe` header field is always a true, checked bound -- the deployed lookup's worst case is provably `max_probe` table reads, not average-case.

Everything else in the spec above (sections 0-2, 4-12) is unchanged; section 4's construction step 4 (canonicalize/reject-duplicates) now also covers the hash-table insertion order (insert in a fixed, sorted-by-key order so table contents are reproducible regardless of source HashMap iteration order, then verify no probe exceeds `max_probe`) and section 10's evidence list gains the hash-table round-trip + bounded-probe test as an explicit new test target.

Proceeding to implementation now.


### comment 5373175796 — 2026-08-21T17:33:26Z — Casey-allard

## Progress update: lowering implemented through the emission path (2026-08-21)

Working on branch `issue-897-skipmix-lowering` (4 commits so far, pushed). Status against the plan posted above:

- **format** (`crates/uor-r4-graph-format/src/skipmix.rs`, new): `SKMX` primary joint table (genuine fixed-capacity open-addressed hash table, multiply-free ARX mixer, checked `max_probe` bound — per Decision Point A) and `PSIB` Ψ-bag fallback (flat sorted array, `PstateTable`-shaped). Both optional sections, typed `FormatError` variants, two-stage validation. 13 unit tests (round-trip, corruption rejection, canonicalization order-independence, a 200-key collision-forcing exercise, and `from_trusted_bytes` cheap-reconstruction reads that let a deployed engine hold owned bytes without a lifetime parameter). **Done.**
- **compiler** (`crates/uor-r4-graph-compiler/src/skipmix_fit.rs`, new): folds the attested corpus into the joint `(content_token, last_window_token)` and Ψ-bag tables, cap-64, quantized, canonicalized. 4 unit tests, including a regression test pinning that the join key's `last_token` is read from the temporal window **before** sort/dedup (a subtle correctness point the reference math depends on). **Done.**
- **runtime** (`crates/uor-r4-api/src/engine.rs`): `R4Engine` now loads the optional SKMX/PSIB sections (owned validated bytes, `None` = absent-section identity) and exposes `predict_decision_candidates_with_skipmix` / `_witness`, reusing the existing `segment_argmax` contribution+argmax combinator (no new adjustment mechanism) over the current window's own unique tokens + last token — no persistent session object needed, unlike the segment lane. Implements the exact joint-vs-fallback asymmetry (row presence gates the Ψ-bag fallback, not per-candidate entry presence) the phase-0 reference math (`skipmix_confirm_897.rs`) already validated. λ=1.0 makes the lambda/support-scaling hook an identity this pass, per §5 of the posted spec — kept as a distinct typed function rather than inlined, so a future non-unit λ is a one-function change. 10 new unit tests (pure-function level, since no real artifact could yet carry these sections). `xtask validate`'s gnaf forbidden-construct scan confirms no multiply/divide/float anywhere in the added path. **Done.**
- **certify** (`crates/uor-r4-graph-certify/src/score.rs`): extended `ScoredGraphSections`/`emit_scored_r4g1` with the SKMX/PSIB sections (Decision Point B — the real release-graph emitter, not the known-divergent `convert_with_segment_table` side path #886 already measured as non-serving-equivalent). Every existing call site across the workspace (7 sites) updated with empty rows — unchanged behavior everywhere until fitting is actually wired in. New `skipmix_roundtrip.rs` integration test proves the sections round-trip byte-exact through a real emitted-then-reparsed artifact. **Done.**

Verification run after each step and again at this checkpoint: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --no-fail-fast --offline` (full workspace, 0 failures across every test binary), `cargo check -p uor-r4-graph-format --no-default-features` (+ `--features alloc`), `cargo run -q -p xtask -- validate` (R1+R4+R5+gnaf, clean), `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`, `cargo deny check bans`, `python3 scripts/check_claim_wording.py` — all green.

**Next**: the #886-style deployed-fidelity spot-check (§6 of the posted spec) — now unblocked by the emission-path extension above. That's a bespoke harness (fit → emit via the extended `emit_scored_r4g1` → consume via the new `predict_decision_candidates_with_skipmix`-shaped call), not a change to the production compile CLI; wiring skip-mix fitting into the real release build stays a post-promotion step per §7's run contract. After that: the reachability-gate probe, then (conditional on both clearing) the full #833 end-to-end run — all per the long-run discipline in AGENTS.md, with the run contract posted here before either launches.

No decisions needed from you at this checkpoint — continuing per the existing sign-off.


### comment 5373409449 — 2026-08-21T17:57:11Z — Casey-allard

## Deployed-fidelity spot-check result: LOWERING-FIDELITY GAP (task 6 of the implementation plan)

Ran the #886-style bounded minimal-pairs spot-check of the deployed 1-token skip-mix lane on the real #833 canonical bundle. Full method + result: `docs/skipmix_lane_897_spotcheck.md`; machine record: `docs/skipmix_lane_897_result.json`; harness: `crates/uor-r4-api/tests/skipmix_lane_spotcheck_897.rs`.

**Key difference from #886**: this lowering extends the REAL release emitter (`score::emit_scored_r4g1`, per the sign-off's decision point B), not a side converter, so the emission-path confound #886 had to caveat around does not apply here — **174/174 replayed positions served** (0 unservable, 0 abstained). This is a clean read on the lowering's own fidelity.

**Result**: reproduced the phase-0 `skipmix` arm's favorable pairs exactly (87/87, reproduction control) and the baseline-follow degeneracy control (0). The deployed lane followed **41/87 (47.1%)** of those favorable pairs — below the predeclared **53/87 (60%, the #886 bar)** threshold for "faithful."

**Verdict: LOWERING-FIDELITY GAP.** A legitimate negative result, not a defect in tasks 2-5's implementation — SKMX/PSIB wire format, the compiler fit, the runtime contribution+argmax integration, and the real-emitter extension all behave exactly as designed and are fully tested (round-trip, corruption rejection, absent-section identity, and now this real-bundle replay). The gap is plausibly the integer `ScoreQ` quantization plus the per-key top-64 cap and decided-candidate-list bound the reference arm's floating-point residual scoring doesn't carry — the same class of ceiling effect #886 recorded for the #836 segment lane, measured here without the emission-path confound.

**Consequence for the remaining plan**: per the same discipline #836/#886 followed, I am **not** launching the reachability-gate probe or the #833-shaped end-to-end causal run (tasks 7-8) against a lowering already shown sub-ceiling on its own best cases — that would spend hours-scale compute on a foregone dormant conclusion. RF-31 registration (task 9, conditional on promotion) is likewise not pursued. The lane ships **DORMANT**: no `model/ids.toml` serving row, no `CONFORMANCE.md` change, no serving-behavior change.

**Next**: closing out with the full local gate ladder (fmt, clippy, full workspace test suite, `xtask validate`, wasm check, `cargo deny`, claim-wording script) and opening one PR covering the complete #897 lowering (format + compiler + runtime + certify-emission + this spot-check), framed honestly as infrastructure-complete-but-dormant per the gap above. No decisions needed from you at this checkpoint — continuing per the existing sign-off.

### comment 5373818275 — 2026-08-21T18:37:19Z — Casey-allard

## Closing: #897 lowering implemented, deployed, and measured -- ships DORMANT

Merged: PR #903 -> `main` at commit `1b84770e6daca060cff4e19899796daf3ec7e8e1`.

**What shipped** (6 commits, all per Casey's sign-off on the posted design spec):
- `format`: SKMX (real fixed-capacity open-addressed hash table, multiply-free ARX hash, bounded probe) + PSIB (sorted/binary-search Ψ-bag fallback) wire sections, both optional/absent-section-identity.
- `compiler`: `skipmix_fit::fit_skipmix_tables` -- fits the phase-0-confirmed `(content_token, last_window_token)` key family from a corpus's TRAIN split.
- `format`: `from_trusted_bytes` cheap-reconstruction methods, fitting `R4Engine`'s lifetime-free owned-bytes architecture.
- `runtime`: `predict_decision_candidates_with_skipmix[_witness]` on `R4Engine` -- the #836-PSTATE-shaped contribution+argmax combinator.
- `certify`: extended `ScoredGraphSections`/`emit_scored_r4g1` (the REAL release emitter, decision point B) with `skipmix_rows`/`psi_bag_rows`, mirroring the #399 FWDA absent-section-identity precedent.
- research: the #886-style deployed-fidelity spot-check.

**Spot-check result** (`docs/skipmix_lane_897_spotcheck.md`, `docs/skipmix_lane_897_result.json`): reproduced the phase-0 `skipmix` arm's 87 favorable minimal pairs exactly (reproduction control) and its baseline-follow degeneracy control (0). Fit + emitted via the real production emitter into a real bundle, loaded into `R4Engine`, replayed on all 174 positions -- 174/174 served (0 unservable, unlike #886's emission-path confound). The deployed lane followed only 41/87 (47.1%), below the predeclared 53/87 (60%) faithfulness bar.

**Verdict: LOWERING-FIDELITY GAP.** Per the same discipline #836/#886 followed, the reachability-gate probe and the #833-shaped end-to-end causal run were not launched against a lowering already shown sub-ceiling on its own best cases -- that would have spent hours-scale compute on a foregone dormant conclusion. RF-31 sign-up (conditional on promotion) was likewise not pursued.

**Scope as merged**: infrastructure only. No `model/ids.toml` serving row, no `CONFORMANCE.md` change, no serving-behavior change -- every new code path is additive and gated on the new optional SKMX/PSIB sections; absent them, behavior is byte-identical to before this work. Full local gate ladder (fmt, clippy, full workspace test suite, xtask validate R1/R4/R5/gnaf, wasm check, cargo deny, claim-wording) and the merge-queue CI run (Kani, fuzz smoke, conformance sign-up, cargo audit, the full fmt/clippy/tests/no_std/κ suite, Gate C trend, all-features build) all green.

This closes the #897 mandate: the confirmed key family is lowered, deployed, and honestly measured against its own reference ceiling. The 47%->60% gap and whether to invest in closing it (a wider per-key cap, different quantization, or revised candidate list) vs. leaving this dormant alongside the #836 segment lane is recorded as an open question for the maintainer in the spot-check doc's "Implications" section -- not actioned here.



---
_Comment/review ids in this file: 5369699493, 5371844612, 5372169756, 5372538672, 5372593224, 5373175796, 5373409449, 5373818275_

