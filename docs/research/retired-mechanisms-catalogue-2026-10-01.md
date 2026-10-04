# Catalogue of retired / failed / negative results — UOR-R4 Geometric Language Model

**Reconnaissance for principal-investigator review. Prepared 2026-10-01.**
Source checkout: `/Users/casey.allard/uor-r4-worktrees/pi-analysis-20261001` at `origin/main` = **`70d8a223`**
("Route the pointer by exact prime arithmetic (ADR-0003 prime router) (#1587)"). Read-only; no file in the
checkout was modified and no cargo build was run.

**Live source caveat, important for completeness.** The checkout's `docs/integration/current-state.md` is a
1794-line historical ledger whose newest entries stop before the last day of work. The authoritative and
*newer* record for the E1–E4 / compiler / session family is the live issue
[#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552) (85 comments, last 2026-10-01T22:29:55Z). Several
of the negatives below (v17, v20, v21, v22, v23, v24, v25) exist **only** there. A PI deciding on retirement
must read that issue, not `current-state.md`; `STATUS.md` (44 lines, updated 2026-10-01) is the only checked-in
file that reflects any of it.

**Governing disposition policy (this is what makes the catalogue ambiguous, so it is stated up front).**
- **D1/D2** (2026-09-19): a mechanism must show measured contribution against a matched control or be retired
  from the critical path; but an ablation is a *measurement, not a verdict*, geometric mechanisms are **never**
  retired on a near-zero delta, and only a census is reported (never conflated with practical significance).
  `selector`-class mechanisms explicitly may not be judged by aggregate loss.
- **D12** (2026-09-28, owner): **"Gates promote; they never kill."** Earlier `FAIL`/`DEAD`/`RETIRED` labels now
  read **"not yet promoted at that scope"**; parking a *family* needs a written root-cause case and the owner's
  OK; kill rules attached to earlier gates (e.g. B1's) are **withdrawn**.
- **D17 §2** (2026-09-30): prospective parity rule — two paired seeds/draws frozen *before* observation; one
  seed over tolerance ⇒ **one more predeclared seed**; mean of three decides. Equality belongs to retention.
- **D19 §3** (2026-10-01): the [October admissibility policy](../../docs/integration/mechanism-admissibility-2026-10.md)
  makes D12/D17/D9 operational — a failed promotion stays failed, a materially changed successor may re-enter.
- **`docs/integration/geometric-toolbox-2026-09-28.md`** is the register of "kept available" mechanisms.

Consequence: the *measured numbers* below are mostly solid. The **dispositions** are a moving target, and a
number of items marked "retired" between 2026-09-19 and 2026-09-28 were formally un-retired by D12 on
2026-09-28 before any re-measurement. That gap — un-retired on policy, never re-tested — is the largest single
risk to this programme.

---

## Part 0 — Methodological key used in the per-item verdicts

For each item I record: **(1)** name/date/document; **(2)** mechanism/hypothesis; **(3)** the acceptance
criterion with a verbatim quote and whether it was frozen prospectively; **(4)** exact measured numbers;
**(5)** verdict — `CLEAN`, `TOO TIGHT`, `CONFOUNDED`, `UNDER-POWERED`, `POLICY-KILLED`, `INSTRUMENT-DEFECT`;
**(6)** status.

Definitions used for the verdict:
- **TOO TIGHT** — the criterion was applied to a mechanism whose *instrument* could not have detected success,
  or the criterion tests a property the mechanism was never designed to supply.
- **CONFOUNDED** — two or more things changed together, so the measured delta cannot be attributed to the
  mechanism named in the conclusion.
- **UNDER-POWERED** — single seed / single draw / reuse of an exposed panel / a point estimate whose own
  sampling error is of the same size as the effect.
- **POLICY-KILLED** — stopped by process rule rather than by a measurement.
- **INSTRUMENT-DEFECT** — a documented defect in the harness or comparator that the document itself records
  but that the disposition did not fully absorb.

---

# Family 1 — Learned retrieval / selector / pointer (A1–A4, R1-X, R1, M-world v2)

This is the family the owner's question about "overly tight testing" most directly concerns. It contains one
pre-registered kill with a genuine prospectively frozen criterion, executed across ten arms, **and** several
gaps in that same experiment that were never closed.

## 1.1 A1 — the integrated attention–language prototype (2026-09-24)

- **Name/date/record:** "Integrated attention A1: loaded model, failed capability, identified learning
  boundary", 2026-09-24 · `docs/integration/integrated-attention-a1-result-2026-09-24.md`; plan
  `docs/integration/integrated-attention-a1-plan-2026-09-24.md`; delivery PR **#1386** (merged `b020f34a`).
- **Mechanism:** first single Rust artifact joining four finite-state lanes (2I geometric vs C120 ordinary),
  exact bounded occurrence memory, learned query/key codes, relative energy, read/write gates and a normalized
  sparse Generate/Copy/Stop head. Fitted with truncated state credit, not full BPTT through hard routing.
- **Acceptance criterion:** the plan states the endpoint, not a numeric gate: *"The complete next deliverable
  remains one loaded model that uses a source beyond its local token prior to produce an appropriate
  **uncopied** consequence and useful prose/code continuations, compared with its ordinary arm and NoRead"*
  (result §"Next development decision"). **No numeric gate was frozen before this run.**
- **Measured:** both arms wrote **6,160/6,160** development tokens; annotated source **admitted 271 (C120) /
  251 (2I)** of 1,421 but **selected correctly 0 / 0**; reads 122 / 24; selected-read token equals the actual
  next token **3 / 1**; Read bits/token **9.383427 / 9.533292**; NoRead **9.380686 / 9.554751**; **0/4**
  complete correction answers in both arms. The first fitted pair produced **zero writes and zero reads** on all
  6,160 tokens (C120 9.408436, 2I 9.615514 bits/token) because the write objective treated unannotated
  occurrences as negatives — a *supervision defect*, corrected prospectively and preserved.
- **Read-only diagnosis (the decisive one):** for **all 28 answer positions in each arm**, the annotated source
  is written and admitted, but there are **64 admitted candidates and only one distinct complete four-lane
  code**; all 64 energy scores tie; correct source ranks first **0/28 pre-gating**, gate opens **0/28**. Forcing
  the correct source as an offline one-step intervention changes NLL by **+5.137353 nats (C120)** and
  **+2.534511 nats (2I)** — positive is worse — i.e. the head had not learned to use the correct value either.
- **INSTRUMENT-DEFECT / UNDER-POWERED.** The document is unusually honest: it derives the exact representational
  consequence (*"if k_i = k_j, every factor of this reader's H(q,k_i) and H(q,k_j) receives the same input, so
  the energies are equal for any learned coefficients"*) and explicitly says this is *"measured code collapse
  within these candidate sets, not a claim about every token or every possible state."* But: one seed, 46,584
  tokens per fit, exposed development only, and 4 authored correction cases. The failure is a genuine
  **learnability/objective** failure at an extremely small dose, not evidence about the mechanism class. The
  authors say so; the ledger later treats A1–A4 collectively as "negative language evidence" (AGENTS.md,
  2026-09-24), which is a wider reading than the run supports.
- **Status:** retained as scaffold; **A1 training permanently stopped** under D18 outcome D (see 1.6).

## 1.2 A2 — address contrast and language credit (2026-09-24)

- **Record:** `docs/integration/integrated-attention-a2-result-2026-09-24.md`; PR #1387.
- **Mechanism:** delayed record commit (16 tokens after the source), shared Key/Query coarse address bank,
  fine codes for relative energy, offline contrastive positives/negatives, and joint natural-text +
  paired-permission-correction language training.
- **Measured:** corrected arms admit the right source on **1/24** new first-answer decisions, rank it first
  **0/24**, select after gate **0/24**, answer **0/12** complete read-enabled variants. Gate opens **22/24**.
  The **dedicated 2,097,152-coefficient context bank ended at exactly its initialized integer values — zero net
  changed coefficients — despite 125,424 contrastive updates per arm.** Forced-source one-step NLL 61.806803
  (C120) / 58.884029 (2I) vs evidence-removed 115.862965 / 114.491525; actual hard-read NLL 119.330910 /
  115.052391 — *"opening the gate more often with the wrong source does not deliver the offline benefit."*
- **Critical confound the document itself discloses:** *"The soft overlap loss at temperature 8 and hard
  single-page admission are different objectives."* The training objective never optimised the hard decision
  that was then gated. A **read-only exhaustive V4096 scan** found the target is surface-MAP on **0/24** in the
  corrected pair for both arms, while an independent NoRead session has colon as MAP on **24/24**; forcing
  correct evidence makes the target MAP on **11/24**. So the *learned* output preference (colon) dominates and
  it is a surface artifact, not a retrieval artifact.
- **CONFOUNDED.** The headline "hard routing remains unresolved" is fair, but the run simultaneously changed
  commit timing, address bank, contrastive objective and language credit; the zero-coefficient result is
  attributed by the authors to soft-vs-hard objective mismatch and to per-step integer rounding with
  unretained master trajectories (*"Master trajectories were not retained, so sub-bin movement, cancellation
  and oscillation are not distinguished"*). That is an **instrumentation gap**, not a mechanism verdict.
- **Status:** superseded by A3.

## 1.3 A3 — integer routing margin learns admission (2026-09-24)

- **Record:** `docs/integration/integrated-attention-a3-result-2026-09-24.md`.
- **Mechanism:** bounded deterministic signed-4-bit coordinate edits on dedicated lane-0 context coefficients,
  minimising integer margins + wrong-entity collisions + page pressure; accepted edits change the served
  artifact directly.
- **Measured:** source **admitted 1/24 → 17/24**; ranked first **2/24**; selected after gate **0/24**;
  complete correct read-enabled answers **0/12**; dev bits/token regressed (6.854878 → 6.929851 C120;
  6.798124 → 6.909304 2I). 188 accepted edits, objective 26,393 → 3,665, fit admission 6/192 → 167/192.
  Fine lanes had **zero net exported coefficient changes** despite ~4,410 query and ~4,410 key gradient calls.
  Page concentration: development queries use only coarse codes **47 and 71**, 12 cases each; maximum
  same-code load 87 fit / 83 dev against a 64-posting cap; **search incomplete on 4/24 dev and 35/192 fit**.
- **Verdict: the *ranking/gate* failure is genuinely measured and decomposed (CLEAN as a diagnosis).** The
  gate is separately identified as the suppressor: both correctly-ranked rows have gate score **−1** (bias −1
  plus six zero active weights), and the gate sees only previous token, four State codes and candidate token —
  **no Key/relative-code and no occurrence/version features**. The output head is separately identified as
  insufficient: forcing the correct source yields the target as true surface MAP on only **6/24 (C120)** and
  **2/24 (2I)**, and the bounded greedy decoder emits it on **1/24 and 0/24**.
- **Status:** superseded by A4.

## 1.4 A4 — operative integer reads, failed retention (2026-09-24)

- **Record:** `docs/integration/integrated-attention-a4-result-2026-09-24.md`.
- **Measured:** new first source **admitted 17/24**, **ranked 2/24**, **selected 2/24** (vs 0/24 controls);
  fit sources retained after export **28/192 (C120) and 24/192 (2I)** out of 167 admitted;
  **0/12 complete correct answers in every arm**; dev bits/token C120 A4 **6.839477** vs control 6.799839
  (worse), 2I A4 **6.798995** vs control 6.872785 (better).
- **The measured failure mode is retention, not generalisation:** *"Immediately after their own local update,
  all 1,336 eligible provenance examples per arm choose the positive source. The final loaded models retain
  only 28/167 and 24/167 of the admitted fit sources. This is an inability to retain and reconcile the training
  decisions, before generalization is even considered."* The document also **corrects its own earlier claim**:
  unchanged context coefficients do not mean unchanged fine codes — query codes change on 21/24 and source
  codes on 18/24 (C120) / 21/24 (2I). *"Representation drift, objective interference and trajectory changes are
  not separately identified by this run."*
- **CONFOUNDED (self-declared).** The run cannot separate interference from the 22,080 later-answer utility and
  3,208 natural utility examples, from representation drift, from objective conflict. It is nonetheless a
  correct *engineering* negative: the exported artifact did not do what the fitter did.
- **Status:** successor line stopped. `project-track.md:483`: *"**Stop the succession of local-credit selector
  adjustments.** [A4] operates but ends at 2/24 correct first reads and 0/12 complete answers in each arm; most
  online-fitted source choices are lost by export. A1–A4 remain negative evidence and reusable native
  scaffolds."* D8 (2026-09-24) made this a standing rule. **This is the single most consequential retirement in
  the family, and it was made on A1–A4 at 46k–1.1M tokens with 4 authored correction cases.**

## 1.5 Causal continuation — crossed-feedback credit (2026-09-24)

- **Record:** `docs/integration/causal-continuation-result-2026-09-24.md`.
- **Mechanism:** training-only intervention supplying the copied-feedback token while keeping selected
  evidence, typed facts and copy length fixed, to break a measured source-fingerprint shortcut.
- **Measured:** **crossed feedback 16/16** (including 4 gradient-withheld cells 4/4) vs 8/16 for the
  matched ordinary-supervision arm and 8/16 for the parent. Cost: **+0.01799 bits/target** vs ordinary training
  and **+0.06955** vs the parent on repository development. Final-source panel: parent 11.471785118, ordinary
  10.598857531, causal **10.614216096** (causal trails ordinary by **0.015359**). Authored instruction test
  **0/4 dialogue answers exact, 0/4 generated Rust outputs compile** (numbers 8–11 vs 0–7 in training).
  Q8 relative-action learner 1,024/1,024 but an ordinary 384-operator control also reaches **1,024/1,024**
  after a tie-break repair — *"The initial apparent predictive gap is therefore not a defensible unique
  geometric advantage."*
- **Verdict: CLEAN as a negative, and an exemplary one.** The 16-cell panel is explicitly labelled an *exposed
  selection gate* with the 4 withheld cells *"gradient-withheld, not an untouched final acceptance
  population."* Only one training seed. The geometric claim was self-refuted by adding the ordinary control.
- **Status:** the interventional credit mechanism itself is **not** re-listed as retired in the toolbox; the
  geometric-advantage claim from it is dead. `0/4 code compiles` is the direct ancestor of the still-open
  coding obligation.

## 1.6 A1 under D18 — outcome D (2026-10-01, owner-confirmed) — **the one genuinely pre-registered kill**

- **Records:** [D18](../../docs/integration/DECISIONS.md) §2 and §10; `docs/integration/current-state.md`
  §"D18 outcome D — 1 October (owner-confirmed)" lines 269–310; live issue
  [#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552) comments 2026-10-01T01:47:44Z and 01:48:29Z;
  pre-registration is #1511 comments 5898059603 / 5902211457 and `a1_gate` in
  `crates/uor-r4-training/src/milestone_world_v2.rs`.
- **Criterion, verbatim and prospective:**
  > **"Gate (unchanged).** On the development-phrasing × development-value cell, MQAR recall ≥0.9 at distances
  > 16, 64 and 200 AND open-relation recall ≥0.9."
  > **"Kill (unchanged).** Every arm, including T and T at 2× steps, below 0.5 at distance 16. Track A
  > retrieval then moves to the exact log plus the prime sieve, with no third round."
  Instrument freeze was required first: `instrument_freeze_ok` true (R-recency and R-nlet each below 0.6 on
  every gated cell), every MQAR distance bucket populated, every probe item ≤256 tokens, digest posted before
  any treatment run.
- **Measured (development cell, `m-world evaluate world=v2`, 300 conversations, seed 9101; one training seed per
  arm; ~2.1M parameters; `corpus-v2.1`; #1017 tokenizer; every training run binary `47850579…`, every
  evaluation `7edd6cc8…`):**

  | Arm | Steps | MQAR all | d16 | Open relation |
  |---|---:|---:|---:|---:|
  | P: Lorentz read, no pointer | 2,590 (wall) | 0.009 | 0 | 0/52 |
  | F7: Lorentz with trained flock (w16, k7) | 2,590 | 0.009 | 0 | 0/52 |
  | T: transformer control, no pointer | 1,908 (wall) | 1/109 | 0 | 0/52 |
  | **T at 2× steps** | **5,180** | 1/109 | **0** | 0/52 |
  | Lorentz read + dot pointer, seed 1 (L1b) | 2,590 | 37/109 (0.339) | 0.27 | 20/52 |
  | Lorentz read + dot pointer, seed 1 (L1) | 2,000 (wall) | 0.35 | 0.24 | 0.42 |
  | Lorentz read + pointer, seed 1 (wall-stopped) | 2,566 | 0.44 | 0.41 | 0.38 |
  | Lorentz read + dot pointer, seed 2 | 2,590 | 0.28 | 0.38 | 0.23 |
  | Dot read + dot pointer, seed 1 | 2,590 | 0.34 | 0.27 | 15/52 |
  | Dot read + dot pointer, seed 2 | 2,590 | 0.34 | 0.22 | 27/52 |
  | *R-recency (untrained instrument rule)* | — | 0.36 | 0.29 | 0.52 |

- **Verdict: `CLEAN` on the letter of the criterion; `CONFOUNDED` on the causal reading that was drawn from
  it.** Justification, in three parts:
  1. **The kill is unambiguous and was pre-registered.** Every arm is below 0.5 at d16 — most at exactly 0 —
     and the transformer control at 2× steps also scores **0**. The gate (≥0.9) is missed by an enormous
     margin, not a borderline one. These are ten arms, not one. I do not think this specific criterion was
     unreachable-by-construction.
  2. **But the experiment that ran was not the experiment the question needed.** The pointer-arm result is
     driven almost entirely by the *pointer head*, and the transformer control was run **without a pointer**:
     *"Attention without a pointer does not learn retrieval here, transformer or geometric. T, P and F7 all
     score 1/109."* The A1 design therefore never tested **transformer + pointer**, which is the standard
     control for "can this scale learn keyed retrieval at all?" The document names this itself — outcome C
     ("read defect") was defined as *"T beats the best stack arm by ≥0.2 at distance 64"*, and that comparison
     was never available. Outcome D therefore certifies **"no pointer-class mechanism learns keyed retrieval at
     ~2.1M / 2,590 steps on this corpus"**, but it was recorded and reported as **"no learned retrieval at this
     scale"** (D18 §3 outcome D).
  3. **The untrained-rule comparison shows the scale is the binding constraint, not the geometry.** Pointer arms
     score MQAR 0.28–0.44 against an **untrained** recency rule at 0.36 and an untrained n-let rule where
     applicable; recall falls as ≈1/N (0.34–0.64 / 0.17–0.25 / 0.08–0.17) which is the signature of copying the
     latest value. *"Within the evaluation's sampling error (about ±0.05 at 109 items), no pointer arm beats the
     untrained rule."* At 109 items the evaluation's own sampling error (±0.05) is comparable to the entire
     spread being used to declare arms interchangeable, and every conclusion rests on **one training seed per
     arm**.
- **Two collateral losses inside the same kill, both avoidable:**
  - **The Lorentz-vs-Dot read parity was closed while explicitly undecided.** D17 v2 was invoked and required a
    third paired seed — pre-declared at 01:16:51Z *before any observation*: *"For each metric: keep the Lorentz
    read if the mean of the three d is ≤ 0.03; otherwise select the Dot read at this scope, keeping Lorentz in
    the toolbox (D12)."* Pair 1 (seed 1) MQAR d = +0.018, open relation d = −0.096; pair 2 (seed 2) MQAR
    d = +0.055, open relation d = +0.288. Exactly one metric exceeds tolerance in each pair ⇒ **third seed
    required**. The third seed was new training, so D18 §3 forbade it, and the owner recorded *"Lorentz vs Dot
    is recorded as undecided at two seeds"* (01:48:29Z). **This is the clearest `POLICY-KILLED` item in the
    catalogue**: a pre-declared decision procedure that could not be completed because the same experiment
    killed the track it belonged to. `current-state.md:295` states it plainly: *"Lorentz vs Dot read parity
    (D17 v2) is **undecided at two seeds**."*
  - **The specified pointer was never the pointer that ran.** D18 §2 arms list *"P+ptr: Lorentz pointer"*, and
    §10 records: *"D18's 'P+ptr' was specified as a Lorentz-scored pointer. Every pointer arm run used the
    default dot-scored pointer, so the Lorentz pointer did not run."* Every "geometric retrieval" conclusion
    from A1 is therefore about a **Dot-scored pointer inside a Lorentz-read model**.
- **A third, purely mechanical cap on the whole family:** *"Copy is 0/33 in every arm, for a structural reason.
  Under the #1017 tokenizer (`add_prefix_space: false`), a reply-initial word's first token (`m` = 79) never
  occurs in the user's text (`Ġm` = 283), so a pointer cannot copy it."* This tokenizer-boundary defect
  independently prevents any copy-based credit from appearing in A1, and it **persisted**: the copy panel is
  still **0/33** in the newest results on #1552 (v25, 22:29:55Z). It is now the explicit first target of the
  prime-route pointer.
- **Status:** D18 §10 consequences — no new A1 training; the exact-log + prime-sieve design memo started
  immediately; **the 7M fit (conditional on outcome A) did not run**; **the pointer's D11 port did not run**;
  the pointer head stays in the toolbox. Track A retrieval moved to the exact log + prime sieve.
- **PI flag:** the *decision* was clean. The **scope of the conclusion** should be re-opened. Specifically:
  (i) a transformer-with-pointer arm was never run; (ii) the Lorentz-scored pointer was never run; (iii) the
  third read-parity seed that D17 v2 required was never run; (iv) copy was structurally 0/33 for a tokenizer
  reason; (v) all of it is one training seed per arm at ~2.1M parameters against an untrained baseline within
  ±0.05 sampling error. Under D12 and D19 §3 (a materially changed successor may re-enter with a causal
  prediction), **items (i)–(iv) are each a materially changed successor and none has been run.**

## 1.7 R1-X — the R1 experiment that produced the other half of outcome D's premise

- **Record:** #1552 comment 2026-10-01T04:05:03Z.
- **Measured:** *"R1 holds the exact training phrasing × values (closed relation 0.867), falls to **0.154** with
  new phrasings, and 0/64 on development relation recall."* D18 §1 rests its "measured" leg on this: *"Every
  English memory score is 0/10, and R1's development relation recall is 0/64."*
- **Verdict: UNDER-POWERED as a premise for a two-week programme-wide kill.** A single trained lineage
  evaluated on a cell it was not trained on; the paired phrasing-transfer contrast is informative, but the
  figure "0/64" is quoted in the decision text without its counterpart (0.867 in-distribution), which changes
  its meaning from "no retrieval" to "no phrasing transfer".

---

# Family 2 — Bounded admission, compact codes, precision and quantization

## 2.1 `orthant64` bounded admission (2026-09-25) — the failure that produced D9

- **Records:** `docs/integration/bounded-admission-plan-2026-09-25.md`,
  `docs/integration/bounded-admission-result-2026-09-25.md`,
  `docs/integration/bounded-admission-outputs-2026-09-25.md`; evidence
  `docs/evidence/bounded-admission-result-2026-09-25.json`,
  `bounded-admission-decision-2026-09-25.json`. Source `78fde5711ea01f80783eb9911e01197a61406d1a`.
  Rejected artifacts retained at
  `/Users/casey.allard/uor-r4-investigations/bounded-admission-20260925/fit-{quaternion,householder_pair}-bounded-2/packed-model`.
- **Mechanism:** an orthant (sign-pattern) geometric partition that admits **at most 64 selected events** out
  of the full 256-token causal history, used identically during learning and incremental generation; compared
  against the same final weights under full, recent64, recent32 and exact_cache64 access.
- **Pre-registered criterion (frozen in the plan before the run):**
  > Each transport's **bounded retention** requires all of:
  > 1. Comparison NLL no more than **+0.05** above its own retained learned-code parent …
  > 2. … NoRead effect …
  > 3. … five noncollapsed generations …
  > 4. **at most 2 first-noun losses against each anchor** …
  > 5. … mechanical/source/export checks …
- **Measured (256 updates / 1,048,576 fitted target visits per arm at B16/T256; one paired seed):**

  | Comparison-tail NLL, nats/target | Quaternion | Householder pair |
  |---|---:|---:|
  | Retained learned-code parent | 2.114226169 | 2.110879800 |
  | Frozen bounded entry | 2.170559173 | 2.171663554 |
  | **Final bounded orthant64** | **2.129595124** | **2.124890468** |
  | Same final weights, **full** | 2.080040745 | 2.069662849 |
  | Same final weights, **recent64** | 2.119169266 | 2.108628005 |
  | Same final weights, recent32 | 2.217738285 | 2.208398679 |
  | Same final weights, **exact_cache64** | 2.146801669 | 2.136447411 |

  Complete source answers: parent 28/32 and 24/32; **orthant64 17/32 and 17/32**; same-weights full **27/32 and
  25/32**; recent64 **27/32 and 25/32**; exact_cache64 **23/32 and 18/32**. Gate vector
  **PASS/PASS/PASS/FAIL/PASS**. First-noun losses **11/8** against the immediate parents and **11/9** against
  rung 1, against a maximum of 2. Bounded-minus-parent NLL gaps **+0.015368955 / +0.014010668**.
- **The document's own reading is the key technical finding:**
  > *"Restoring full access on the same final weights recovers 10/8 complete answers, localizing much of the
  > loss to the admission policy rather than an inability of those weights to use the information. … Indexed
  > complement utility also passes versus recent32 (17 additional complete answers; late-position NLL gains
  > 0.110267/0.105152), but cannot override retention failure."*
- **Verdict: `CLEAN` on criterion, `TOO TIGHT` on instrument, `INSTRUMENT-DEFECT` on the recent64 comparison.**
  - **Clean:** the criterion was in the plan before the run; the tolerance is explicit; the gate vector is
    reported per-arm; the failing sub-gate is named. It was not retrofitted.
  - **Too tight:** the mechanism **improved NLL over its own parent in both arms** (+0.015368955,
    +0.014010668 — better, not worse) and passed the NLL gate. It was killed by a **12-row complete-answer
    count** (11 first-noun losses vs a maximum of 2) on a single paired seed, while the same run demonstrates
    that the loss is recoverable by changing only the access policy. A retention gate whose failing sub-gate is
    a small-N count, on a candidate that wins the primary loss metric, is exactly the "near-zero delta used as
    a verdict" pattern D2 warns against. This is the strongest `TOO TIGHT` case in the catalogue.
  - **Instrument defect in the comparison that decided "recent64":** `exact_cache64` was *"not selected as the
    next bounded reference"* on 23/18 vs recent64's 27/25 — but the document states the *bounded* arm's own
    utility discriminator passed only *"versus recent32"*. Three access policies were compared on complete-answer
    counts at n=32 (one answer = 3.1 percentage points) with one paired seed; the ranking recent64 > exact_cache64
    > recent32 is within that resolution.
  - **Same-weights caveat is explicit and correct:** *"The full-access parent receives no equal extra training,
    so this is not an equal-compute superiority comparison."* Good — but it cuts both ways: the *failure* is
    also not an equal-compute comparison, since the bounded arms received 256 extra updates the parents did not.
- **Status:** **not promoted**, preserved. D9 (2026-09-25) made this the trigger for a programme-wide process
  correction; `geometric-toolbox-2026-09-28.md` §C re-lists orthant64 as a kept piece
  (*"Neutral against full admission: 17 of 32 answers in both arms"* — note this phrasing describes it as
  *neutral*, while the result that killed it said *failing*). **Sparse admission is deferred, requiring a
  demonstrated implementation need, a candidate-recall learning mechanism and a discriminating comparison
  before another fit** (D9). It has not been reopened.

## 2.2 `recent64` training follow-up — withdrawn before execution

- **Records:** [D9](../../docs/integration/DECISIONS.md) (*"The later recommendation to train recent64 is withdrawn
  before execution"*); D9 ¶"Current implementation direction" (*"Sparse admission is deferred as a separate
  optimization, requiring a demonstrated implementation need, a candidate-recall learning mechanism and a
  discriminating comparison before another fit"*); `current-state.md` line 934 (*"The recent64 training
  follow-up is withdrawn under the owner-directed D9 correction"*); AGENTS.md progress-control block.
- **Verdict: `POLICY-KILLED`, unambiguously.** The recommendation came from the orthant64 result, where
  same-weights recent64 recovered **27/25** complete answers with **lower NLL than either tested indexed
  proposer** — i.e. it was the best-measured access policy in the entire experiment. It was withdrawn by a
  process rule (prevent experiment loops), not by any measurement, and the same-weights diagnostics that
  motivated it are explicitly *"not independently trained policy superiority."* **No trained recent64 model
  has ever existed.** The owner's specific question — "were good mechanisms retired too early by overly tight
  testing?" — is answered **yes** for this item: the only evidence anyone has about recent64 is favourable, and
  it was never fitted.
- **Note:** this is *not* a violation of D9's purpose. It is nonetheless a live, cheap, unused experiment: train
  full-context-256 with recent64 *access* so that access matches the deployed session, then re-run the orthant64
  gate. Nothing in D19 forbids it; the re-entry condition in D18 §6 (*"Memory port, AERM, G v2 probes, I1
  integration. Re-entry: outcome D, or §8 passes"*) is now **satisfied — outcome D occurred on 1 October.**

## 2.3 `exact_cache64` — dropped without a fair comparison

- Same records as 2.1. **Measured:** exact_cache64 complete answers **23/32 / 18/32**, NLL **2.146801669 /
  2.136447411** — worse than recent64 on both metrics and worse than orthant64 on NLL in both arms.
- **Verdict: `CLEAN`.** It is dominated on the measured metrics. Two caveats: it is an *exact* cache, so it
  should not have lost to a recency window on a task where exactness matters — which suggests the cache
  implementation or its interaction with the shared state is what failed, not "exact cache" as an idea.
  The document does not diagnose the mechanism of its loss. **Status:** not selected as the next bounded
  reference; kept in `geometric-toolbox-2026-09-28.md` §C under "Orthant (sign-pattern) bounded admission" and
  "E8/H4 fixed codebooks; product-key memory (cycle 5: NOT_RUN)".

## 2.4 Precision factorial — quantized vs float parameters and interfaces (2026-09-25)

- **Records:** `docs/integration/precision-factorial-result-2026-09-25.md`,
  `docs/integration/precision-factorial-review-2026-09-25.md`,
  `docs/integration/precision-factorial-outputs-2026-09-25.md`; evidence
  `docs/evidence/precision-factorial-{result,endpoints,closeout}-2026-09-25.json`. Source
  `49dfd0b15486ba3a7a763c598257f651e857279f`. Container
  `/Users/casey.allard/uor-r4-investigations/precision-factorial-20260925`.
- **Mechanism:** evaluate all four precision combinations (Floating/Quantized parameters ×
  Floating/Quantized interfaces) on both retained step-8,348 parents, to separate **parameter** quantization
  cost from **interface** quantization cost and their interaction.
- **Criterion:** the original `+0.05` nats allowance against continuous (the discretization gate), plus
  behavioural retention of source answers. **No new criterion was frozen for this diagnosis** — it is a
  fixed-parent comparison, and the document says so.
- **Measured:**

  | Comparison-tail NLL, nats/target | Quaternion | Householder pair |
  |---|---:|---:|
  | Floating params / floating interfaces (FF) | 2.113220567 | 2.098025647 |
  | Quantized params / floating interfaces (QF) | 2.150291052 | 2.147912004 |
  | Floating params / quantized interfaces (FQ) | 2.113217828 | 2.098039459 |
  | Quantized params / quantized interfaces (QQ) | 2.150319798 | 2.147928712 |

  At floating interfaces, parameter quantization costs **+0.037070 / +0.049886** nats/target. Interface
  quantization costs **−0.000002739 / +0.000013812**. Interaction **+0.000031484 / +0.000002896**.
  Exact source completions, FF/QF/FQ/QQ: **25/22/25/23** of 32 (quaternion), **28/22/28/22** (ordinary).
- **The finding the PI should weigh most heavily:** *"Small mean interface loss is not behavioral identity:
  FF→FQ changes nine of ten free generations, and QF→QQ changes eight of ten."* And *"Even QF misses the
  original +0.05 allowance against continuous."*
- **Verdict: `CLEAN` and unusually well-scoped.** The document explicitly limits itself: *"This diagnoses these
  fixed forward paths; it does not isolate individual tensors, predict retraining recovery or establish
  geometric advantage."* The parameter/interface separation is a genuine, well-powered, 8-evaluation factorial.
- **Status:** *"Both prior quantized recipes remain rejected and all computation here remains F32 emulation."*
  The successor — *"one paired learning of neighboring integer-code choices"* — ran as
  **learned-rounding** (2.5) and **passed**. So this negative was correctly superseded rather than retired.

## 2.5 Learned rounding — the accepted successor (2026-09-25), and what it still did not fix

- **Record:** `docs/integration/learned-rounding-result-2026-09-25.md`. Source `036eabcc…`.
- **Measured:** 512 alpha-only updates / 2,097,152 fitted target visits per arm, B16/T256. Hard codes
  **2.114226169 / 2.110879800**; hard-minus-continuous **+0.023707670 / +0.046476579**, both within the +0.05
  allowance; whole-prefix NoRead 2.609608832 / 2.607989949. Five-gate vectors **PASS/PASS/PASS/PASS/PASS**.
  Exact source completions **28/32 and 24/32**; **both still lose 3 previously correct complete answers against
  rung 1** (first-noun losses 2/1, inside the frozen maximum of 2).
- **Verdict: this is the one clean *positive* in the family, and it is the accepted parent for everything after
  it.** But note the structural weakness that travels forward: **one paired seed**, exposed development, and the
  continuous references are the *original step-8,348 anchors* while the new code candidates received extra
  learning — *"This one-paired-seed result does not establish equal-compute superiority."* Every subsequent
  gate (orthant64, projected, language continuation) is measured against these artifacts, so the entire
  discretization ladder inherits a single-seed baseline.
- **Status:** accepted parents remain current.

## 2.6 Projected-recurrent continuation — rejected at the +0.05 gate (2026-09-25)

- **Records:** `docs/integration/projected-recurrent-plan-2026-09-25.md`,
  `docs/integration/projected-recurrent-result-2026-09-25.md`,
  `docs/integration/projected-recurrent-review-2026-09-25.md`; PR **#1391**; evidence
  `docs/evidence/projected-recurrent-{result,closeout}-2026-09-25.json`.
- **Mechanism:** continue each preserved fully-quantized midpoint **7,836 → 8,348** (+512 updates /
  2,097,152 target visits, matched B16/T256), applying projection to the packed parameters at entry and after
  **every AdamW update**, preserving moments, clocks, fixed scales and all five original gates.
- **Pre-registered criterion (unchanged from the discretization ladder):** packed-minus-continuous
  **≤ +0.05 nats**, plus the other four gates.
- **Measured:**

  | Comparison-tail NLL, nats/target | Quaternion | Householder pair |
  |---|---:|---:|
  | Matched continuous continuation | 2.090518499 | 2.064403221 |
  | Original **unprojected** packed QAT | 2.149631373 | 2.144230654 |
  | New **projected** packed model | 2.150319798 | 2.147928712 |
  | Same projected shadows, quantizers disabled | 2.113220567 | 2.098025647 |
  | Projected packed whole-prefix NoRead | 2.637288928 | 2.618333390 |

  Packed-minus-continuous **+0.059801 / +0.083525** — both exceed +0.05 ⇒ rejected. Other four gates pass:
  NoRead penalties +0.486969 / +0.470405, five noncollapsed generations per arm, 2/1 lost rung-1-correct first
  nouns, loaded parity. Exact completions **23/32 and 22/32**.
- **The most interesting measurement in the whole quantization line, and it is a negative about the method:**
  > *"Entry projection brings **41,184 / 43,770** out-of-range shadows to zero while preserving all hard codes,
  > all actual full-context probabilities and optimizer/sampler state. Both final models still have **zero
  > clipped parameter coordinates**. Shadow likelihood improves and packed-versus-shadow state RMS falls, yet
  > **packed likelihood does not improve**."*
- **Verdict: `CLEAN`.** Pre-registered, matched, both arms, explicit numbers. One paired seed and exposed
  development (*"This is one paired seed on exposed development, with no fresh holdout opened"*), which the
  document states.
- **A genuine technical anomaly worth a PI's attention:** heavy projection during training made the *shadows*
  better and the *served packed model* no better. That means the binding constraint in this ladder is **not**
  coordinate clipping. The document does not pursue why. The successor (`precision-factorial`) then showed the
  cost is in the parameter representation collectively, not in 4-bit matrix rounding per se:
  *"The reference substitutes packed matrices, scalars, biases, folded normalization gains and head values, so
  the localization is to the packed representation collectively, not uniquely four-bit matrix rounding."*
- **Status:** both projected and unprojected artifacts preserved; rejection unchanged. `geometric-toolbox`
  §D lists "exposure-only continuation of the 1.68M native model" as an ordinary negative.

## 2.7 QAT on geometric_s1 and on the S2 dialogue stack (2026-09-29)

- **Records:** `docs/integration/d4-geometric-s1-qat-result-2026-09-29.md`,
  `docs/integration/d4-s2-dialogue-qat-result-2026-09-29.md`.
- **geometric_s1:** straight-through QAT, 1,000 steps, lr 0.0005 ⇒ exported integer NLL **1.973127** vs float
  continuation control **1.955115** = **+0.018012 nats**, inside the ≤0.02 nats gate; **−0.024986** below the
  original float 1.998113. Multiplier-free D11 logit parity **bit-identical** (`max_abs_diff == 0`); integer NLL
  matches served forward within **7.79e-7** nats (≤1e-5 gate). Export 4.2500 raw parameter bpw (4.7234 total
  container bpw). **PASS.**
- **S2 dialogue stack:** 1,024-step QAT ⇒ **56/58 turns (96.55%)** integer-vs-served-forward kernel agreement,
  but **greedy agreement against the pre-adaptation float baseline is 5/58 (8.62%)** and against the matched
  float continuation control **6/58 (10.34%)**, against a **pre-registered ≥29/58** float-agreement gate.
  **The unquantized float control itself drifted to 7/58 turns.** Served NLL 2.5680 nats vs matched float
  control 2.5649 (**+0.0032**), missing the literal registered target ≤2.5255 and the corrected parent target
  ≤2.5509 *"due to base float drift."* QAT integer vs its own unquantized float weights **was not evaluated and
  remains missing.**
- **Verdict, S2 QAT: `INSTRUMENT-DEFECT` / `UNDER-POWERED`.** This is a textbook case for the PI. The gate asks
  QAT to preserve agreement with a baseline that **no longer exists** — the matched *unquantized* float
  continuation drifted to 7/58 turns by itself on the same schedule. QAT scored 6/58 against that control,
  i.e. **within one turn of the float control's own drift**. The registered ≥29/58 threshold therefore measured
  schedule drift, not quantization fidelity. The document says this in its own words (*"missed due to base float
  drift"*) and flags the missing measurement (*"QAT integer vs own unquantized float weights was not evaluated
  and remains missing"*). **The single measurement that would have isolated the quantization effect was never
  taken.** The kernel-level result (56/58 = 96.55%) is the informative number and it is good.
- **Status:** D18 §6 parks *"QAT and codec reruns on the old lineages"* with re-entry *"on item 5's kill"*
  (served recall more than 0.10 below float). Track B parked.

## 2.8 B3 — E8 lattice weight coding on SmolLM2-360M MLP layers (2026-09-29) — **kill criterion triggered**

- **Records:** `docs/integration/b3-e8-smollm2-result-2026-09-29.md`; evidence
  `docs/evidence/b3-e8-smollm2-360m-2026-09-29.json`; issue **#1519**; PR **#1478** (D18 §6 parks the B3 rerun).
- **Mechanism/hypothesis:** $E_8$ vector quantization (QuIP# E8P codebook + Randomized Hadamard Transform) of
  all 32 layers of SmolLM2-360M's MLP weights (`gate_proj`, `up_proj`, `down_proj`; **235,929,600 weights**)
  at 2/3/4 bpw, against RTN 4-bit (~4.25 bpw) and matched-bit scalar controls, on 32 windows × 1,024 tokens
  (32,768 tokens) of SimpleWiki.
- **Criterion:** kill if RHT-E8P 3-bit is more than **+0.05 nats** worse than RTN 4-bit. (Stated as the
  pre-registered kill criterion in `current-state.md`.)
- **Measured:** float reference 2.122189 nats.
  RTN 4-bit **2.193682** (+0.071494, 4.2765 bpw, 85.61% top-1).
  RHT+RTN 4-bit **2.237834** (+0.115645, 4.2766 bpw, 82.74%).
  RHT+RTN 3-bit scalar control **2.850512** (+0.728323, 3.2766 bpw, 61.03%).
  RHT+E8P 2-bit **13.723733** (+11.601545, 2.0134 bpw, 0.03%).
  RHT+E8P 3-bit **7.084971** (+4.962782; **+4.891289** over RTN 4-bit, 3.0134 bpw, 12.51%).
  RHT+E8P 4-bit **5.814762** (+3.692573, 4.0134 bpw, 24.57%).
  All matrices passed exact bit-for-bit codec round-trip gates.
- **Verdict: `CONFOUNDED` — as the programme itself later found.** D16 item 4 records *"the instrument of
  `b3-e8-smollm2-mlp/attempt-full-32layers` is disputed. **Its float reference scored 9.45 nats/token on
  SmolLM2, consistent with #1017 token IDs (max 4095) fed to a 49,152-token model.** It is not a verdict on E8
  until a re-run with a float-NLL validity band."* There is a second confound the result itself shows: the
  **matched scalar control also degrades violently** at the same bit budget (RHT+RTN 3-bit +0.728 nats), so the
  measurement is at least partly about RHT + low bit-width rather than about the $E_8$ lattice. And the E8P arms
  were run **only on MLP weights, not embeddings or attention**, on 32,768 tokens of one corpus, with a single
  k-means/seed configuration.
- **Status:** *"negative result preserved; kill criterion triggered"*; **Track B's second lever halted**;
  D18 §6 requires *"a one-matrix exhaustive-encoder check"* before any B3 rerun. **This is a `CONFOUNDED`
  `POLICY-KILLED` combination: the family was halted on an instrument the project had already flagged as
  invalid.**

---

# Family 3 — Language: E3 compiler gate, E4 route, log-sieve, and emitted language

## 3.1 E3 relation compiler — gate never met across v8/v9/v10/v11/v14/v23/v24/v25

- **Records:** `docs/integration/grounded-memory-evidence-2026-10-01.md`;
  `docs/integration/log-sieve-retrieval-design-2026-10-01.md`; #1552 comments at 03:45:47Z (v8), 04:12:18Z
  (v9), 05:58:53Z (v11), 07:59:24Z (v14), 19:09:40Z (v23), 20:16:58Z (v24), 22:29:55Z (v25).
  `STATUS.md` §"Semantic compiler" is the only checked-in summary.
- **Frozen gate, quoted:** relation **≥ 0.9**, act **≥ 0.95**. `grounded-memory-evidence-2026-10-01.md:19`:
  *"The unchanged .9 relation/.95 act gate remains unmet."* D19 §3 confirms *"Existing E3 thresholds and §8
  acceptance are not weakened."*
- **Measured progression on development phrasings × development values (all development cell, one draw):**

  | Card | Trunk | Relation | Act | Note |
  |---|---|---:|---:|---|
  | v8 | frozen `emit-1` trunk | 0.712 | 0.709 | 373 teacher paraphrases |
  | v9 | R1 7M trunk | 0.714 | 0.702 | *"size alone does not fix paraphrase at 7M"* |
  | v11 | frozen `emit-1` | 0.712 | 0.709 | |
  | v11 | R1 7M | **0.828** (1,737/2,098) | 0.772 | gate unmet |
  | v11 | **lexical control** | **0.837** (1,756/2,098) | 0.767 | trunk *loses* to a word table |
  | v14 | R1 7M + 776 paraphrases | **0.836** (1,754/2,098) | 0.766 | gate unmet; dense combined head 0.949, word table 0.900 |
  | v23 | native 2.1M op model | 0.694 | 0.767 | Correct **0/313** |
  | v24 | native, no paraphrases | 0.398 | 0.627 | |
  | v25 | native + ~5,900 paraphrases | **0.872** (1,829/2,098) | 0.774 | still rising |

- **Verdict: `TOO TIGHT` on the *instrument*, `CLEAN` on the *measurement*.** The gate itself has never been
  met and the numbers are honest. But three structural facts undermine it as a *mechanism* verdict:
  1. **The lexical control beats the trunk at every measured point** (0.837 vs 0.828 at v11; a word table at
     0.900 vs the trunk head at 0.836 at v14). *"A useful control. Its accuracy does not prove the trunk has
     less information"* — true, but it does mean the gate is being applied to a classifier whose input the
     project has not shown to be sufficient.
  2. **`grounded-memory-evidence-2026-10-01.md:144-147` states the limit explicitly and the disposition did not
     absorb it:** *"The existing E3 failure therefore supports a narrow statement: this frozen
     feature/readout/training combination misses its gate. **A linear probe is not a mutual-information bound
     and does not establish a family-wide geometric capacity failure.**"* And: *"E3 training deliberately
     combines training phrasings with both training and development value pools … Thus its dev×dev score is a
     held-out-phrasing diagnostic, not evidence of unseen-value transfer."*
  3. **The supervision it was graded against is known-bad.** *"v14–v16 trained on the raw union, which carries
     audited label errors"*; the 40-row stratified audit found *"user/addressee reversal and
     hometown/current-home confusion."* The **v21 paired test** then showed reviewed labels did **not** help:
     relation 0.900 → 0.890, value span 1,493/1,503 → 1,423/1,503, session open relation **34/52 → 33/52**,
     closed **13/17 → 9/17**. Conclusion recorded: *"Label noise is not the binding cause, and halving the data
     costs value-span supervision."* That is a clean, matched test with a null result — rare and valuable here.
- **Status:** E3 gate unmet and **not weakened**; D19 §4 permits a labelled end-to-end diagnostic before
  reaching the old component gate. v25 is the live best (relation 0.872, still rising).

## 3.2 E4 log-sieve route — v12 → v16 → v17, and the "sparse-table closed-value extraction defect"

- **Records:** #1552 comments 06:09:18Z (v12), 08:04:03Z (v15), 08:06:43Z (v16), 08:16:14Z (v17);
  `docs/integration/grounded-memory-evidence-2026-10-01.md` §"V12 diagnostic addendum"; `STATUS.md`.
- **Measured:**

  | Development cell (`emit-1`) | v12 | v15 (fixed table, 776 paraphrases) | **v16 `route_acts=any`** | `recall=oracle` | Best trained read |
  |---|---:|---:|---:|---:|---:|
  | MQAR | 106/109 | 106/109 | **106/109** | 106/109 | 37/109 |
  | Open relation | 25/52 | 30/52 | **36/52 (0.69)** | **48/52** | 27/52 |
  | Closed relation | 2/17 | — | — | 16/17 | — |
  | Queries named | 63/83 | 77/83 | 77/83 | — | — |

- **The "sparse-table closed-value extraction defect", quoted from the source review
  (`grounded-memory-evidence-2026-10-01.md:56-64`):**
  > *"Its `RelationRoute::value` in `relation_compiler.rs` selects the latest prior user clause classified as
  > the query's relation and as assert or update. **It extracts values by removing every instrument-reserved
  > word; this does discard closed-relation values by construction.**"*
  And from #1552 06:09:18Z / `current-state.md:48-51`:
  > *"Closed values (2/17) are dropped by the extraction rule, so this is an **interface defect, not evidence
  > against exact memory**."*
- **Verdict: `INSTRUMENT-DEFECT`, correctly identified, and *not* used as a mechanism verdict.** This is one of
  the best-handled items in the catalogue: the project found the defect, named it, and explicitly refused to
  read it as evidence against exact memory. The defect was then *fixed*: the learned value-span head raises
  closed relation from **2/17 → 12/17**, and the integrated session reaches **13/17** (`#1552` 09:28:55Z and
  09:19:15Z comments). **A defect that was caught and repaired, not a retirement.**
- **v17 — the honest negative variant:** appending 576 standardized dense R1 trunk features to sparse word rows
  gave relation queries named **69/83 vs 77/83**, open relation **33/52 vs 36/52**, abstention **2/8 vs 4/8**
  (MQAR unchanged 106/109). Verdict as recorded: *"a recorded negative variant."* **`CLEAN`.** The document
  correctly notes *"Appending 576 standardized dense trunk features to sparse word rows, with a fit tuned for
  sparse words, is not equivalent to v14's dense combined head (0.949)"* — i.e. the negative is about **fit
  hyperparameters not being re-tuned for the new feature scale**, which the card's own framing concedes.
  *"No further route variant is run without new causal evidence (progress control)."* **This is a
  `TOO TIGHT`-adjacent case: a feature-augmentation arm was fitted with a learner tuned for the
  un-augmented features, and the resulting negative then closed the trunk-feature direction.**
- **Status:** v16 remains the best measured route; **not served under D11**; copy **0/33**; closed relations
  fail by construction in the E4 harness. The successor line (saved compiler in the real session) has
  superseded v16 for integration purposes.

## 3.3 The compiled relation classifier — v19 → v20 → v21 → v22, four recorded negatives

- **Records:** #1552 comments 09:28:55Z (v19), 10:18:09Z (v20), 16:32:18Z (v21), 17:45:09Z (v22);
  also `current-state.md:224-241`.
- **v20 — combined dense relation head (R1 trunk + words): a recorded negative.**
  Criterion, pre-declared in the v20 card: no gain ⇒ keep v19. Measured: relation naming **1,934/2,098
  (0.922)** vs table **0.900 (1,889)** — *better* — but **Correct exact 41/313 vs 272/313**,
  Assert 1,037/1,190 vs 960, Query 514/595 vs 559, Unresolved 1,399/1,566 vs 1,440. Session open relation
  **26/52 vs v19's 34/52**. *"The combined act head almost never chooses update over assert. Corrections then
  compile as bare asserts of a new value, which the store marks as conflicts, with no recall line."*
  Run then failed at save/load with `EIO` (SSD fault), so **v20 reload continuity was never measured.**
  - **Verdict: `CLEAN` on the pre-declared rule; `CONFOUNDED` as a statement about the trunk.** The arm changed
    **two** things — the relation head and the act head — and the *relation* head won (0.922 vs 0.900). The
    conclusion drawn was *"better relation naming did not survive the act regression."* Correct. But the
    follow-up (v22) then separated them, and that separation was itself declared a negative — see below.
- **v21 — reviewed vs raw paraphrase labels, paired.** Criterion pre-declared: no gain ⇒ stop. Measured:
  relation **0.890 vs 0.900**; value span exact **1,423/1,503 vs 1,493/1,503**; session open **33/52 vs 34/52**;
  closed **9/17 vs 13/17**; abstain 6/8 vs 7/8. The raw run reproduced v19 exactly at a different binary.
  - **Verdict: `CLEAN` and valuable.** A properly paired data-quality experiment with a null result. It also
    exposed **11 inconsistent duplicate marks in #1573's own review decisions** (fixed in a follow-up).
- **v22 — split compiler (combined relation head + table act).** Criterion pre-declared: *"Clearly above 34/52
  open: adopt it. Otherwise: record a negative."* Measured: relation **0.922** (better than 0.900), assert
  1,068/1,190 (better than 960), correct 266/313 (≈ v19's 272), query 514/595 (worse than 559); session open
  relation **32/52 vs v19's 34/52**; closed 13/17; abstain 5/8 vs 7/8. *"11 of its 20 misses have the statement
  stored correctly, but the query is named as another relation, so the read is absent. The combined head's gain
  on statements is lost on queries."* Decision: *"Further rebalancing of statement vs query heads would be
  classifier tuning on development results. **This line stops here.**"*
  - **Verdict: `TOO TIGHT`.** The card's own decision rule was **"clearly above 34/52"** — no margin was
    specified, and 32 vs 34 is **2 answers out of 52 (3.8 percentage points) on one development draw with no
    replicate**. The split design achieved *exactly* what it was built to achieve on the relation head
    (0.922 vs 0.900) and recovered the act head (correct 266/313 vs v20's 41/313), and it was closed by a
    two-answer deficit in the end-to-end session. The document's own diagnosis (*"the gain on statements is lost
    on queries"*) is a **concrete, actionable, localised defect** — a query-side relation head — and the
    decision rule converted it into a stop. The stated reason (*"would be classifier tuning on development
    results"*) is defensible process, but the effect size does not support it.
- **Status:** v19's table compiler stays integrated; `codex/compiler-combined` stays **unmerged** as the
  negative's record.

## 3.4 v23/v24/v25 — native op-model compiler: a negative that turned into the live most-promising line

- **Records:** #1552 19:09:40Z (v23), 20:16:58Z (v24), 22:29:55Z (v25).
- **v23 — the recorded negative.** Criterion pre-declared: *"Above v19's 34/52 open: the native op model becomes
  the compiler … Below: report where it fails."* Measured: fine-tune checkpoint development response NLL
  **7.18 → 0.0015** on training phrasings × development values (near-perfect memorisation); on development
  phrasings **relation 0.694, act 0.767, correct 0/313**; session open relation **19/52 vs v19's 34/52**;
  closed 7/17; abstain 3/8. Failure modes named: unseen correction cues never yield `update`; unseen relation
  words map to frequent relations; some unseen query wordings produce malformed ops. Reading recorded:
  *"The binding constraint is phrasing diversity and language knowledge, not the classifier or the memory
  path. This is a negative for this data scale."*
- **v24 — the data-slope measurement.** Templates only ⇒ relation **0.398** (836/2,098), act 0.627, session
  **15/52**. Adding the 776 paraphrases (≈5% more examples) ⇒ relation **0.694**, act 0.767, session **19/52**.
  Verdict recorded: *"wording diversity is the lever."*
- **v25 — scale-up.** ~5,900 more teacher paraphrases (21,410 training documents) ⇒ relation **0.872**
  (1,829/2,098), act 0.774, span exact 1,493/1,503; session open relation **24/52** (still below v19's 34/52),
  relation category 41/83, MQAR 15/109, **copy 0/33**. *"The data slope is still rising at the compiler
  (+0.18 relation for this addition). The end-to-end session does not yet beat the table compiler."*
- **Verdict: `CLEAN` as a negative *and* a model of how to convert one.** v23's negative was immediately
  followed by a *measurement of the slope* (v24), which identified the causal variable (wording diversity), and
  then a scale-up (v25) that recovered most of the deficit. This is the correct response to a negative and the
  contrast with 3.3 (v22) is instructive: **v22 and v23 were both "session below v19" negatives; v23's was
  investigated and v22's was closed.**
- **Status:** LIVE and rising. v25's next step is queued behind the prime-route R0 measurement; more teacher
  data is described as *"cheap (about 1 h of GPU)"*.

## 3.5 The language-continuation closeout — 0/5 prose (2026-09-26)

- **Records:** `docs/integration/language-continuation-result-2026-09-26.md` (+ `-plan`, `-direction`
  2026-09-26); evidence `docs/evidence/language-continuation-{result,continuous-quality,integer-quality,postprocess-6,review}-2026-09-26.json`; `current-state.md:961-973` and §§"Completed continuous and integer prose result".
- **Mechanism:** continue both retained step-8,348 parents to step **15,672** (+29,999,104 target visits per
  arm, 7,324 updates, B16/T256, full causal access) and evaluate frozen prose criteria.
- **Criterion, pre-registered:** **3/5 fully acceptable stories** and a **+2 improvement** over the same-path
  parent (0/5 baseline).
- **Measured:**
  - Continuous prose: **0/5 quaternion, 0/5 ordinary**; same-path parents 0/5.
  - Integer prose: **0/5 quaternion, 1/5 ordinary**.
  - Continuous comparison-tail Read NLL **1.996490474 / 1.974724189**; NoRead **2.492064889 / 2.472758698**;
    penalties **0.495574415 / 0.498034509** nats on 233,472 targets per arm.
  - *"Ordinary numerical/source-retention gates also fail; quaternion has source regressions against the
    accepted parent and an actual integer short-cycle."*
  - Execution was heroic and fault-tolerant: fit1 unsaved (charged loss), checkpoints at 10,480/10,488,
    12,707/12,715, 13,939/13,949, 14,000/13,949, then TARGET_COMPLETE at 15,672. Total saved learning exactly
    **7,324 updates / 29,999,104 target visits per arm from 8,348.**
- **Verdict: `CLEAN`, and correctly scoped.** Principal and independent reviewers agreed on the overall
  failure; dimension-level disagreements were *preserved* rather than resolved away; new characters and
  personification were explicitly allowed; the cap was explicitly not treated as automatic failure. Both
  candidates were refused promotion. The document also states the caveat that matters: *"Different
  continuous/integer sampling policies prevent attributing sample differences alone to conversion."*
- **Status:** **no candidate promoted**; the accepted parents remain. `project-track.md:253`: *"The September26
  exposure-only negative rules out an automatic repeat dose for that candidate; choose the next
  objective/data/representation/emission change from causal evidence."* The follow-on diagnostics
  (emission/selection, selection-policy, termination-weighting, geometric-read kernel) all ran and all closed
  negative — see Family 5.

---

# Family 4 — Geometry: tracking lanes, exact memory, codes, reads

## 4.1 B1 — finite-group tracking lanes: 2I lanes retired on a **pre-registered kill rule**, then the rule was withdrawn

- **Record:** `docs/integration/b1-finite-group-lanes-2026-09-27.md` (§8 closure 2026-09-28);
  `docs/integration/b1c-context-lanes-swap-stories-2026-09-28.md`; PRs #1442, #1447;
  `docs/evidence/b1-finite-group-lanes-2026-09-27.json`, `b1-closure-2026-09-28.json`.
- **Mechanism:** token-conditioned finite-group lanes `h_t = M[token] h_{t−1}` — quaternion (2I), ordinary
  reflection-pair, commutative phase, and frozen — trained on the A5 word problem and then mixed into text
  training (Stage B), with a compile-to-automaton serving path.
- **Criterion, verbatim and pre-registered (ROADMAP §4.1(b), before any run):**
  > **"Stage A gate:** 2I lanes are exact at 4,096 when snapped in ≥2 of 3 seeds, with commutative lanes at
  > chance."
  > **"B1 kill rule:** if 2I lanes fail to match the strongest non-diagonal control's tracking *at lower
  > serving cost*, retire the geometric-state claim from the serving path."
  Stage B gates (per seed): text NLL Δ ≤ 0.05 vs lane-free, and A5 accuracy at position 128 ≥ 0.99.
- **Measured — Stage A: PASS.** 17 of 18 seed×rate runs exact at length 4,096; commutative and frozen lanes at
  chance in 18 of 18; every exact run minimises to the **60-state A5 automaton**; quaternion lanes converge to
  2I with real parts within **0.0001–0.016** of exact values; reflection pairs converge to icosahedral
  rotations with trace signature **(1+φ)/4 = 0.6545**. Serving is one byte of state and two table reads per
  token, no multiplier and no float.
- **Measured — Stage B and closure.** Reflection-pair: ΔNLL **+0.036, −0.050, +0.009** (Stage B, 3 seeds;
  mean −0.002, inside 0.05) and **+0.0688, +0.0043, +0.0383** (fresh replication seeds 4–6). Over six seeds
  Δ = +0.036, −0.050, +0.009, +0.069, +0.004, +0.038 ⇒ **mean +0.018, one seed above 0.05.** Replication
  requires both gates in 3 of 3; it held in 2 of 3 ⇒ **FAIL**. *"no lane type is kept in the stack; finite-group
  state stays a Stage A (synthetic) result."* Quaternion lanes failed reliability in Stage B (seed 3 never
  learned A5, 0.039, cost +0.121 nats). The transformer control (run only at closure, at the stack's lr, its own
  optimum not searched) scored A5 0.008–0.027 at position 128, i.e. it does not track A5.
- **Verdict: `CLEAN` on the Stage A pass; `TOO TIGHT` / `UNDER-POWERED` on the closure.** The
  decisive number is that the **failure is the text-cost gate, not the tracking**: *"Tracking is reliable. A5
  accuracy is 1.000 in 6 of 6 reflection-pair seeds. … The failure is the text cost."* Over six seeds the Δ
  spread is **−0.050 to +0.069** (range 0.119 nats) with a mean of +0.018 against a 0.05 threshold — and the
  document itself reports that *"the lane-free stack's own seed spread is 0.063 (seeds 1–3) and 0.027 (seeds
  4–6)."* **The between-seed noise of the baseline is larger than the mean effect being adjudicated.** A
  3-of-3 per-seed rule applied to a quantity whose own seed spread exceeds the tolerance is a
  construction-guaranteed failure. Additionally, only **1–3 of 8 lanes** close into an automaton in these runs,
  so the mechanism is being tested in a configuration where 5–7 lanes are dead weight on the text loss.
- **The disposition contradiction the PI should note:** the B1 kill rule was pre-registered and was applied —
  and then **D12 (2026-09-28) withdrew it**: *"Kill rules attached to earlier gates are withdrawn, for example
  B1's 'retire the geometric state claim from the serving path.'"* B1 item 1 in the toolbox then re-lists 2I
  lanes as an **active candidate** to be *"trained jointly with the trained-in 2I transport (S4)"* — a
  configuration that has never been run. **So: retired on a clean pre-registered rule, un-retired on policy,
  never re-tested, and the strongest single measurement in the record (exact 4,096-length A5 tracking at 1.000
  with a byte of state) is still not in the model.**

## 4.2 D2 / AERM — exact relational memory: the store was perfect, the gate failed on margin

- **Records:** `docs/integration/d2-aerm-probe-plan-2026-09-28.md`,
  `docs/integration/d2-aerm-probe-result-2026-09-28.md`; `docs/evidence/d2-aerm-probe-2026-09-28.json`.
- **Frozen gate:** held-out Updated accuracy ≥ 0.90 **and a margin ≥ 0.30 over the equal-parameter dense
  control** in every seed; text NLL within 0.05.
- **Measured:**

  | Seed | Memory arm Updated | Control Updated | Margin (≥0.30) | Text NLL (≤0.05) |
  |---:|---:|---:|---:|---:|
  | 1 | **1.000** | 0.854 | 0.146 ✗ | −0.027 ✓ |
  | 2 | **1.000** | 0.878 | 0.122 ✗ | −0.039 ✓ |
  | 3 | **1.000** | 0.898 | 0.102 ✗ | −0.038 ✓ |

  The memory arm scored **1.000 on every class in distribution** — First, Updated, Updated-recency-trap
  (36/36), Reasserted, Previous, PreviousAbsent, Absent — and **32/32 free-running exact final answers in every
  seed**, with **0 misfires per 1,000** on 131,072 development story tokens. The dense control mostly answers
  with the latest mention (0.85–0.90 on Updated but 0.25–0.42 on recency traps and 0.00–0.07 on Absent).
  Held out: memory arm **0.000/0.000/0.015** Updated — but **with a gold parser, 1.000**.
- **Verdict: `TOO TIGHT` — the clearest case in the catalogue, and the project says so itself.** The gate
  demanded a **0.30 margin over a control that was itself scoring 0.85–0.90** on the gated class. A perfect
  scorer (1.000) therefore *cannot* pass: the maximum achievable margin was 0.146. **This gate was
  unsatisfiable by construction.** D12 cites this exact case as one of the five examples that motivated
  "gates promote, never kill": *"D2's exact memory scored 1.000 in distribution, then was marked FAIL because
  the margin gate missed against a strong control."*
- **Status:** reopened by D12 as an active candidate; the diagnosed next step is the **read** (G v1 below).

## 4.3 G v1 — always-on address-driven read: three seeds, gate fails, but the failure moved downstream

- **Record:** `docs/integration/g1-always-on-read-result-2026-09-28.md`; pre-registration
  [#973 comment 5876711862](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5876711862)
  posted **before any fit**; PR #1469.
- **Frozen gate:** *"held-out-template Updated accuracy ≥ 0.90 in every seed, with an equal-parameter control
  and development text NLL within 0.05."*
- **Measured (3 seeds × 2 arms, 12,347 s wall):**

  | Seed | Memory arm held-out Updated | Control | NLL Δ (≤0.05) |
  |---:|---:|---:|---:|
  | 1 | **0.1575** (43/273) | 0.4652 | −0.0162 |
  | 2 | **0.0733** (20/273) | 0.7729 | −0.0118 |
  | 3 | **0.0293** (8/273) | 0.8132 | −0.0335 |

  Reads did fire: `read_events` **2,294 / 2,574 / 2,706** (D2's trigger fired ~0). In distribution the memory
  arm was **perfect on every non-abstaining class** (138/138 First, 343/343 Updated, 26/26 Reasserted,
  159/159 Previous, 168/168 PreviousAbsent) and free-running **32/32** in every seed. Failure trace:
  `Unavailable` **460 / 484 / 519**, `NotSelected` 175/172/59, `Emission` 1/15/20, `WrongValue` absent.
  Tag accuracy 0.975/0.974/0.977.
- **Verdict: `CLEAN` on the run; the conclusion is well-localised and the mechanism was correctly *not*
  retired.** The documented reading is the important part: *"The dominant held-out failure is now
  `Unavailable`: … the surviving failures are on the **write/key side** — role-tag or address-key formation
  under unseen names and phrasings, not the read decision. The trace is coarse and does not separate a missed
  write from a wrong key from an eviction."* Under D12 this is recorded as *"not yet promoted at this scope"*,
  the exact store and G stay active, and a bounded diagnostic to split `Unavailable` is named. **Note the
  honest one:** *"The exact-store arm is more overfit to the training phrasings than the equal-parameter dense
  control on this world."*
- **Status:** active; next step recorded. **`complete`-answer generation was NOT_RUN** in G v1.

## 4.4 G-binding — masking the tag/trigger losses (2026-09-29): a big win blocked by a text-NLL guard

- **Records:** `current-state.md:428`; `docs/integration/g-binding-result-2026-09-29.md`;
  `docs/evidence/g-binding-2026-09-29.json`. Checkpoints for `prime-route-eval`:
  `opencode-g1-binding/checkpoints-{a,b}/aerm-s1`; `AermModel::from_stack` merged in #1491.
- **Measured:** Arm A (masking only, seed 1) lifts held-out-template Updated from **0.158 → 0.967**
  (**264/273**); tag accuracy 0.974 → 0.996; `Unavailable` **460 → 0**. **The text-NLL guard fails (+0.119
  nats)**, and name diversity (Arm B, 0.645) is a recorded negative.
- **Verdict: `TOO TIGHT` / ambiguous by construction.** A **single policy change** moved the failing metric
  from 0.158 to 0.967 — near-perfect — and simultaneously cost 0.119 nats of text NLL against a 0.05 guard.
  The document is explicit that *"acceptance of the masking is Lab 1's call."* Two things a PI should weigh:
  (i) `Unavailable` went to **zero**, which is precisely the failure mode G v1 had localised, so the mechanism
  did what the diagnosis predicted; (ii) the +0.119 text cost is on a **1.4M-parameter synthetic-world probe**
  whose text NLL is a guard, not a language result — and the trade is a 6× improvement on the gated capability
  for a 0.119-nat cost on a probe. Closing this on the guard alone would be the same pattern as D2's margin
  gate.
- **Status:** recorded as a negative component; *"acceptance of the masking is Lab 1's call"* — never resolved
  in the checked-in record.

## 4.5 D6 — what the 2I read representation discards (2026-09-28)

- **Record:** `docs/integration/d6-information-audit-result-2026-09-28.md`; PR #1464;
  `docs/evidence/d6-information-audit-2026-09-28.json`.
- **Setup:** evaluation only, no training. Parent #1438's kernel-off checkpoint (step 16,696, dot read,
  width 256, read width 64), anchor reproduced at 1.9847528. Population 48,412 selected read positions of
  232,560 (184,148 excluded by NoRead). Metric Δ = fraction of positions whose top-1 read event differs from
  the parent's own dot score.
- **Measured:**

  | Arm | Representation | Δ | Top-1 differs |
  |---|---|---:|---:|
  | O | dot/√read_width (anchor) | 0.000000 | 0 |
  | U | per-lane unit dot, unquantized | 0.413431 | 20,015 |
  | D | 2I direction only (snapped to nearest of 120 roots) | **0.464368** | 22,481 |
  | G | 2I direction + 3-bit dyadic gain (10 bits/lane) | 0.414938 | 20,088 |
  | K | per-lane raw k-means, 1,024 centroids (10 bits/lane) | **0.347331** | 16,815 |

  Outcome **ORDINARY-BETTER**, applied mechanically by pre-declared logic: Δ(D) ≥ 0.05 and Δ(U) ≥ Δ(D)/2;
  the gain arm does not restore it (Δ(G) = 0.4149 > 0.2322 and > Δ(K) + 0.01 = 0.3573).
- **Verdict: `CLEAN` as a ranking-fidelity measurement, correctly bounded.** Scope limits stated by the
  document: *"One parent, one population, one declared gain scheme and one k-means seed, evaluation only. It
  audits the native model's dot read, not the main-line stack's Lorentz read. **Δ measures ranking changes, the
  top-1 event. It says nothing about value error or answer quality.** Complete answers: **NOT_RUN.**"*
- **Important nuance the PI should not miss:** Δ(D) = 0.4644 is a *large* effect and it is **not** a
  demonstration that 2I loses information — arm U (unquantized per-lane unit dot, i.e. no 2I at all) already
  differs from the anchor at Δ = 0.4134. So **89% of the 2I "damage" is present without 2I.** The
  ordinary-better verdict is relative to K, and K's advantage over G is 0.068 Δ. The disposition is proportionate
  and explicitly non-veto (*"D6 is evidence, not a veto (owner, 15:27 UTC)"*).
- **Status:** 2I/E8 codes **reopened by D12** for *trained-in* use inside G. D6 tested only **post-hoc**
  substitution. **No trained-in 2I read code has ever been evaluated.**

## 4.6 Spherical-harmonic (Peter–Weyl) group kernel — a rare fully-negative mechanism line with a preserved mathematical positive

- **Record:** `docs/integration/EVIDENCE.md` rows dated 2026-09-19
  (`docs/evidence/native_geometric_spherical_harmonic_kernel_2026-09-19.txt`).
- **The mathematical positive that survives:** 2I has **9 conjugacy classes `[1,1,12,12,12,12,20,20,30]`**,
  computed from the verified Cayley table with conjugation-invariance checked over all 14,400 pairs — *"the
  harmonic band dimension, so a class-function kernel is 9 ternary weights rather than 120."* The graded read
  `y = Σ_g w[class(q⁻¹g)]·S[g]` uses ternary weights and adds/subtracts only. **A hand-set class-function
  kernel recovers a corrupted query 0.13 → 0.23 (≈1.8×) at a clean cost 0.55 → 0.15.**
- **The negatives, all recorded with their reasons:**
  1. *"Learning the harmonic filter: attempted, negative, reverted."* Trained on clean addresses only, the
     filter spreads: fixed `[0,1,0,…]` clean **0.42** / corrupted 0.13 vs learned `[1,1,1,0,…]` clean **0.14** /
     corrupted **0.14**. *"Reason: robustness is not in the objective — the optimiser adds filter mass only
     while it helps the clean loss, and spreading never does."* The change regressed five tests, so it was
     **reverted**; the diagnosis was retained, the code was not.
  2. *"Corruption in the objective: diagnosis confirmed; the learned filter still is not worth it."* With
     `corrupt_frac` in training the filter does change (3 → 7 classes) and corrupted accuracy rises 0.12 → 0.13
     while clean rises 0.11 → 0.27 — **but the fixed exact filter reaches clean 0.42 on the same budget.**
  3. *"General group-algebra filter: no gain — the filter is not the bottleneck."* A **13× capacity increase
     changes nothing: class 0.11 (3/9 slots used) vs general 0.11 (5/120 used).**
  4. *"Four readout levers ruled out."* serving readout ternary 0.28 vs unquantised 0.28; 900 vs 4,000 steps
     0.28; dv 64 vs 256 0.28; filter 9 vs 120 slots 0.11.
  5. *"the nearest-prototype decode is the fifth flat lever"* — the reduced form `argmax_r(2·num·p_r − ‖p_r‖²)`
     is linear-plus-bias; the bias test gave **0.28 vs 0.28**.
- **Verdict: `CLEAN`, and this is the best-executed falsification sequence in the entire record.** Five
  independent levers tested and ruled out with a stated reason each; the conclusion is bounded exactly
  (*"the ceiling is structural in the readout mechanism — one linear map over one read vector decoding 120
  classes"*); the code was **reverted** rather than left in place; and the mathematical result (9 classes ⇒ a
  9-weight class-function kernel) was **retained**. `repo-review-direction-2026-09-23.md` §3.1 summarises the
  family as *"class filter 0.13→0.23 corrupted | learned filter reverted; capacity immaterial; sieve parked |
  partial/parked."*
- **Status:** `repo-review-direction-2026-09-23.md` §10: *"**Park (conditional, unchanged):** S7/E8/icosian
  relationship state, harmonic coefficient banks, resonance softmax replacement, quantum-spin/relative-phase
  semantics — **all remain NOT_RUN pending a witnessed structural need against an ordinary control.**"*
  See Family 6.

---

# Family 5 — Objective, emission and read-kernel negatives (2026-09-26/27)

These four are the complete set of "what to change next" experiments run after the language-continuation
closeout. **All four closed negative, and the branch was then closed with them.**

| # | Experiment | Pre-registered criterion | Measured | Verdict |
|---|---|---|---|---|
| 5.1 | **Emission/selection diagnostic** (read-only, same step-15,672 checkpoints) | predeclared plan `emission-selection-diagnostic-plan-2026-09-27.md`; reproduce the retained packet exactly first | `PARITY_EXACT`, 0 mismatches, max float delta 0.0, all five stories both arms. **8 of 12 witnessed decisions are low-probability draws** that departed from a materially better-ranked alternative (2 near-ties, 2 top choices, 0 copy-dominated). Malformed tokens were **vocabulary-side, not copy-side.** | `CLEAN` — the instrument reproduced the parent exactly. Supported the next step (selection policy), which then also failed. |
| 5.2 | **Selection-policy diagnostic** (greedy replay, same checkpoints) | predeclared `selection-policy-diagnostic-plan-2026-09-27.md`; state the frozen acceptability bar before replay | Source panel `PARITY_EXACT` (21/32 quaternion, 23/32 ordinary). All five greedy trajectories diverge within 3 tokens so the sampled malformed clauses do not recur — **yet greedy prose stays below the frozen bar: quaternion 2/5, ordinary 0/5**, and the greedy source panel still fails **11/32 and 9/32** rows (lost vs parent: 9 and 6). Dominant modes: correct-noun-then-extra-phrase 13/20; wrong first noun 7/20. | `CLEAN`. Independently audited with identical verdicts. Also carries a **self-correction**: the plan's `5/2`/`7/4` citation was the new hard artifact's row, not this panel's (`9/2`, `6/5`). |
| 5.3 | **Termination-weighted objective** (`end_weight=2.5` on sentence-final targets) | predeclared `termination-objective-plan-2026-09-27.md`; INERT band recorded as treatment-minus-control | 4 sequential fits, 1,024 updates / 4,194,304 targets each. Treatment uniquely resolves **3** source rows (2 non-termination + 1 wrong-noun) with **0 unique losses**, but the **plain dose alone recovers 12 of 20 failing rows**; sampled prose stays **0/5 in all four finals**; standard dev NLL slightly worse for the treatment. Guardrails hold. | `CLEAN` — but note this is the pattern where **the control moved more than the treatment** (12 vs 3), so the experiment is really a null on the *comparison*, not a harm. `termination-objective-review-2026-09-27.md` classifies it **INERT: no automatic extension**. A separate **source-verified normalization/reporting defect** was found at `c54d7801` (shard-normalized weighted losses combined by equal batch mass; training curve records weighted loss as `batch_mean_nll`). |
| 5.4 | **Finite geometric read kernel** (learned score over the signed relative element of 2I; 16 unit-coded lanes, exact composition table, STE, per-lane 4→8→1) | predeclared `geometric-read-plan-2026-09-27.md`; HARM if it worsens the comparison-tail Read NLL or trips a guardrail (1.5× cost gate) | **HARM.** Read NLL **1.984753 → 2.004471 (+0.019719)**; complete source answers **27/32 → 17/32** (ten unique losses, zero unique gains); sampled prose **0/5 both arms**; the predeclared **1.5× cost gate failed at 1.84×** paired same-session. Hard-path usage non-collapsed (120/120 codes per lane, 99.25% non-identity relations). | **`TOO TIGHT` on the cost gate; `CONFOUNDED` on the harm.** The plan's own heading makes *a guardrail failure itself a HARM condition* — so a **1.84× vs 1.5× timing ratio became part of a mechanism-harm verdict.** The document discloses the two things that matter: *"The harm is measured for this parameterization including its **784 added per-lane parameters**; the **capacity/normalization attribution control is unrun**"* and *"Conditional attribution arm C is **deferred**."* **Arm C is the arm that would have separated "the geometric score is bad" from "784 extra parameters plus an unnormalized added score is bad." It was deferred on budget (decision-time balance 712,153,247 ms; projected ~739M against a 735M stop margin and a 744M ceiling) and never run.** |

- **Read-side localization (5.5), the constructive one.** `read-localization-plan-2026-09-27.md` /
  `read-localization-result-2026-09-27.md`: decision-0 read-mass attribution on the 32-row source panel under
  five declared conditions, `PARITY_EXACT` on all 32 rows, 1.29 s. Pre-declared rule returned **MIXED** (the
  frozen `READ_ACCESS_LIMITED` thresholds were not met: the entity was present at read rank 2–3 with 16–32%
  share and the entity-share metric did not separate the entity mention from its matched control). Post hoc at
  the same scope, a clean **READ_RANKING** signature: in **5/5** distractor rows the emitted token equals the
  read top-1 (` clouds`); the near-query entity mention completes **5/5** while the matched non-entity control
  completes **0/5** and emits the inserted noun (` carrot`) 5/5; the extra-phrase class is **not**
  entity-specific (D 3/5 vs control 4/5). Recommended successor: an **oracle read re-rank intervention**
  (clamp decision-0 mass onto the entity occurrence) *"to confirm the ranking as the sole bottleneck before any
  mechanism change."*
- **Status of 5.5: the recommended oracle re-rank intervention was never run.** The next mechanism change was
  made anyway (the radial reader comparison, D1 attribution, then `rra`-pattern A1, then the whole D18/D19
  pivot). This is a `POLICY-KILLED` diagnostic: a cheap, pre-specified, single-bottleneck test that would have
  decided between "ranking" and "state" was named and skipped.

---

# Family 6 — Geometry families demoted, "conditional", or never actually tested

This section answers the owner's specific questions directly.

## 6.1 Hopf / S3 fiber, S7, E8/icosian, harmonics — "conditional", and **never tested**

- **The demotion statement (2026-09-23), `docs/integration/repo-review-direction-2026-09-23.md` §3.1:**

  | Family | Best positive (scope) | Strongest negative (scope) | Status |
  |---|---|---|---|
  | Hopf / S3 fiber | representation with pole convention (tolerances) | **no measured language benefit; Q30 uses int mul/div** | **conditional, NOT_RUN for language** |
  | S7 / E8 / icosian | **none measured** | "geometry does not supply free storage or advantage" | **hypothesis, NOT_RUN** |
  | Harmonics / resonance | class filter 0.13→0.23 corrupted | learned filter reverted; capacity immaterial; sieve parked | **partial/parked** |
  | Spin / chirality | exact sign/inverse bookkeeping | **no semantic axis measured** | conditional |

- **§10 verbatim:** *"**Park (conditional, unchanged):** S7/E8/icosian relationship state, harmonic
  coefficient banks, resonance softmax replacement, quantum-spin/relative-phase semantics — all remain NOT_RUN
  pending a witnessed structural need against an ordinary control."*
- **They were re-affirmed as conditional at least six times** and never promoted or tested:
  `read-conditioned-review-2026-09-21.md:52` (*"remain tools for a witnessed capacity/interference or
  structural-role need"*); `policy-objective-review-2026-09-21.md:89` (*"conditional representation choices, not
  automatic extra memory or orthogonal syntax"*); `reader-confidence-review-2026-09-21.md:72`; `EVIDENCE.md:34`,
  `:54`, `:71`, `:94`, `:128`; `model-direction-2026-09.md:38`; `project-track.md:806`, `:810`;
  `architecture-2026-09/README.md:47` (*"the old Hopf/prime MoE experiments really do dispatch to expert FFNs
  and remain outside the current design; later reconsideration is conditional"*).
- **The mathematics was also corrected, several times, against the owner's framing** —
  `research-leader-handoff-2026-09-23.md:90`: *"the quaternionic Hopf fibration is S3 → S7 → S4, not a
  reversible universal S3 → S2 → S1 → S3 chain. S7 unit vectors are 8 real coordinates, while a mutable
  harmonic **field** needs stored coefficients and a read/write law. **Orthogonality under an integral does not
  make a single sampled wave crosstalk-free; E8's 240 roots are not 240 orthogonal registers.**"* And
  `structural-memory-hopf-direction-2026-09-20.md:76`: *"`G2/SU(3) = S6`, while `Spin(7)/G2 = S7`"* — a
  correction to a supplied summary.
- **Verdict: `POLICY-KILLED` / NOT-RUN, and this is the single largest untested area in the project.** Nothing
  in this family has ever been measured against an ordinary control. The status labels are honest
  (*"NOT_RUN"*, *"hypothesis"*, *"conditional"*), but a PI should register that **the project's headline
  geometry (S7, E8/icosian relationship state, harmonic banks) has zero experimental content** — not a negative
  result, an absent one. D16 item 3 does define what a fair test would be and it has never been run:
  *"Harmonic attention is compared with Taylor-2 at equal feature count, and it enters serving only on a
  measured win over a byte-matched window. The owner's Lie-group case is tested with an arm that can differ from
  Taylor-2: a RoPE-plane-aligned or SU(2)/Wigner-D basis."*

## 6.2 "Terminal D5 sparsity" — **never tested; a design constraint masquerading as a result**

- **What D5 actually says** (`DECISIONS.md`): per-token parameter sparsity is the **terminal serving invariant**;
  the current served artifact is *"D0-b-compliant and D5-non-compliant (**611,814 inspections/step**); this is
  recorded, not retroactively banned."* *"This also converts Goal R into a cheap, fair, measurable contest
  (geometric routers — prime/zeta/H4/VSA — versus ordinary LSH/learned-kNN at matched access budget, judged by
  top-K decision-equivalence), **which has never been run**."*
- **Every mention of terminal D5 sparsity in the record is a disclaimer, not a measurement.** Exhaustive list
  of the phrasings found:
  - `language-continuation-direction-2026-09-26.md`, `precision-factorial-result-2026-09-25.md:122`, `:188`,
    `learned-rounding-result-2026-09-25.md`, `joint-recurrent-result-2026-09-25.md:41`,
    `quantized-recurrent-result-2026-09-25.md:184`, `integer-execution-result-2026-09-25.md`,
    `bounded-admission-result-2026-09-25.md`, `current-state.md:1073`, `:1127`, `:1193`, `:1421`, `:1631`,
    `:1784` — all of the form *"no … terminal D5 parameter sparsity … claim is promoted"* / *"terminal D5
    parameter sparsity remains open"* / *"D5 sparsity … unqualified."*
  - `geometric-toolbox-2026-09-28.md` §C lists D5 parameter sparsity only as a **future use** of the E8/H4
    codebook entry.
  - D18 §7: *"It needs a D5 selected-weight-access mechanism, and **no such mechanism is built.**"*
  - D18 §9: the D5 design memo is *"one-day, no-compute … **Nothing is run**."*
  - `current-state.md:64`: *"D5 selected access … not established."*
- **Verdict: `POLICY-KILLED` — and it is a *whole-goal* gap, not a mechanism retirement.** The terminal
  invariant has been carried for the entire programme as a constraint on every result and has **never had a
  single experiment run against it**. There is therefore no negative result to catalogue here: the honest
  entry is "no measurement exists." The PI's question "was this ever really tested?" is answered **no**, for
  terminal D5 sparsity.

## 6.3 "Hamiltonian dynamics" — **never tested; three distinct meanings were conflated and then separated**

- **The separation, `principal-attention-mathematics-2026-09-24.md` §"Three meanings of 'Hamiltonian'":**
  1. **Hamilton product** — finite quaternion/2I transport, a signed permutation / table lookup;
  2. **A discrete statistical-mechanical Hamiltonian** — a scalar `H_θ(a; q_t, M_t)` over a finite
     configuration, mutable through state. *"Such a scalar is a proper operational Hamiltonian/energy function,
     but **has no automatic conservation or symplectic theorem**."*
  3. **Hamiltonian flow** — `ż = J∇H(z)`, symplectic, conserving. *"This requires a phase space and
     differentiable or otherwise rigorously specified Poisson/symplectic dynamics. **Neither quaternion
     multiplication nor a scalar ranking table supplies that structure.**"*
  §36: *"use 'Hamilton product' for the finite transport and 'discrete mutable energy' for read/write
  competition. **Reserve 'Hamiltonian flow' for an explicitly built and tested dynamical subsystem.**"*
- **What exists in code:** `learner/hamilton_transport.rs:1–37` is *"an exact Q8 signed permutation on four
  integer coordinates with inverse/relative action, **not a fitted general vector Hamiltonian**"*
  (`principal-attention-engineering-2026-09-24.md:20`). And: *"`hamilton_transport.rs:5-41` implements Q8
  signed-permutation actions and a relative operation, but **neither an address learner nor a learned
  Hamiltonian dynamics law**"* (`principal-attention-computer-science-2026-09-24.md:15`).
- **Architectural reason it was dropped (a considered no-go, not a failure):**
  `geometric-attention-2026-09-26.md:55`: *"A conservative (Hamiltonian, norm-preserving) memory cannot forget.
  The review's no-go result (§4 of the review) says **every forgetting lane must round and dissipate.**
  Retrieval can be Boltzmann-weighted; the state dynamics must be dissipative."* And
  `principal-attention-computer-science-2026-09-24.md:24`: *"a continuous symplectic Hamiltonian model usually
  needs numerical integration and conserves information, so **it cannot by itself implement irreversible
  overwrite/forgetting**."*
- **Verdict: `POLICY-KILLED` by design argument, never tested.** Every mention of Hamiltonian *dynamics* in the
  record is a disclaimer (`integrated-attention-a1-result:109` — *"not … a physical Hamiltonian flow"*;
  `joint-recurrent-result:41`; `learned-rounding-result:162`; `bounded-admission-plan:144`;
  `precision-factorial-result:122`, `:188`; `fourth-lab-geometric-attention-2026-09-26.md:124`;
  `kvar-recall-result-2026-09-24.md:74` — *"**`(h)` the geometry gate is `NOT_RUN`** — the owner's 'vectors +
  Hamiltonians' hypothesis is therefore **not** …"*). **Answering the owner's question directly: Hamiltonian
  dynamics was never tested, and there is a written mathematical reason (a conservative flow cannot forget)
  why it was not. That reason is sound. But note the asymmetry: the *discrete mutable energy* reading (meaning
  2) is exactly what the read kernels implement, and it was tested — with mixed results (5.4 HARM, 5.5
  READ_RANKING).**
- **`kvar-recall-result-2026-09-24.md:74` is the cleanest record that the owner's supplied "vectors +
  Hamiltonians" lead was never given its own experiment:** *"`(h)` the geometry gate is `NOT_RUN`."*

---

# Family 7 — Cross-cutting findings the owner specifically asked about

## 7.1 "Removing the individual-history row improves aggregate loss" (2026-09-20)

- **Records:** `docs/integration/occurrence-reader-review-2026-09-20.md` (especially lines 11–24);
  `docs/integration/query-read-review-2026-09-20.md` (especially §"The numerical result survives, but its
  attribution changes"); `docs/integration/EVIDENCE.md:165`; original results
  `docs/integration/s-attribution-result-2026-09-20.md` +
  `docs/integration/query-read-result-2026-09-20.md`; PR #1312 (merged `0bca39b5`), PR #1310 (merged
  `7ea3744c`).
- **The exact finding, verbatim:** *"**Removing the individual-history contribution improves aggregate loss on
  both panels**; this supports changing the mechanism, **not a theorem that older content is absent**"*
  (`EVIDENCE.md:165`).
- **The underlying numbers:**

  | Frozen condition | Old panel, bits/target | New positions, bits/target |
  |---|---:|---:|
  | E (local only) | 7.170816 | 7.048239 |
  | S11: E + history row + query row | 7.032228 | 6.919775 |
  | **Qonly: E + query row** | **7.022581** | **6.912660** |
  | Honly: E + history row | 7.171048 | 7.045576 |

  The Qonly − S11 difference on matched older-present targets is **−0.009978 bits (old panel)** and
  **−0.007348 (new panel)**, with paired intervals excluding zero; the new interval
  **[−0.012050, −0.002327]** is *not* contained in the declared ±0.01 equivalence band. M01 (an offline
  fit-average history comparator) beats S11 by 0.023622 and 0.018695 bits.
- **Verdict: `UNDER-POWERED` + `CONFOUNDED` + `INSTRUMENT-DEFECT`. Three independent problems, all
  documented by the project itself:**
  1. **The effect is below the project's own equivalence margin.** D2 sets ε = 0.01 BPB and states *"anything
     below it cannot be decisive for the competitive question"* and *"failing the magnitude test is not
     evidence that it is a dead end."* The primary effect here is **0.007 bits/target (new panel)** — below ε.
     Honly vs E is **7.045576 vs 7.048239 = 0.0027 bits**, i.e. the history row alone is worth 0.04% and is
     *worse* than no history at all. Using this to change memory mechanism is exactly what D2 forbids.
  2. **S11 and Qonly are not an ablation of the history row.** The document says it: *"the fitted architectures
     differ in more than the presence of individual history."* The clean ablation is Honly (E + history row)
     versus E, and that difference is 0.0027 bits.
  3. **The comparator that "wins" has a documented population defect.** M01 *"includes terminal states outside
     the training-target population"*: lines 869–879 iterate `2..w.len()` while the shared target iterator
     predicts only through `n−2`. Saved counts sum to 253,017 against 248,921 older-present targets; **34 of the
     4,096 extras affect the evaluated lengths 1–61.** *"This is a scoped estimator-population deviation, not
     target leakage."* Correct — but it is a defect in the comparator whose 0.0236-bit win is used as
     corroboration for the *architectural* claim.
  4. **The counter-evidence was already in the file and had been misread.** The decisive datum was sitting in
     `query-read-2/result.json`: *"**S older-prefix donor penalty = −0.002298261 bits per eligible target,
     interval [−0.014071554, +0.008581473].** There are 1,177 eligible observations in 36 documents and 564
     changed S reads. … This detects **no** predictive older-content benefit on its supported subset. The
     result's sentence that the run showed S benefited from older content is unsupported."* **The exact-tail
     donor test is the correct instrument for this question, it was run, and its interval spans zero (and is
     underpowered at n=1,177).** The review's conclusion — *"no demonstrated net predictive benefit from this
     frozen individual-history branch at this training dose and support"* — is properly hedged, and does say
     *"It does not prove that the state contains no information, that all subpopulations are harmed, or that
     geometric memory is intrinsically ineffective."*
- **Status: the hedge did not survive.** The architectural conclusion drawn from this pair of reviews —
  *"The justified architectural statement is no demonstrated net predictive benefit … **That is sufficient to
  move toward a different memory mechanism**"* — became, downstream, the D8 retirement of the A1–A4 selector
  line and the D18 outcome-D pivot. **A sub-ε effect with a defective comparator, on one training dose, on a
  reused open-development panel, is load-bearing for two programme-level decisions.**

## 7.2 "Adding a direct two-token channel worsens it" (2026-09-23)

- **Records:** `docs/integration/ordinary-lexical-audit-2026-09-23.md` §4 (the exact table is at lines
  118–126); `docs/integration/resource-ledger-2026-09-19.md:2322` (the sign-mislabel correction);
  `project-track.md:757`.
- **Setup, verbatim:** *"One ordinary learner (the retained `TlModel`/`TlTrainer`), one split, one schedule, one
  seed, one architecture; the **only** change between arms is the bounded local channel handed to the readout
  (none, versus the last two tokens with the retained fixed order rotation). Both arms train on the same
  full-context task and are scored on the same 5,376 development targets (**2,000 steps, batch 224, lr 0.02,
  seed 13** — matching the delivered run's step count and targets-per-step). Sealed root `olx-channel-1`."*

  | Arm (same data, steps, batch, seed, architecture) | dev bits/target | gain vs count |
  |---|---:|---:|
  | tuned interpolated `(prev,cur)` count reference | 5.1217 | — |
  | delivered artifact (window objective, prefix recurrence) | 7.3722 | −2.2505 |
  | **recurrence-only** (full-context per-position objective) | **6.2039** | −1.0822 [−1.2822, −0.8957] |
  | **recurrence + direct two-token channel** | 6.6732 | −1.5515 [−1.7519, −1.3613] |

- **Measured effect:** the channel makes development loss **worse by 0.469 bits** (CI `[+0.410, +0.527]`,
  excludes zero). Both arms remain behind the count reference.
- **Verdict: `CLEAN` on the number, `TOO TIGHT` on the inference, and the document concedes it:** *"On this
  matched evidence the principal's 'give the shared path a direct bounded last-two-token channel' hypothesis is
  **not supported as a sufficient remedy in this training configuration**. **The channel may be redundant or
  harder to optimise; this comparison does not distinguish those causes or retire the mechanism family.**"*
  Two additional caveats the PI should carry:
  - **Disclosed non-equivalence:** *"this per-position formulation feeds the preceding tokens with the
    `OBSERVE` event throughout, whereas the delivered window objective feeds them `OBSERVE` for the frozen
    prefix and `GENERATE` afterwards; the two arms are self-consistent and matched to each other, but their
    absolute values are not a like-for-like replay of the delivered path."*
  - **The receipt's sign was wrong and had to be corrected in a new audit root** —
    `resource-ledger-2026-09-19.md:2322`: *"its saved negative value is recurrence-only minus two-token, while
    the key says two-token minus recurrence-only."* The correction is documented and preserves the sealed
    receipt; it does not change the direction, but it is a second instrument defect on the same experiment.
  - **The much larger effect in that table is not the channel at all:** recurrence-only recovers **1.17
    bits/target without any geometry** by changing the training formulation. The document's own conclusion is
    *"**Do not add geometry yet.** Repair the shared path's training formulation first."* That reformulation
    (`OBSERVE`/`GENERATE` handling) is the actual finding, and the two-token channel is a side negative.

## 7.3 The count-prior blend (2026-09-23) — a scoped negative that was correctly scoped

- **Records:** `docs/integration/ordinary-lexical-count-blend-result-2026-09-23.md`;
  `project-track.md:723`; result root `olx-count-blend-1`.
- **Measured:** the exact criterion is satisfied (`mean_kl 2.115408 > gap 1.562381`) so an *in-sample* λ can beat
  C, but the **tune-frozen** pool is **+0.004928 [−0.026228, +0.040527]** against C on the full development
  split; the in-sample oracle gain is only **−0.025181**; the one-token control collapses to C exactly; and the
  pool is **≈+0.552 bits/target worse on the 422 rare targets outside the top-1024**.
- **Verdict: `CLEAN`.** The conclusion is properly narrow — *"No material, tune-robust complementarity; **do not
  build an exact addressed local read inside the served readout on this evidence**"* — and it is exactly the
  scope of the measurement: a frozen-hyperparameter blend, not addressed memory. Note that this is the *only*
  place in the record where a "do not build X" conclusion is drawn from a tune-frozen (rather than
  cherry-picked) hyperparameter, and it is the correct pattern.

## 7.4 Joint fit / training-budget negative (2026-09-23)

- **Records:** `docs/integration/ordinary-lexical-joint-fit-result-2026-09-23.md`; `model-direction-2026-09.md:24`.
- **Measured:** doubling steps 2,000 → 4,000 changed served development loss from 6.684105 to **6.708963**,
  **+0.0249 [−0.0422, +0.0967]** — interval includes zero, probe oscillating rather than descending. The two
  cheapest output-side levers recovered −0.1313 bits (in-class readout refit) and nothing (longer schedule).
- **Verdict: `CLEAN` and structurally important.** *"the model has converged operationally and training budget
  is not the constraint"* — leading to the diagnosis that the binding constraint is the **interface**: a linear
  readout of a 64-dimensional state (effective per-position rank ≤ 64 for ordinary prose). This is the strongest
  causal diagnosis in the record, and it is what produced the long-range-probe objective (D6) and ultimately
  the retrieval question. **A related correction is recorded**: the earlier claim that the state does not retain
  `prev` was wrong — a transport-aware decode recovers the immediately preceding token at **97.84%**, the one
  before at **74.86%**, then 18.69% and 0.60% at lags 3 and 4 (*"the state retains roughly two-to-three tokens
  of decaying second-order context"*).

---

# Part 8 — Items killed or parked by process rather than by measurement

Consolidated, because the owner's question is specifically about premature retirement. Each of these was stopped
by a rule, a budget, or a scheduling decision, **not** by a measurement that failed.

| # | Item | Stop mechanism | What measurement exists | Cheapest decisive test that was never run |
|---|---|---|---|---|
| 8.1 | **`recent64` trained access policy** | D9 withdrew the recommendation *before execution* | Same-weights recent64 recovered **27/25** complete answers, **lower NLL than either tested indexed proposer** (2.119169 / 2.108628 vs orthant64's 2.129595 / 2.124890) | Train 256-token context with recent64 access and re-apply the orthant64 gate. D18 §6's re-entry condition is now satisfied (outcome D occurred). |
| 8.2 | **Lorentz-vs-Dot read parity, third seed** | D18 outcome D forbade new training | 2 matched pairs; exactly one metric over tolerance in each ⇒ third seed required by a rule pre-declared at 01:16:51Z | One paired seed (`arm-L-s3` + `arm-C-s3`). The arms and metrics were fully specified before observation. |
| 8.3 | **Lorentz-scored pointer (the specified `P+ptr`)** | Never implemented; dot pointer used in all runs | None | Replace the pointer score function. This is the arm D18 §2 actually named. |
| 8.4 | **Transformer + pointer control** | Not in the A1 design | T (no pointer) = 0.009 MQAR; every pointer arm 0.28–0.44, geometric or not | Add a pointer head to T. This is the control that decides whether "no learned retrieval at this scale" is true. |
| 8.5 | **Geometric-read kernel, attribution arm C** | Budget/ledger ceiling (projected ~739M vs a 735M stop margin) | HARM measured, but *"the capacity/normalization attribution control is unrun"* | Arm C: same 784 added per-lane parameters with a non-geometric/random relative code. Without it, the HARM verdict cannot be attributed to geometry. |
| 8.6 | **Oracle read re-rank intervention** | Named as the recommended successor in `read-localization-result-2026-09-27.md`; never scheduled | READ_RANKING signature is clean: 5/5 distractor rows emit read top-1; entity mention completes 5/5 vs control 0/5 | Clamp decision-0 mass onto the entity occurrence on frozen weights. Cheap, read-only, decides ranking-vs-state. |
| 8.7 | **E8/Hopf/S7/harmonics** | *"conditionally parked … pending a witnessed structural need"* (×6) | **Zero measurements.** Every status is NOT_RUN | The D16 §3 design (harmonic vs Taylor-2 at equal feature count; SU(2)/Wigner-D arm) is fully specified and unrun. |
| 8.8 | **Terminal D5 parameter sparsity** | Carried as a constraint; no mechanism ever built | **Zero measurements.** D18 §7: *"no such mechanism is built"* | The D5 design contest itself: geometric router vs ordinary LSH/learned-kNN at matched access budget, judged by top-K decision-equivalence. D18 §9 memo exists; nothing ran. |
| 8.9 | **Hamiltonian flow** | Written no-go (conservative dynamics cannot forget) | **Zero measurements.** `(h)` geometry gate explicitly NOT_RUN | N/A — the no-go argument is sound. Catalogue as *deliberately untested*, not *failed*. |
| 8.10 | **B3 / E8 lattice weight coding** | Kill criterion triggered, then Track B halted | Kill triggered on a **disputed instrument** (float reference 9.45 nats/token, consistent with token-ID mismatch); scalar control also collapses (+0.728 nats at 3 bpw) | The required *"one-matrix exhaustive-encoder check"* (D18 §6) with a float-NLL validity band. |
| 8.11 | **QAT on the S2 dialogue stack** | Gate missed at ≥29/58 float agreement | QAT 6/58 vs a float control that **itself drifted to 7/58**. *"QAT integer vs own unquantized float weights was not evaluated and remains missing."* | The missing measurement: QAT integer vs its own unquantized float weights. |
| 8.12 | **v22 split compiler** | *"This line stops here"* on 32/52 vs a 34/52 rule with no margin specified | Relation head **0.922 vs 0.900** (better), act head recovered (correct 266/313 vs v20's 41/313), 11 misses localised to query-side naming | A query-side relation head, or two more draws on the session cell. This is a 2-answer deficit on one draw. |
| 8.13 | **v17 trunk-feature route** | *"No further route variant is run without new causal evidence"* | 33/52 vs 36/52, but *"a fit tuned for sparse words"* was used on an augmented feature scale | Re-fit with feature scaling appropriate to 576 dense features. The negative is currently about the fit, not the features. |
| 8.14 | **A1–A4 selector line (D8)** | *"Stop the succession of local-credit selector adjustments"* | A1–A4 are genuine negatives at 46k–1.1M tokens with 4 authored correction cases; A1's failure is exact code collapse; A4's failure is export retention | Not a "cheaper test" — but note the retirement was taken on a *succession of fixes* rather than on the mechanism, and A1's own review explicitly warns *"Do not turn this finding into a new sequence of isolated pass/fail fixture gates."* |

---

# Part 9 — Synthesis: where I believe a rejection was UNJUSTIFIED or CONFOUNDED

Ranked by how much programme direction each one is currently carrying.

### Tier 1 — rejection should be re-opened before further mechanism change

1. **D2/AERM's margin gate (4.2) — the gate was unsatisfiable.** Memory arm 1.000 in every seed, every class;
   the control scored 0.854–0.898 on the gated class, so the maximum attainable margin was 0.146 against a
   required 0.30. A perfect mechanism cannot pass. D12 already recognised this; **what has not happened is the
   re-measurement** with a well-posed criterion (e.g. abstention behaviour and recency-trap accuracy, where the
   arm scored 1.000 vs the control's 0.00–0.07 and 0.25–0.42).

2. **`recent64` (2.2, 8.1) — withdrawn before execution on evidence that was favourable.** The only existing
   measurement (same-weights) shows recent64 as the best access policy tested, beating both trained bounded
   proposers on NLL and recovering 27/25 complete answers. It was withdrawn by D9 as part of a loop-prevention
   correction. The re-entry condition in D18 §6 is now met. **This is the single cheapest untested experiment in
   the project.**

3. **The geometric-read kernel's HARM verdict (5.4) — the attribution control was deferred on budget.** A
   mechanism was marked HARM, and a HARM label is the strongest negative in the vocabulary, on a run where
   *"the capacity/normalization attribution control is unrun"* and *"a guardrail failure is itself a HARM
   condition"* — i.e. a **1.84× vs 1.50× timing miss was folded into a mechanism verdict.** Arm C is the
   pre-specified control and was skipped for ~4M ms of ledger headroom.

4. **G-binding's masking arm (4.4) — a 6× capability gain blocked by a probe guard.** Held-out Updated
   0.158 → **0.967**, `Unavailable` → **0**, tag accuracy 0.974 → 0.996, blocked by +0.119 nats of text NLL on
   a 1.4M-parameter synthetic probe against a 0.05 guard. The document explicitly leaves the call open and the
   record never resolves it.

### Tier 2 — the rejection is defensible but the conclusion drawn from it is too wide

5. **Outcome D (1.6)** — the kill is clean on its own terms and I would not re-run A1. But the **scope** of the
   conclusion was extended from "no pointer-class mechanism learns keyed retrieval at ~2.1M / 2,590 steps" to
   "no learned retrieval at this scale," and onto that wider claim the programme (i) stopped all local selector
   work, (ii) skipped the transformer+pointer control, (iii) never ran the Lorentz-scored pointer it had
   specified, (iv) closed the read-parity question the D17 rule required a third seed for, and (v) inherited a
   **structural 0/33 copy panel** from a tokenizer `add_prefix_space: false` artifact. Each of (ii)–(v) is a
   materially changed successor under D19 §3 and none has been run.

6. **Removing the individual-history row (7.1)** — the effect is **below the project's own 0.01-BPB equivalence
   margin** (0.0073 bits new panel; 0.0027 for the clean Honly-vs-E ablation), the "winning" comparator has a
   documented 34-target population defect, the clean exact-tail donor test **spanning zero** is underpowered at
   n=1,177, and the panel is reused open development at one training dose. The review's own hedge
   (*"no demonstrated net predictive benefit … at this training dose and support"*) is correct; the
   **architectural** conclusion drawn downstream is not supported by it.

7. **B1 tracking-lane closure (4.1)** — the mechanism's headline capability was **perfect** (1.000 A5 tracking
   at length 4,096, exact to 128×, served in one byte and two table reads). It was closed by a text-cost gate
   whose tolerance (0.05 nats) is **smaller than the lane-free baseline's own seed spread (0.063 nats)** — and
   the run used a configuration where only **1–3 of 8 lanes** close into an automaton, so 5–7 dead lanes were
   charged to the text loss. The pre-registered kill rule was withdrawn by D12 and the mechanism was re-listed
   as an active candidate to be trained jointly with S4 — **which has never been done.**

8. **v22 split compiler (3.3)** — the design did exactly what it was built to do (relation 0.900 → 0.922, act
   head repaired from 41/313 to 266/313) and was closed on **2 answers out of 52** against a rule that said
   *"clearly above 34/52"* with no margin specified, on one development draw. The card's own diagnosis names a
   concrete successor (query-side relation head). Contrast with v23 (3.4): also below v19, but investigated
   rather than closed, and it is now the most promising line in the project.

9. **S2 QAT (2.7)** — the pre-registered ≥29/58 float-agreement gate was applied to a comparison in which the
   *unquantized* control itself drifted to 7/58 turns. QAT scored 6/58. The gate measured schedule drift, not
   quantization. The one measurement that isolates the effect (QAT integer vs its own unquantized float
   weights) is recorded as **missing** and still has not been taken.

10. **orthant64 (2.1)** — clean criterion, but a candidate that **beat its own parent on the primary loss
    metric in both arms** was killed by a 12-row complete-answer count, and the same run proved the loss was
    recoverable by changing only the access policy. The `exact_cache64` < `recent64` ordering that decided the
    next reference rests on 4-answer differences at n=32 on one paired seed.

### Tier 3 — genuinely clean rejections (I would not re-open these)

- **The Peter–Weyl harmonic filter sequence (4.6)** — five levers ruled out with stated reasons, code reverted,
  mathematical result retained. Exemplary.
- **A2's zero-context-bank finding (1.2)** — confounded but the confound is the finding (soft-vs-hard objective
  mismatch), and it directly produced the A3 integer-margin design.
- **A4 (1.4)** — export retention failure, self-diagnosed, self-corrected on scope.
- **Causal continuation (1.5)** — the geometric claim was self-refuted by adding the missing ordinary control.
- **Termination weighting (5.3)** and **selection policy (5.2)** — both pre-registered, both with parity
  checks, both null, both correctly closed. 5.2 even self-corrected a citation error.
- **v21 reviewed-vs-raw labels (3.3)** — properly paired, null, and it found 11 defects in the review tooling.
- **Precision factorial (2.4)** and **projected recurrent (2.6)** — pre-registered, both arms, exact numbers,
  scope stated. The projected-recurrent anomaly (*"shadow likelihood improves … yet packed likelihood does not
  improve"*) is a genuine open scientific question, not a rejection problem.
- **D6 (4.5)** — bounded, honest about Δ being a ranking metric, explicitly *"not a veto."* The 89%-without-2I
  observation (U vs D) is a nuance the disposition did not use.
- **Joint fit / training budget (7.4)** and **count-prior blend (7.3)** — both correctly scoped negatives that
  changed the programme's direction for good causal reasons.
- **E3's gate (3.1)** — never met, honestly reported, not weakened — but see the `TOO TIGHT` note on its
  instrument (a linear probe on frozen features, against supervision the project audited as label-noisy, while
  a lexical control beats it).

### The structural observation

Three patterns recur, and they are the actual answer to "were good mechanisms retired too early?"

1. **Gates applied to quantities whose own noise floor exceeds the tolerance.** B1's text gate (0.05 nats
   tolerance vs 0.063-nat baseline seed spread); D2's margin gate (0.30 required, 0.146 attainable); outcome D's
   read-parity tolerance (0.03 vs ±0.05 evaluation sampling error at 109 items); v22's two-answer session
   deficit; orthant64's 12-row count at n=32. In each case the *measurement* was clean and the *decision
   threshold* was not calibrated to the instrument.
2. **A verdict issued while the pre-specified attribution control was unrun.** The geometric-read kernel
   (arm C deferred on budget), S2 QAT (QAT-vs-own-float missing), B3 (float reference invalidated), A2 (hard vs
   soft objective), A3 (gate and output head named as separate causes but not separately intervened), v17
   (fit not re-tuned for the new feature scale), D6 (never carried to the stack's Lorentz read).
3. **Policy withdrawals of favourable evidence.** `recent64` (withdrawn before execution, evidence favourable),
   the third read-parity seed (pre-declared, then forbidden by the same experiment's kill), the oracle re-rank
   intervention (recommended, never scheduled), and D12's un-retirement of mechanisms that were then never
   re-tested (2I lanes, exact memory, sparse index, 2I/E8 read codes).

The counterweight — and it is real — is that this record contains **exemplary** falsification discipline: D2's
"measurement, not a verdict," the Peter–Weyl five-lever sequence, the v21 label audit, the self-corrections in
A4 and selection-policy, the explicit *"A linear probe is not a mutual-information bound and does not establish
a family-wide geometric capacity failure,"* and D12's own list of five near-misses that had become family
eliminations. **The programme's problem is not that it lacks the policy to avoid premature retirement. It is
that the policy was written after the retirements, and the re-tests it mandates have not been run.**

---

## Appendix — records consulted (primary, with the disposition each carries)

**Decisions:** `docs/integration/DECISIONS.md` (D0–D19).
**Plans of record:** `docs/integration/project-track.md`, `STATUS.md`, `ROADMAP.md`,
`docs/integration/agent-execution-policy.{md,json}`, `docs/integration/mechanism-admissibility-2026-10.md`,
`docs/integration/geometric-toolbox-2026-09-28.md`.
**Current state:** `docs/integration/current-state.md` (1794 lines) and
`docs/integration/current-state-archive-through-2026-09-24.md`.
**Results/negatives cited:** `integrated-attention-a{1,2,3,4}-result-2026-09-24.md`;
`causal-continuation-result-2026-09-24.md`; `compiled-relative-query-result-2026-09-24.md`;
`addressed-lexical-bridge-result-2026-09-24.md`; `ordinary-lexical-audit-2026-09-23.md`;
`ordinary-lexical-count-blend-result-2026-09-23.md`; `ordinary-lexical-joint-fit-result-2026-09-23.md`;
`occurrence-reader-review-2026-09-20.md`; `query-read-review-2026-09-20.md`;
`s-attribution-result-2026-09-20.md`; `query-read-result-2026-09-20.md`;
`bounded-admission-{plan,result,outputs}-2026-09-25.md`; `precision-factorial-{result,review,outputs}-2026-09-25.md`;
`projected-recurrent-{plan,result,review}-2026-09-25.md`; `quantized-recurrent-result-2026-09-25.md`;
`learned-rounding-{plan,result,outputs}-2026-09-25.md`; `joint-recurrent-result-2026-09-25.md`;
`integer-execution-result-2026-09-25.md`; `integer-serving-result-2026-09-25.md`;
`language-continuation-{plan,result,direction}-2026-09-2{5,6}.md`;
`emission-selection-diagnostic-{plan,result}-2026-09-27.md`;
`selection-policy-diagnostic-{plan,result}-2026-09-27.md`;
`termination-objective-{plan,result,review}-2026-09-27.md`;
`geometric-read-{plan,result}-2026-09-27.md`; `read-localization-{plan,result}-2026-09-27.md`;
`radial-adaptation-{study,result}-2026-09-27.md`; `b1-finite-group-lanes-2026-09-27.md`;
`b1c-context-lanes-swap-stories-2026-09-28.md`; `d2-aerm-probe-{plan,result}-2026-09-28.md`;
`g1-always-on-read-result-2026-09-28.md`; `g-binding-result-2026-09-29.md`;
`d6-information-audit-result-2026-09-28.md`; `d4-geometric-s1-qat-result-2026-09-29.md`;
`d4-s2-dialogue-qat-result-2026-09-29.md`; `b3-e8-smollm2-result-2026-09-29.md`;
`dialogue-code-choice-result-2026-09-28.md`; `addressing-contest-result-2026-09-28.md`;
`structural-memory-hopf-direction-2026-09-20.md`; `geometric-attention-mechanism-synthesis-2026-09-20.md`;
`repo-review-direction-2026-09-23.md`; `model-direction-2026-09.md`;
`principal-attention-{mathematics,engineering,computer-science}-2026-09-24.md`;
`geometric-attention-2026-09-26.md`; `first-principles-review-2026-09-25.md`;
`kvar-recall-result-2026-09-24.md`; `direction-review-2026-09-30.md`;
`grounded-memory-evidence-2026-10-01.md`; `log-sieve-retrieval-design-2026-10-01.md`;
`EVIDENCE.md`; `resource-ledger-2026-09-19.md`.
**Live GitHub:** issue [#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552) (85 comments, the
authoritative newer record for E1–E4/v12–v25); issues #973, #820, #962, #963, #964, #1511, #1512, #1515, #1519,
#1546, #1551, #1563, #1573; PRs #1310, #1312, #1386, #1387, #1390, #1391, #1433, #1464, #1469, #1470, #1478,
#1491, #1505, #1518, #1532, #1540, #1548, #1554, #1557, #1562, #1565, #1578.
*(No file in the checkout was modified. No cargo build was executed. The GitHub issue bodies were read via the
authenticated `gh` CLI.)*
