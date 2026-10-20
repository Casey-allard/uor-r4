# #822 — TRACKER/S1: persistent prompt-conditioned predictive state

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:32:19Z
- **Updated:** 2026-08-22T00:02:45Z
- **Closed:** 2026-08-22T00:02:45Z
- **Milestone:** R4 Intelligence / S1 — Prompt-conditioned state
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/822
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent programme: #820

## Objective

Make the complete supplied prompt exert persistent, causal influence on candidate support and scoring across generated tokens without relying on future completion tokens.

## Current evidence

- #784 measured 0.0% full-depth code collisions, yet 11/15 distinct rows still selected newline under argmax.
- Sampling restored valid outputs but only modestly changed canary distinctness; diversity alone did not establish relevance.
- The present NGRAM/exact-context path is chiefly suffix-local and lacks a demonstrated session-state contract.

## Entry criteria

S0 has frozen the normative scorer, benchmark identities, path attribution, and a production-admitted baseline.

## Native dependencies

Blocked by #821.

## Promotion gate

A selected bounded state/residual arm improves predeclared document-disjoint, EXCT-disabled causal metrics with a positive confidence-bound margin on two domains; gains survive prompt swaps and paraphrases, improve relevance rather than only diversity, and meet runtime budgets.

## Kill / redesign criterion

If two independently motivated arms fail causal controls, return to the representation/compiler model; do not continue code-space or decoder tuning as though prompt conditioning were established.

## Native children

- [x] #835 — Specify persistent causal prompt/session state
- [ ] #834 — Fit prompt-conditioned arms against causal controls
- [ ] #836 — Lower and certify the selected prompt-state scorer

GitHub's native parent/sub-issue relationship is the hierarchy source of truth.

## Non-goals

- Treat bidirectional access as future-token leakage.
- Promote W(3,3), generic novelty, or sampling heuristics as substitutes for causal prompt evidence.

## Repository conformance

Primary IDs: RF-01, RF-21, RF-22, RF-27, RF-28. Reference f32/owned models are not serving evidence; deployed state must remain bounded, integer/table-native, zero-allocation, and witnessable.

Every leaf declares execution scope, preserves the applicable P-4/determinism/allocation/no_std/error/witness contracts, maps to existing RF capabilities or justifies a new one, and updates generated conformance only through its TOML/Gherkin/test sources. Empirical work posts the repository run contract before expensive execution.

## Closure rule

Close only after every native child has a completed, negative, not-triggered, or explicitly descoped verdict and this tracker records `PROMOTE`, `REVISE`, `LIMIT`, or `RETIRE` with exact evidence links.


## Comments (17)

### comment 5364117342 — 2026-08-21T01:34:39Z — Casey-allard

### #836 outcome and S1 follow-ups (implementation order)

#836 lowered the #835 segment lane end-to-end onto the deployed `R4Engine`
serving path — format (#876/#877), P-4 ring (#878/#879), engine wiring
(#880/#882/#883), compiler emission (#881/#883), and the learned
content→teacher-argmax residual table (engine consumes it in 4c-i #883; compiler
fits + emits it in 4c-ii #884). Verdict (#885, `docs/segment_lane_836_verdict.md`):
**REVISE**, lane **dormant**. The mechanism is now the faithful #834 §6.2 arm and
is safe by construction (absent-section identity; top-8-only re-rank that never
fabricates a token or turns a serve into an abstention), but the whole-prompt
effect is real yet **sub-floor**: the reference-arm ceiling is +17.5‰ (CI upper
19.0‰) against a pre-registered 20‰ causal floor, so the ceiling is below the
floor **unconditionally** and no full causal run was launched.

Because no in-mechanism change to the lane can clear 20‰ (19.0‰ is the reference
arm's own optimistic ceiling), the follow-ups are ordered cheapest-and-most-
decisive first:

1. **#886** — bounded minimal-pairs spot-check of the deployed lane on the real
   bundle (seconds, not a full run). Confirms the quantized top-8 serving path
   tracks the §6.2 reference within its ceiling. *No blocker.*
2. **#887** — governance review of the 20‰ floor for a re-ranking lane whose
   honest ceiling is ~19‰. The pivotal gate for whether this lane can ever
   promote. *Follows #886.*
3. **#888** — widen the lane's candidate support beyond top-8 (candidate-support
   expansion). Necessary-but-insufficient mechanism work; worth building only if
   #887 reframes the bar and #886 shows the lowering is faithful. *Blocked by
   #887 (and #886's evidence).*

The lane stays dormant throughout: no `model/ids.toml` serving row,
`CONFORMANCE.md` unchanged, activation gate unchanged (*re-clears the 20‰ causal
floor on the packed path*).


### comment 5364766414 — 2026-08-21T03:16:46Z — Casey-allard

### S1 follow-up #886 done — deployed lowering is **NOT faithful** on the real bundle

PR #889 (merge `c4e7e239`). The deployed segment lane, replayed on the real #833 bundle, follows **1/10** of the #834 §6.2 favorable minimal pairs — below the pre-registered **6/10** "faithful" bar → **LOWERING-FIDELITY GAP**. The fitted table (19,710 keys) reaches the engine intact, so the gap is in the quantized top-8 / 8-slot-ring re-rank. Additionally, **6/20** favorable held-out positions are **unservable** through the lane's only PSTATE emitter (`convert_with_segment_table`), which is not serving-equivalent to the released cover/score graph (`score::emit_scored_r4g1`, no PSTATE). Lane stays dormant (no `model/ids.toml` row, `CONFORMANCE.md` unchanged). Records: `docs/segment_lane_886_result.json`, `docs/segment_lane_886_spotcheck.md`.

Bearing on the remaining S1 follow-ups (maintainer's call): #888 (candidate-support widening) sits atop a lowering that does not yet track its own ceiling, and #887 (20‰ floor governance) concerns a lane whose deployed realization is weaker than the reference ceiling it was measured against.


### comment 5365023161 — 2026-08-21T04:00:23Z — Casey-allard

**S1 governance update — #887 resolved and merged (`7bb8b4ec`).**

The #836 segment-lane follow-up-B governance question is resolved: the pre-registered **20‰ `CAUSAL_FLOOR_PERMILLE` stands** for the bounded state/residual (segment re-ranking) arm class. The segment lane **fails** this tracker's promotion gate and is **retired from the promotion track** (dormant / ledgered), per `docs/segment_lane_887_governance.md`.

Rationale (both hold): the reference-arm ceiling (+17.5‰, CI upper 19.0‰) is below the floor **unconditionally** (#836), and the deployed lane does not track that ceiling (1/10 < 6/10; deployed delta −1.6‰; #886).

**Stage state — unchanged by this record.** #822 is **not** yet closable: native children are #835 ✓, **#834 open (maintainer-parked)**, #836 ✓. The kill/redesign criterion ("if **two** independently motivated arms fail causal controls, return to the representation/compiler model") is **not** triggered by this single arm's retirement. The stage `PROMOTE / REVISE / LIMIT / RETIRE` verdict remains the maintainer's, against the full child set (including #834's further Ψ-family arm evaluation).

**Downstream:** #888 (candidate-support widening) is closed off by this verdict — widening a lane whose ceiling is sub-floor and whose deployed lowering already shows a fidelity gap cannot change the dormant verdict.


### comment 5365832637 — 2026-08-21T06:02:55Z — Casey-allard

### S1 child work complete — #834 closed with `REVISE` (PR #891, merge `77d09023`)

The last open S1 child is resolved. Status of the full S1 child set:

- **A #835 (spec):** CLOSED — frozen experimentable contract + reference model (PR #872).
- **B #834 (research):** CLOSED — the five-arm space is fully dispositioned; the final arm `conditional-residuals` = **REVISE, real but unconditionally sub-floor** (+16.2‰ paired [14.6, 17.9], upper bound < the 20‰ floor; the mechanism-specific increment vs Ψ is non-positive: −1.3‰ [−2.4, −0.1], residual-shuffle null lands at the real arm). **No arm cleared the frozen causal floor.** (PRs #873/#874/#875/#891; records `docs/prompt_arms_bakeoff_834.md` §6.1–6.3.)
- **C #836 (runtime):** CLOSED — segment lane lowered end-to-end, verdict REVISE/dormant (ceiling 19.0‰ < floor).
- Follow-ups: #886 (deployed lowering-fidelity gap 1/10) DONE · #887 (20‰ bar stands; lane retired from the promotion track) DONE · #888 closed not-planned.

**This tracker's kill/redesign criterion is now met on complete evidence:** two-plus independently motivated arms fail the causal gate. Per #834's pre-registered contract and the 2026-08-21 decision comment, the `PROMOTE/REVISE/LIMIT/RETIRE` stage verdict against the full child set is the maintainer's; the evidence set for that call is complete. Further movement at S1 is representation/compiler redesign, not code-space or sampling tuning. S2 (#823) is parallel-available and is now the natural live lane for session work (#838 → #837 → #839).


### comment 5368815253 — 2026-08-21T10:42:25Z — Casey-allard

### Maintainer stage verdict (2026-08-21): **REVISE** — S1 returns to representation/compiler redesign

Recorded on the maintainer's decision this morning, against the complete child-set evidence (see the 2026-08-21 child-state summary above):

- The five-arm space of #834 is fully dispositioned and **no arm cleared the frozen 20‰ causal floor**: `current-scoring` / `longer-local-context` negative (#874); `persistent-state` positive-sub-floor off-serving (#875) → lowered end-to-end (#836) → retired with the bar standing (#886/#887); `conditional-residuals` REVISE, unconditionally sub-floor with its distinctive mechanism falsified (PR #891); `candidate-support-expansion` closed not-planned (#888).
- The pre-registered kill/redesign criterion — two-plus independently motivated arms fail causal controls — is met.

**Verdict meaning:** the S1 prompt-conditioning claim is NOT established at the current representation; the stage neither passes nor is retired. The content-evidence signal is real (+16–17‰ in two independent arms) but structurally sub-floor, so S1's remaining work is **representation/compiler redesign** — not further arm fitting, code-space, or sampling tuning at the current representation.

**Process gate for the redesign:** a redesign RFC (design document, no code) comes first — candidate mechanisms, how each addresses the measured falsifiers (generation and scoring suffix-locality #874/#891/#894; singleton-margin pollution and the 2,454 content-answerable novel positions invisible to suffix features #893), pre-registered re-entry gates (**the 20‰ causal floor and the #838 selective-prediction gates DO NOT MOVE**), and cost estimates — submitted for maintainer approval before any build. This tracker stays OPEN as the home for that work; #840's corrective rounds (see its thread) wait for the approved direction.


### comment 5368900534 — 2026-08-21T10:51:15Z — Casey-allard

### Maintainer approval (2026-08-21): the S1 redesign RFC is APPROVED — execution unblocked

The maintainer approved `docs/s1_redesign_rfc_822.md` (PR #896) as written this morning. Blanket approval adopts the RFC's stated recommendations on its four questions (any of these can be overridden by a later maintainer comment):

- **Q1 — YES, runs first:** verify from the obs-shard producer what prefix length the recorded `t_argmax` labels consumed (reading only; reframes ceilings, blocks nothing).
- **Q2 — direct read-only:** the D2 instrument may read compiled region assignments directly from the released artifact; no API surface is required first.
- **Q3 — ADOPTED:** ≥ 25‰ off-serving paired lower bound is the pre-declared bar for OPENING a lowering track; the 20‰ causal floor itself remains the unchanged promotion gate.
- **Q4 — DEFERRED:** the D3 re-observation budget decision waits until Q1 + D1/D2 evidence exists (RFC §6 step 4).

**Execution order for working sessions (S1 lane, this tracker):**

1. **Q1** verification (record the answer on this tracker).
2. **D1 instrument** — joint conditional (content-token × suffix) residual reference arm, D4 as an internal comparison arm, backed-off-mix fallback pre-declared; run contract posted before the run; SELECT bar = 25‰ lower bound (floor 20‰ unchanged).
3. **D2 instrument** — region-conditional evidence, granularity sweep against the #784 convergence null; same contract discipline.
4. Decision point per RFC §6 (lowering track / D3 escalation / honest bound recording).

Parallel approved tracks (their own mandates already recorded): #839 phase-1 legacy-mode typed-surface execution (issue comment 2026-08-21) and F0 #859 milestone-0 statements-only scaffold (issue comment 2026-08-21, "References #859" PR for review).


### comment 5369213028 — 2026-08-21T11:24:51Z — Casey-allard

### RFC Q1 answered (S1 redesign, `docs/s1_redesign_rfc_822.md` §4-D3): what prefix did the recorded `t_argmax` labels consume?

**Answer.** Each recorded label is the teacher's argmax after consuming the **full article-token prefix up to that position — full causal attention over positions `0..=pos` — hard-capped at `--sequence-length 128`** (the observe-text default; SmolLM2-360M-Instruct's native `max_position_embeddings` 8192 is not the binding cap). The deployed 8-token `compiler::WINDOW` never bounds the teacher during observation; it keys only the emitted sample (`sample_id`/`shard_of` routing).

**Producer-code evidence** (read at main `6fc46149`):
- `crates/uor-r4-graph-compiler/src/observation_text.rs::produce_article_records`: one `oracle.reset()` per article, then `for pos in 0..positions { oracle.step(tokens[pos], pos, &mut logits) }` with `positions = min(tokens.len()-1, seq_len)`; the v4 record's top-8 (whose `[0]` becomes `Corpus::t_argmax` per `compiler::load_corpus_bytes`) is encoded from those logits; the 8-token `window` feeds only `observe::sample_id`/`shard_of`. Teacher-forced: the sampled token is discarded, `next` is the actual next text token.
- `crates/uor-r4-model-source/src/lib.rs::Llama::layer_forward`: attention spans `st.att[h*seq_len .. h*seq_len + pos + 1]`, KV caches sized `n_layers x seq_len x kv_dim` — the whole cached prefix, every step.
- seq_len plumbing: `Teacher::load_with_sequence_length(source, options.sequence_length)` with the graph-CLI observe default `sequence_length: 128` (`.min(max_position_embeddings)`); the #833 run-contract wording concurs ("dense Simple-Wiki 0..2999 seq-len-128 re-observe").
- One doc nit found while reading: the `observation_text.rs` module header says the text stream is "tokenized BOS-prefixed", but the text path steps raw `encode_lossy` tokens with no BOS prepend (`TokenizerKind::encode_lossy` -> `HfBpeTokenizer::encode` inserts no specials); the BOS wording matches the autoregressive `observe` path, which starts from `oracle.bos_token()`. Immaterial to this answer (one token of prefix either way); a one-line doc correction will ride the next S1 PR.

**Corpus-byte confirmation** (attested #833 bundle, `corpus.meta` CID `blake3:aa9d1767...`, 360,924 x 88-byte records):
- `span_start` spans 0..=127; exactly 2,994 records at `span_start = 0` (one per story);
- per-story position counts max out at exactly 128; **2,556/2,994 stories (85.4%) hit the cap** (truncated articles);
- mean label prefix length (`span_start + 1`) = **62.7 tokens**.

**Consequences for the RFC branches (§6):**
1. The labels are **not** window-conditioned: the teacher consumed up to 16x the deployed 8-token key (mean ~63 tokens). The measured suffix-locality (#874/#875/#891/#894) is a property of the **compiled key space**, not an observation-protocol artifact at the window scale. **D1/D2 remain fully meaningful** — label-borne conditioning between 8 and 128 tokens exists for joint/region keys to reach.
2. The label-side ceiling is **128 tokens of document prefix**, not the teacher's native 8,192: conditioning beyond 128 tokens is absent from every recorded label, and 85.4% of articles were truncated at the cap. D1/D2 run contracts must not promise conditioning beyond that bound.
3. If D1 and D2 both report sub-floor, the honest §6-step-4 recording is: the S1 claim as measured is additionally bounded by <=128-token label conditioning, and the D3 re-observation decision (raise `--sequence-length`; possibly alongside wider observation windows) goes to the maintainer with its own contract as the remaining lever.

Reading-only: no code changed. The numbers above are reproducible from the producer sources at main and the attested bundle bytes. This answer will be restated in the D1 record (`docs/*_822.md`).


### comment 5369264553 — 2026-08-21T11:29:28Z — Casey-allard

### RFC D1 run contract (S1 redesign §4-D1 / §6-2): joint conditional keys — posted BEFORE the run

Instrument: `crates/uor-r4-api/tests/joint_conditional_run_822.rs` (new; #875/#891 harness mold), offline reference arm on the attested #833 bundle (`corpus.meta` CID `blake3:aa9d1767...`), teacher-grounded via recorded `t_argmax` (no live teacher). Per RFC Q1 (comment above): labels condition on <=128-token article prefixes; this instrument fits evidence within that bound by construction.

**Pre-registered arms** (fixed lambda = 1.0 before evaluation; exploratory sweep is not the verdict):
- `base` — suffix rate over suffix candidates (the suffix-local floor; #891 construction verbatim).
- `psi`, `cr` — reproduced exactly as harness-correctness gates (#875/#891 constructions + their original candidate sets).
- `joint` (PRIMARY) — score(c) = suffix_rate(c) + lambda * mean over the window's unique tokens t with a supported joint key (t, 2-token-suffix) of [P_train(c | t, suffix) - suffix_rate(c)]. Bounded tables: cap 64 per key, as #835.
- `mix` (pre-declared fallback, RFC-named) — joint residual where the joint key is supported, the exact #875 psi-bag contribution where not, normalized over all unique window tokens.
- `d4pos`, `d4skip` (internal comparison arms, RFC §4-D4) — same residual shape over (token, distance-from-end) keys and (token, last-token) skip-bigram keys.
- Candidate sets: repro arms use the exact #891 sets; new arms use suffix ∪ content-top-32 ∪ joint-top-32 (shared across all new arms, so contrasts stay scoring-rule ablations); `joint-narrow` on the legacy set separates scoring from candidate-availability.
- Nulls (planted, must be able to fail): `swap` (different-story content/joint evidence under this window's suffix) and `keyshuf` (this window's tokens looked up under the swap partner's suffix key — the conditioning-specificity null unique to D1). Both must change >=1 prediction; both must sit below a positive arm.
- Coverage reporting (pre-registered sparsity risk): joint-table entries/mass, positions with >=1 supported token, mean supported tokens per position, known-suffix vs novel-suffix split.

```text
metric to move:      teacher-grounded held-out top-1 vs the suffix-local floor; base 246.6permille
                     (recorded #875/#891); best prior content arms: psi +17.5 [15.9,19.0], CR +16.2 [14.6,17.9]
reachability ceiling: base-miss mass = 1000-246.6 = 753.4permille overall; joint keys can fire only on
                     known-suffix positions (46,128/72,130 = 639permille of held-out; #893's 26,002 novel-suffix
                     positions fall back joint->base, mix->psi-bag). Ceiling >> the 25permille bar; run decides.
instrument + verdict: in-run reproduction gates must PASS before the verdict binds — base 246.6+-0.05,
                     psi 264.1+-0.05, cr 262.8+-0.05, minimal-pairs total 4,722 exact, psi-follow 10,
                     cr-follow 13, base-follow 0 (docs/conditional_residuals_834_result.json); joint-table
                     anti-vacuity (nonempty, >0 covered positions); both nulls change >=1 prediction.
exit rule:           PRIMARY = joint-vs-base paired 95% lower bound. >=25.0permille -> SELECT (Q3-adopted
                     lowering-track opening bar; the 20permille floor unchanged as the promotion gate).
                     Fallback consultation order (pre-declared): mix consulted only if joint is below bar.
                     Lower bound <=0 AND 0 joint minimal-pair follows -> NO ARM. Otherwise REVISE —
                     the [20,25) floor-clearing-but-below-opening-bar case is recorded explicitly as such.
if positive:         file an #836-shaped lowering-track issue for the selected arm (built-capability order
                     judged at implementation; #886-style deployed-fidelity spot-check pre-planned). D2 still
                     runs afterward — independent evidence for the same §6-4 decision.
if negative:         record on #822 + docs/s1_redesign_instruments_822.md; proceed to the D2 instrument
                     (RFC §6-3). §6-4 then consumes Q1 + D1 + D2 together. The branches differ.
cost estimate:       offline, zero teacher compute; ~2 min table build + multi-arm eval over 72,130
                     positions, est. 10-30 min wall, ~2 GB peak, Mac-local. Blocks nothing.
```

Deliverables on completion: `docs/s1_redesign_instruments_822.md` (contract + outcome, append-only), `docs/joint_conditional_822_result.json` (CID-bound, written by the run), the harness test, and the one-line `observation_text.rs` BOS doc correction flagged in the Q1 comment. PR will reference #822 (the tracker stays open for the stage verdict).


### comment 5369435954 — 2026-08-21T11:48:13Z — Casey-allard

### D1 outcome vs contract: **SELECT (backed-off mix)** — first floor-clearing off-serving arm

The D1 instrument ran to completion against the posted contract (35.9 s). **All reproduction gates PASS exactly**: base 246.6 permille, Ψ 264.1 (+17.5 [15.9, 19.0]), CR 262.8 (+16.2 [14.6, 17.9]), minimal pairs 4,722, follows Ψ=10 / CR=13 / base=0; double-run 2,000 positions identical.

**Verdict under the pre-registered rule (joint → mix consultation order):**

| arm | top-1 | paired Δ vs base [95% CI] |
|---|---|---|
| base (suffix floor) | 246.6‰ | — |
| joint (strict, PRIMARY) | 253.4‰ | +6.8 [+4.8, +8.9] |
| **mix (pre-declared fallback)** | **277.2‰** | **+30.6 [+28.6, +32.5]** |
| d4skip (comparison) | 296.2‰ | +49.6 [+46.8, +52.4] |
| d4pos (comparison) | 217.9‰ | −28.7 [−31.8, −25.7] |
| swap null | 213.4‰ | −33.2 (changed 33,360) |
| keyshuf null | 130.4‰ | −116.3 (changed 39,734) |

- The mix lower bound **28.6‰** clears the Q3 **25‰ opening bar** and sits entirely above the frozen **20‰ floor** — the first arm in the S1 record to do either, off-serving. The strict joint arm is real but sparsity-bound: its support exists on exactly the 46,128 known-suffix positions (= #893's decomposition), mean 2.85/7.41 window tokens supported.
- **Conditioning specificity is certified by both planted nulls collapsing**: foreign content under the true suffix (swap, −33.2‰) and true content under a foreign suffix (keyshuf, −116.3‰). The (token, suffix) PAIRING carries the signal — the #891 falsifier is answered: the missing subtrahend was the suffix-conditional, and representing it turns the conditioning increment positive.
- Minimal-pair follows: joint 83 / mix 78 of 4,722 (Ψ was 10; base structurally 0) — an order of magnitude beyond the bag arms.
- λ-sweep exploratory, flat at the pre-registered λ=1 (250/253/248/239/230 for 0.5/1/2/4/8).
- Comparison-arm note (recorded, not a verdict): `d4skip` — (token, last-token) keys, denser 1-token conditioning — measured **+49.6 [46.8, 52.4]**, stronger than both joint and mix on the shared candidate set. Key granularity is design input for the lowering track. `d4pos` is a recorded NEGATIVE.

**Contract disposition (positive branch taken):** the #836-shaped lowering track is filed as **#897** (sub-issue of this tracker; trigger-gated on the RFC §6-4 decision point; the 20‰ floor stays the promotion gate end-to-end). Proceeding to **D2** (region-conditional instrument) per §6-3; §6-4 then consumes Q1 + D1 + D2 together.

Records: `docs/s1_redesign_instruments_822.md` §4 and CID-bound `docs/joint_conditional_822_result.json` (result_cid `blake3:41be22f04af4662277e65029e558442dfdc02dd93494db369765a8606ede0900`) — landing via PR #898.


### comment 5369472904 — 2026-08-21T11:52:38Z — Casey-allard

### RFC D2 run contract (S1 redesign §4-D2 / §6-3): region-conditional evidence — posted BEFORE the run

Instrument: `crates/uor-r4-api/tests/region_conditional_run_822.rs` (new; #875/#891/D1 harness mold), offline, on the attested #833 bundle. Region assignment is read DIRECTLY from the released artifact per the Q2 approval: codes derived for every corpus record by the deployed quantization path (`runtime::bundle_plain` -> `runtime::assign_code_for_bundle` — the exact #784 instrument path), cached via the κ-gated code sidecar. No recompile.

**Pre-registered arms** (λ = 1.0 fixed before evaluation; the sweep is exploratory):
- `base` — suffix floor. Reproduction gates: 246.6 permille +-0.05; minimal-pair mining 4,722 exact; base-follow 0.
- `region[d]`, d in {1,2,3,4} — score(c) = suffix_rate(c) + λ·(region_rate_d(c) − suffix_rate(c)); region key = the record's graded code truncated to depth d (cap-64 tables). **PRIMARY = region[4]** (full depth — the H(x) bet); the depth sweep is the pre-registered granularity sweep.
- `rxs` — region × suffix product key at full depth (code4, 2-token suffix): the geometric analog of D1's joint key.
- Null (planted): `codeshuf` — this position's suffix baseline with region evidence looked up under the swap partner's full-depth code (different-story rotation, #891 partner derivation). Must change >=1 prediction; must sit below a positive arm.
- Convergence diagnostic reported (the pre-registered #784 null at corpus scale): held-out-usage-weighted share of region[4] tables whose argmax equals the corpus-marginal token, plus distinct-code and table-coverage stats per depth.

```text
metric to move:      teacher-grounded held-out top-1 vs the suffix floor (246.6 permille); prior for THIS
                     mechanism family: #784 measured continuation-distribution convergence (11/15 distinct
                     full-depth codes argmax to the corpus mode) — the null this instrument must beat.
reachability ceiling: 753.4 permille base-miss mass; region keys are total (every window has a code), so
                     reach is bounded by TRAIN table coverage per depth (reported), not by key sparsity.
instrument + verdict: reproduction gates must PASS before the verdict binds — base 246.6+-0.05, minimal
                     pairs 4,722 exact, base-follow 0; region tables non-empty at every depth; >1
                     distinct full-depth code on held-out; codeshuf changes >=1 prediction.
exit rule:           consultation order region[4] -> rxs (pre-declared; the depth sweep is exploratory):
                     SELECT iff the consulted arm's paired 95% lower bound >= 25.0 permille (Q3 opening
                     bar; the 20 permille floor unchanged as the promotion gate). NO ARM iff region[4]
                     and rxs both have lower bound <= 0 with 0 minimal-pair follows. Otherwise REVISE
                     (a [20,25) lower bound recorded explicitly as floor-clearing-below-bar).
if positive:         the geometric key joins #897's design space (region-keyed evidence lowering);
                     §6-4 consumes Q1+D1+D2 and the maintainer picks the lowering key family.
if negative:         the #784 convergence result stands at corpus scale and is recorded as the honest
                     bound on the geometric-key direction; §6-4 proceeds on Q1+D1 evidence (D1's
                     SELECT already stands). The branches differ.
cost estimate:       offline, zero teacher compute; one code pass over 360,924 records via the deployed
                     quantization path (~2-10 min, sidecar-cached) + table build + eval; est. <=20 min
                     wall, ~1-2 GB peak. Mac-local. Blocks nothing.
```

Deliverables on completion: §5 appended to `docs/s1_redesign_instruments_822.md`, `docs/region_conditional_822_result.json` (CID-bound, written by the run), the harness. PR will reference #822 once #898 lands (the record doc lives there).


### comment 5369649809 — 2026-08-21T12:12:31Z — Casey-allard

### D2 outcome vs contract: **REVISE** — the geometric key is real but sub-floor at the pre-registered consultation

The D2 instrument ran to completion against the posted contract (5.0 s; the per-record code pass was sidecar-cached; codes derived via the deployed `bundle_plain` → `assign_code_for_bundle` path per the Q2 approval). **Reproduction gates PASS** (base 246.6 permille exact; minimal pairs 4,722 exact; base-follow 0); double-run 2,000 identical; 37,587 distinct full-depth codes on held-out.

| arm | top-1 | paired Δ vs base [95% CI] |
|---|---|---|
| region[1] | 247.5‰ | +0.8 [−1.8, +3.4] |
| region[2] (sweep, exploratory) | 270.0‰ | +23.3 [+21.0, +25.7] |
| region[3] | 267.6‰ | +21.0 [+18.6, +23.4] |
| **region[4] (PRIMARY)** | 266.1‰ | **+19.5 [+17.0, +21.9]** |
| rxs (code4 × suffix) | 257.0‰ | +10.4 [+8.5, +12.3] (covered 34,656/72,130) |
| codeshuf null | 84.2‰ | −162.5 (changed 56,988) |

- **Verdict under the pre-registered consultation (region[4] → rxs): `REVISE`** — region[4]'s paired lower bound 17.0‰ is real but below the frozen 20‰ floor; rxs is sub-floor with key sparsity. No lowering track opens from D2.
- **The #784 convergence null is materially refined, not merely confirmed**: only 5,392/52,398 (10.3%) of covered held-out positions sit in region tables whose argmax IS the corpus mode. At fine granularity the artifact's own regions are differentiated and carry +19.5‰ of real conditional signal that the deployed scorer currently discards. The signal is code-specific: the codeshuf null collapses by −162.5‰ (changed 56,988 predictions).
- **Granularity trade-off recorded (exploratory, NOT a verdict):** the depth sweep peaks at depth 2 — +23.3‰ [21.0, 25.7], a floor-clearing lower bound that is still below the 25‰ opening bar — while full depth fragments support (d4 table coverage 52,398/72,130; d2 covers 71,020). The λ-sweep reads 279‰ at λ=0.5 on region[4] (softer mixing helps). Both are recorded as design input for #897's key-family decision only.
- Minimal pairs: region4-follow 53 / rxs-follow 56 of 4,722 (base structurally 0) — same-suffix disambiguation from the geometric key alone.

**Contract disposition (negative branch, as pre-declared):** no track opens from D2; the region-key direction is recorded as REAL-but-sub-floor with the granularity/λ observations attached to #897 as design input. **§6-4 now consumes Q1 + D1 + D2 — the decision request follows as a separate comment.**

Record: `docs/s1_redesign_instruments_822.md` §5; CID-bound `docs/region_conditional_822_result.json` (result_cid `blake3:4c70f8f1102fb7d5675169fc4ec4088ef28b2c7425add5d168125597809af502`). PR follows once #898 lands (the record doc lives there).


### comment 5369661098 — 2026-08-21T12:13:38Z — Casey-allard

### §6-4 decision point (RFC step 4): S1 re-entry — evidence complete, maintainer decision requested

Q1 + D1 + D2 are recorded on this tracker and in `docs/s1_redesign_instruments_822.md` (§2/§4/§5; CID-bound JSONs in docs/). Standing evidence:

| item | outcome |
|---|---|
| Q1 | recorded labels condition on full article prefixes hard-capped at 128 tokens (mean 62.7); suffix-locality is a compiled-key property |
| D1 joint (strict, 2-token conditioning) | +6.8‰ [4.8, 8.9] — real, sparsity-bound |
| **D1 mix (SELECTED)** | **+30.6‰ [28.6, 32.5]** — first arm to clear the 25‰ opening bar and the 20‰ floor off-serving |
| D1 d4skip (comparison, 1-token conditioning) | +49.6‰ [46.8, 52.4] — denser support, stronger than the selected arm |
| D2 region[4] (PRIMARY) | +19.5‰ [17.0, 21.9] — REVISE, sub-floor |
| D2 region[2] (exploratory sweep) | +23.3‰ [21.0, 25.7] — floor-clearing, below the bar |
| planted nulls | all collapse (swap −33.2; keyshuf −116.3; codeshuf −162.5) — the specific key pairing carries the signal |

Maintainer decisions requested (the RFC's SELECT branch makes 1 the default; 2-3 are the open design/budget questions):

1. **Activate #897** (the D1-triggered #836-shaped lowering track, currently trigger-gated on this decision) — per the approved RFC, a SELECT re-enters S1 through the lowering with the #886-style deployed-fidelity spot-check pre-planned and the frozen 20‰ end-to-end promotion gate unchanged.
2. **The lowered key family** (named on #897 as its first design decision): (a) the selected backed-off mix exactly as measured (2-token joint + Ψ fallback); (b) a 1-token-conditioning mix variant, first confirmed by a cheap pre-registered run with it as PRIMARY (the d4skip comparison evidence suggests it is both stronger and simpler, but it sat outside the SELECT consultation order, so a confirmation run keeps the selection pre-registered — minutes of compute); (c) additionally admit region-key features (D2's depth-2 observation) into the same design space.
3. **D3 (re-observation beyond the 128-token label cap):** keep deferred per Q4, or budget now.

Once recorded, the re-entry direction governs #897's activation and the S2 calibrator re-entry sequencing (#823 / RFC §5). PR #899 (the D2 record) is in the merge queue.


### comment 5369699295 — 2026-08-21T12:17:35Z — Casey-allard

### Maintainer decision (2026-08-21): §6-4 re-entry recorded — #897 ACTIVATED; key family = 1-token-conditioning mix behind a pre-registered confirmation; D3 stays deferred

Recorded against the complete Q1 + D1 + D2 evidence set (decision-request comment above):

1. **#897 is ACTIVATED** — the trigger is satisfied by the D1 `SELECT (backed-off mix)` outcome. S1 re-enters through the #836-shaped lowering with the #886-style deployed-fidelity spot-check pre-planned. The frozen 20‰ end-to-end floor remains the promotion gate; the 25‰ bar remains an opening bar only.
2. **Key family:** #897 phase 0 is a cheap pre-registered confirmation run with a **1-token-conditioning backed-off mix as PRIMARY** (motivated by the d4skip comparison evidence, +49.6‰ [46.8, 52.4]; it sat outside D1's consultation order, so a fresh pre-registered run keeps the selection disciplined). SELECT bar 25‰ unchanged, plus a head-to-head requirement against the D1-selected arm. If it clears, #897 lowers the 1-token key family; if not, #897 lowers the D1-selected 2-token mix as measured. The phase-0 spec is posted on #897.
3. **D3 (re-observation past the 128-token label cap): DEFERRED** — revisited only if the lowered arm's end-to-end measurement hits the label ceiling (Q4 disposition confirmed).
4. Sequencing: S2's #839 phase-1 (typed-abstention production surfaces) proceeds next per its recorded re-scope; the S2 calibrator re-entry continues to ride the S1 lowering direction per RFC §5.


### comment 5370066116 — 2026-08-21T12:56:42Z — Casey-allard

D2 record merged: PR #899 → main `201a747a` (docs/s1_redesign_instruments_822.md §5 + CID-bound docs/region_conditional_822_result.json now on main; the D1 record landed via PR #898 → `9d2dbe19`). The full Q1+D1+D2 instrument series and the §6-4 re-entry decision are recorded on this tracker; the S1 lane now proceeds through #897 (phase-0 confirmation pre-registered there). Unassigning — no S1 work is actively running this hour; #897 is the activated home for the next S1 action.

### comment 5371847876 — 2026-08-21T15:24:00Z — Casey-allard

### S1 lane update — #897 phase 0 COMPLETE: `SELECT-1-token` (PR #902 → main `9bc2b592`)

The pre-registered 1-token key-family confirmation (§6-4 decision, item 2) ran and merged. `skipmix` (the 1-token-conditioning backed-off mix) measured **+56.2‰ [53.6, 58.7] vs base** (clears the 25‰ opening bar) and **+25.6‰ [23.1, 28.1] vs the D1-selected 2-token mix** (> 0) — both pre-registered conditions met; the conditioning-specificity null collapses (−81.3‰, 47,185 changed). All D1 reproduction gates reproduce exactly (base 246.6‰, mix 277.2‰, d4skip 296.2‰, minimal pairs 4,722, base-follow 0).

**Verdict `SELECT-1-token`** — #897 lowers the 1-token skip-mix key family. Record: `docs/s1_redesign_instruments_822.md` §6; CID-bound `docs/skipmix_confirm_897_result.json` (result_cid `blake3:4a13095b…`). #897 stays OPEN for the `#836`-shaped lowering (parked at the maintainer design call on table layout/mixing weights); the frozen 20‰ end-to-end floor remains the promotion gate; D3 stays deferred. The S2 #839 calibrator re-entry continues to ride this lowering direction (RFC §5).


### comment 5376376454 — 2026-08-21T23:17:49Z — Casey-allard

## S1 stage verdict — PROMOTE (2026-08-21)

All S1 native children are resolved and the pre-registered promotion gate — the frozen **20‰ end-to-end causal floor** — is **cleared**, so **S1 is promoted**: prompt-conditioned predictive state is established at the redesigned representation (the 1-token skip-mix lowering), and S3 (#824) is unlocked. This is the maintainer decision; it is the readable mirror of the tracker verdict.

### The lowering track that cleared the gate

The S1 REVISE redesign (§6-4, D1 `SELECT (backed-off mix)`) re-entered through the #897 lowering and its follow-ups:

- **#897 (PR #903)** lowered the confirmed 1-token skip-mix scorer (real open-addressed SKMX hash + sorted PSIB fallback) onto the deployed `R4Engine`; shipped **dormant** after a spot-check fidelity gap (41/87).
- **#904 (PR #905)** diagnosed the gap as **breadth-bound** (the base candidate list structurally excludes the targets; width levers don't help).
- **#906 (PR #907)** fixed it with **candidate injection** — the lane discovers candidates from its own SKMX/PSIB rows — lifting deployed fidelity to **58/87 (66.7%)**, clearing the 60% bar.
- **#908 (PR #909)** measured the deployed **end-to-end causal** effect (teacher-free, all 72,130 held-out positions through the deployed lane vs the deployed base): **+28.45‰ [25.57, 31.32]** — the paired 95% lower bound clears the 20‰ floor — with the conditioning-specificity (label-shuffle) null collapsing to **−236.95‰**. Reachability probe passed first (ceiling 549‰; subsample +35.67‰). Record `docs/skipmix_endtoend_causal_908.md`, result_cid `blake3:e32e4e33…`.

As predicted, the deployed +28.45‰ is ~half the off-serving +56.2‰ (toy base) — the strong deployed base absorbs the rest — but the lowered lane still clears the floor against the real deployed base.

### Frozen gates held

The 20‰ causal floor and the #838 selective-prediction gates did not move (the #887 discipline). D3 stays deferred (the end-to-end measurement did not hit the 128-token label ceiling).

### Follow-through

Activation of the promoted lane (turn it on: serving reroute + compile-path wiring + **RF-31** registration) is tracked on **#910** (this tracker's open child). #822 stays open until #910 lands, then closes. The completion-plan S1 section (`docs/r4_intelligence_completion_plan.md`) is reconciled REVISE → PROMOTE alongside #910.


### comment 5376628647 — 2026-08-22T00:02:44Z — Casey-allard

## S1 CLOSED — PROMOTED and activated (2026-08-22)

All six native children are resolved and the stage promotion decision is recorded, so the S1 tracker closes. Persistent prompt-conditioned predictive state is **established at the redesigned representation** (the 1-token skip-mix lowering), and **S3 (#824) is unlocked**.

**Children (all closed):** #835 (spec) · #834 (arm bake-off, REVISE) · #836 (segment-lane lowering, REVISE/dormant) · #897 (D1-selected skip-mix lowering) · #908 (end-to-end deployed causal run) · #910 (activation / RF-31).

**Promotion gate cleared.** The frozen 20‰ end-to-end causal floor was cleared by #908 (PR #909): deployed **+28.45‰ [25.57, 31.32]**, paired 95% lower bound above 20‰, with the conditioning-specificity null collapsing to −236.95‰ (`docs/skipmix_endtoend_causal_908.md`, result_cid `blake3:e32e4e33…`). Verdict recorded above (2026-08-21).

**Activated.** #910 (PR #911 → main `eee8f235`) turned the lane on: `R4Engine::predict_decision` routes through the skip-mix lane (byte-identical on sections-absent bundles; re-selects only the served token; allocation-free; P-4-clean), the release compile path fits and emits the SKMX/PSIB sections, and the capability is registered as **RF-31** (CONFORMANCE.md regenerated, 31 ids).

**Frozen gates held** (the #887 discipline): the 20‰ causal floor and the #838 selective-prediction gates did not move; D3 stays deferred (the end-to-end measurement did not hit the 128-token label ceiling). The plan doc S1 section is reconciled REVISE → PROMOTE (`docs/r4_intelligence_completion_plan.md`, append-only).


---
_Comment/review ids in this file: 5364117342, 5364766414, 5365023161, 5365832637, 5368815253, 5368900534, 5369213028, 5369264553, 5369435954, 5369472904, 5369649809, 5369661098, 5369699295, 5370066116, 5371847876, 5376376454, 5376628647_

