# RFC: S1 representation/compiler redesign (#822 REVISE follow-through)

- **Status:** DRAFT for maintainer review. Design only — **no code lands from this
  document**, and no direction below is started until the maintainer approves a
  direction and its first instrument.
- **Date:** 2026-08-21.
- **Mandate:** the S1 stage verdict of 2026-08-21 (**REVISE**, recorded on #822 and in
  `docs/r4_intelligence_completion_plan.md`): the prompt-conditioning claim is not
  established at the current representation, the five-arm mechanism space at that
  representation is exhausted, and S1's remaining work is representation/compiler
  redesign. Claim language follows `docs/formal_vocabulary.md`.

## 1. What does not move

- **The 20‰ causal floor** (`CAUSAL_FLOOR_PERMILLE`, #834 harnesses) for any
  prompt-conditioning promotion, evaluated as a paired lower confidence bound on the
  #833 protocol (or its successor bundle compiled under the same discipline).
- **The #886/#887 lowering calibration:** an off-serving positive must carry a
  comfortable margin above the floor — the packed path re-faces integer-`ScoreQ`
  quantization and the top-8 candidate bound that sank a 19.0‰ ceiling. Planning target
  for any candidate: **≥ 25‰ off-serving lower bound** before a lowering track opens.
- **The #838 selective-prediction gates** (false-answer UCB95 ≤ 10‰ release / ≤ 50‰
  research at their coverage floors) for the S2 re-entry that rides on this redesign.
- **The #841 §6 corrective bar** for any generation claim (median first-divergence
  +≥2 steps AND diverged-at-0 −≥100‰ per round, ≤3 rounds).
- **Deployed invariants:** P-4 operation classes (no multiply/divide/float in the
  deployed kernel), allocation-free steady state, bounded capacities, deterministic
  bytes, typed errors, no_std boundaries, witness replay. The offline compiler remains
  unconstrained (f32/allocation permitted) per the spec split.

## 2. The measured falsifiers a redesign must answer

**F1 — Suffix-locality of scoring and generation.** The deployed artifact's effective
predictive state is the 2-token suffix: full-window context measured −1.6‰ vs
suffix-only on the deployed path (#874); the best reference content evidence measured
+16.2…+17.5‰ with CI upper bounds ≤ 19.0‰ (< floor, #875/#891); and 99/100 free-running
rollouts are token-identical to suffix-only rollouts with median first divergence at
step 0 (#894). Whatever whole-window meaning the semantic codes carry (0.0% full-depth
collisions, #784), the *evidence tables the scorer consults* do not condition on it.

**F2 — Bag-level content evidence is sub-floor and its refinement was falsified.** The
Ψ-class arms aggregate window tokens as an unordered bag against per-token argmax
tables (cap 64). That mechanism class measured +16–17‰ in two variants, and the one
attempted refinement — subtracting the *global* corpus marginal (#891) — made it
slightly worse, with the residual-shuffle null landing on the real arm. The bag has
been squeezed; the next signal must come from **structure the bag discards**.

**F3 — Confidence signals are singleton-polluted and thin (#893).** Margin 1000‰ is
dominated by total=1 suffix keys; support×margin bucketing rank-orders risk cleanly but
the ~99%-precision slice covers only ~1% of positions. 2,454 of 26,002 novel-suffix
positions are content-answerable yet invisible to every suffix feature — quantified
headroom that only a representation change can reach.

**F4 — No trajectory awareness (#894).** 710‰ of 32-step rollouts collapse into ≤4-token
cycles; no step of any trajectory abstained. Nothing in the artifact distinguishes "I am
repeating myself" from fresh prediction. (S3's lane; recorded here because candidate
representations differ in whether they can carry trajectory state at all.)

## 3. Ceiling analysis (why these falsifiers imply *representation*, not tuning)

Three independent studies point at the same object: the **evidence key**. Scoring keys
(suffix 2-grams), confidence keys (suffix-table statistics), and rollout state (the
last-2-token window function) are all projections of the same suffix-local key space.
The instrument ceiling is not the corpus: 982‰ of baseline errors expose an observable
low-confidence signal (#837 instrument), and the content-answerable-novelty pool alone
is ~34‰ of held-out positions. The bag arms show real mutual information between window
content and the teacher answer that the deployed key discards. The redesign question is
precisely: **what key, computable in the deployed operation classes, carries
window-conditional structure the suffix key cannot?**

## 4. Candidate directions

Each direction lists: mechanism, falsifiers addressed, why it can clear where the dead
arms could not, deployed-lowering sketch, the **cheap instrument that runs first**, and
cost. Directions are ordered by the recommendation in §6; D1/D2 are the primary bets.

### D1 — Joint conditional keys: content evidence conditioned on the suffix

**Mechanism.** Replace the bag's *unconditional* per-token tables
(`content_next[t] → argmax counts`) with **joint conditional tables keyed by
(content-token, suffix-key)** — i.e., learn `P(answer | t ∈ window, suffix)` residuals
relative to `P(answer | suffix)`, rather than `P(answer | t)` relative to nothing (Ψ)
or relative to the global marginal (#891-CR, falsified). Bounded per-key caps as today.

**Why it is not a sixth arm of the dead space.** The five arms all scored evidence
whose key structure was fixed (suffix tables + unigram content tables); their λ-mixes
and marginal subtractions re-weighted the *same* conditionals. D1 changes the
conditional itself — the table's key space — which is exactly the compiler/representation
lever the REVISE verdict names. The #891 falsifier (global-marginal subtraction adds
nothing) is consistent with D1's premise: the useful baseline to subtract is the
*suffix-conditional*, not the global marginal, and that quantity was never represented.

**Addresses.** F1/F2 directly (window structure enters the conditional); F3 partially
(joint keys carry their own support counts, giving the calibrator non-singleton
evidence mass on exactly the positions that matter).

**Lowering sketch.** Same shape as the retired segment lane's PSTATE machinery
(bounded table, fixed-offset reads, saturating adds) with a composite key hash —
P-4-legal; reuses the #836 engineering that already exists dormant.

**Cheap instrument (runs first, hours not days).** Offline reference arm in the exact
#875/#891 harness mold, with the reproduction gate against the recorded 246.6‰
baseline: fit (content, suffix)-keyed residuals on TRAIN, score held-out, pre-registered
run contract, SELECT iff paired lower bound ≥ 20‰ (planning bar ≥ 25‰). Sparsity is the
known risk — joint keys fragment counts — so the instrument reports coverage of the
joint tables alongside the delta, and a backed-off mix (joint where supported, Ψ-bag
where not) is the pre-declared fallback arm within the same run.

**Cost.** Table build minutes-scale on the existing bundle; no teacher compute; no
format change until a positive.

### D2 — Region-conditional evidence: the geometric code as the context key

**Mechanism.** Use the artifact's own semantic code/region assignment for the full
window — H(x), which #784 showed is collision-free at full depth — as the key for
continuation-evidence tables (`region → bounded answer residuals`), with region
granularity (depth/radius) as the design knob. This is the "holographic" bet stated
plainly: the compiled geometry already summarizes the whole window; today no evidence
table conditions on it (the graph path consults cover/score structures whose measured
continuation distributions converge, #784).

**Addresses.** F1 (whole-window key), F2 (structure beyond the bag: the code is
order- and composition-sensitive where the bag is not), F3 (region occupancy counts are
natural, non-singleton support masses), and it is the only direction that also gives S3
a native hook (region-trajectory state for F4).

**Why it can clear.** The bag arms never touched the geometric layer; #784 established
distinctness (codes separate) but convergence of *continuation distributions at the
current granularity*. The open question — measurable cheaply — is whether
finer-granularity regions (or region × suffix product keys) carry conditional signal
the coarse convergence result hides.

**Cheap instrument.** Offline: recompute region assignments for held-out windows from
the existing artifact (no recompile at first — read the compiled codes), tabulate
region-conditional argmax on TRAIN, teacher-grounded delta on held-out, sweeping
granularity. Run contract first; the #784 convergence result is the null this
instrument must beat, so it is pre-registered as the primary falsifier.

**Cost.** Minutes-to-hours offline; a granularity change that requires re-inducing
covers escalates to compile-scale (hours) and is gated behind the read-only instrument.

### D3 — Deeper observation: raise the teacher-signal ceiling itself

**Mechanism.** The compiled evidence can never exceed what the observation protocol
recorded. Today's context windows are capped at `WINDOW = 8` tokens; **verification
item Q1 (do first, half a day):** confirm from the obs-shard producer what prefix
length the recorded `t_argmax` consumed — if the teacher labels themselves were
produced from short prefixes, every table above is fitting to a suffix-conditioned
teacher and the measured "suffix-locality" partly mirrors the observation protocol,
which would reframe D1/D2's ceilings. If labels are full-prefix, D3 becomes: widen the
*observation windows* (8 → 32/64) so compiled keys can condition on longer context.

**Addresses.** The ceiling under F1/F2 at its source.

**Cost.** Q1 is a reading exercise. A re-observation/recompile is the expensive branch
(exact-GEMM teacher ≈ 1 pos/s ⇒ ~3.3 days single-threaded for 288k positions;
parallelism and subsetting to a pilot corpus are the mitigations) — it launches only
with its own run contract, only if Q1 shows the ceiling is real, and only after D1/D2
instruments report, since they use the corpus that already exists.

### D4 — Structured content lanes (positional/distance-bucketed pairs)

**Mechanism.** Keep unigram-bag tables but add order structure: keys of
(token, distance-bucket) and skip-bigrams (t₋ₖ, suffix-last). A milder cousin of D1.

**Addresses.** F2 marginally. Kept as a comparison arm inside the D1 bake-off rather
than a direction of its own — it shares D1's harness for near-zero extra cost.

### Explicitly out of scope for this RFC

Sampling/decoding changes, code-space tuning at the current representation (both named
by the verdict as non-answers), reopening the retired segment lane without a new
pre-registered bar (#887), moving any frozen gate, and S3 corrective observation
(#840 — held; it targets whichever representation this RFC's outcome selects).

## 5. How this feeds S2 and S3 (alignment, not scope creep)

Whichever direction survives its instrument produces, as a by-product, **content-side
confidence features with real support mass** — exactly the evidence-acquisition
redesign the S2 REVISE verdict sanctions for the one calibrator re-entry (#823), and it
does so on the same frozen #838 gates. D2 additionally defines the state object S3's
corrective rounds (#840) would ground in. Neither stage's gates move; this RFC only
sequences the shared prerequisite.

## 6. Recommended sequencing and decision points

1. **Q1** (D3's verification item): confirm the recorded-label prefix length. Reading
   only. Outcome reframes ceilings but blocks nothing below.
2. **D1 instrument** (joint conditional keys, with D4 as an internal comparison arm)
   — run contract → offline reference run on the existing bundle. Exit: SELECT
   (lower bound ≥ 20‰; planning bar 25‰) → lowering track per the built-capability
   order; REVISE/sub-floor → record and proceed to 3; the backed-off-mix arm is the
   pre-declared fallback within the run.
3. **D2 instrument** (region-conditional evidence, granularity sweep vs the #784
   convergence null) — same contract discipline.
4. **Decision point:** if either instrument clears, S1 re-enters through an
   #836-shaped lowering with the #886-style deployed-fidelity spot-check pre-planned.
   If both are sub-floor **and** Q1 says labels are full-prefix, D3's re-observation
   decision goes to the maintainer with its own contract (it is the only remaining
   lever, and it is expensive). If both are sub-floor and Q1 shows short-prefix
   labels, the honest recording is that the S1 claim is bounded by the observation
   protocol, and the re-observation decision becomes the stage's single question.
5. Every instrument: pre-posted run contract, reproduction gate against the recorded
   baselines, planted negatives, CID-bound record, `docs/*_822.md` per-issue records —
   the discipline of the four studies that produced this verdict, unchanged.

## 7. Open questions for the maintainer (answers wanted on this PR)

- **Q1 priority** as stated (§4-D3) — agree it runs first?
- **Q2:** for D2, may the instrument read compiled region assignments directly from the
  released artifact (read-only), or do you want it derived through a designated API
  surface first?
- **Q3:** the ≥ 25‰ planning bar above the frozen 20‰ floor — adopt as the pre-declared
  SELECT bar for lowering-track *opening* (the floor itself stays the promotion gate)?
- **Q4:** budget stance on D3's expensive branch (multi-day teacher compute) — cap it,
  or defer any decision until step 4's evidence exists?
