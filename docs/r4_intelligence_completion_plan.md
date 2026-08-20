# R4 Intelligence Completion Plan

- **Status:** Authoritative for post-v0.1 intelligence sequencing (adopted 2026-08-19, issue #829).
- **Source of record:** GitHub tracker root #820 and its native sub-issues. Where this
  document and the live GitHub hierarchy disagree, **the native parent/sub-issue tree on
  GitHub wins**; this file is the readable, linkable mirror maintained alongside it.
- **Relationship to earlier plans:** This document supersedes, for *post-v0.1 sequencing
  only*, the phase ordering in `docs/r4_graph_compiler_implementation_plan.md` and the
  `.github/DOCUMENTATION_OVERHAUL_PLAN.md`. It does **not** retract or rewrite any prior
  engineering plan or measurement record — those remain valid history and, where noted,
  valid engineering references. Claim language in this document follows
  `docs/formal_vocabulary.md` (normative), and `python3 scripts/check_claim_wording.py`
  gates it in CI.

## Why this plan exists

UOR-R4 reached a reproducible v0.1 engineering baseline: it compiles a pinned Hugging Face
teacher into a multiplication-free, table-native artifact with a witnessed integer runtime,
serves it, and ships it as a verified release. What it does **not** yet have is a
capability-sequenced research programme that promotes each intelligence claim only against
replayable evidence at the exact execution scope claimed.

The programme deliberately corrects several stale premises carried in earlier planning prose
(all corrections are append-only against the historical records; the live summaries are
reconciled, the records are not rewritten):

- **#784 refuted full-depth context-code collision**, reporting **0.0% full-depth
  context-code collisions at k = 2/4/6/8**. The observed effect is **continuation-distribution
  convergence** — distinct context codes still favor a shared continuation (11/15 distinct
  rows still favored newline) — *not* a code collision. Earlier prose that framed #784 as a
  "context/code collision" is superseded.
- **#811 wired the deployed D4 abstention policy into the CLI ask path**, but all five
  semantic out-of-distribution probes remained `SERVABLE`. **D4 parity is not semantic
  answerability**; semantic unanswerability detection is *not* established. This is a
  measured substrate property, the same family as #784.
- **#605/#804 made the real route-attention arm executable and then closed it with a
  negative S1 verdict** (2026-08-19): the pre-registered anti-vacuity null was not cleared
  (teacher attention supports are temporally smooth at this scale; 119/120 heads individually
  vacuous), so the instrument licenses nothing. The `R4RouteAttentionV1` operator stays
  **dormant** behind its unchanged `model/ledger.toml` gate. W(3,3) remains a separately
  qualified research hypothesis, **not** the graph-migration path.
- R4G1 is already multiresolution and patch-capable. **R4G2, network composition, and
  larger-scale work are trigger-gated**, not assumed next steps.
- Seeded sampling restored valid surface output, but has **not** established prompt causality
  or coherent free-running generation.

## Goal

Produce a CPU-first, deterministic, bounded, multiplication-free deployed system whose prompt
conditioning, selective prediction, free-running behavior, typed-state planning, instruction
behavior, scale, composition, and release claims are each backed by replayable evidence **at
the exact execution scope claimed**.

## Programme hierarchy

Native sub-issues are the source of truth. The root tracker (#820) has eight stage trackers
plus one cross-cutting formal foundation. Each stage tracker owns its bounded native
children, its milestone, its promotion gate, and its final verdict.

| Lane | Tracker | Title | Milestone |
|---|---|---|---|
| **F0** | #859 | Lean formalization of the stable R⁴ prime-router mathematical kernel | (cross-cutting; runs during S0) |
| **S0** | #821 | Truth baseline, conformance, and normative inference closure | R4 Intelligence / S0 |
| **S1** | #822 | Persistent prompt-conditioned predictive state | R4 Intelligence / S1 |
| **S2** | #823 | Evidence-grounded selective prediction and calibrated abstention | R4 Intelligence / S2 |
| **S3** | #824 | Coherent free-running generation and trajectory correction | R4 Intelligence / S3 |
| **S4** | #826 | Learned semantic planning and geometry qualification | R4 Intelligence / S4 |
| **S5** | #825 | Native instruction behavior and production delta covers | R4 Intelligence / S5 |
| **S6** | #827 | R4G1 v1 scale, streaming, and local composition | R4 Intelligence / S6 |
| **S7** | #828 | Proof/release closure and distributed composition | R4 Intelligence / S7 |

### S0 native children (this stage's ordered work) — tracker #821

1. #829 — Publish the completion plan and evidence-first issue forms *(this document)*
2. #830 — Encode execution scope, serving reachability, and non-vacuous verdicts
3. #831 — Designate the normative R4G1 scorer
4. #832 — Commit CID-bound suites and per-token attribution
5. #833 — Rebuild the attested broad bundle and re-ratify the baseline

Stage order, then the parent tracker's listed child order, governs what is worked next.
A later stage's preparatory child does **not** displace the next sequential stage item
merely because it lacks a direct blocker. Cross-cutting foundation work (F0) may proceed in
parallel but does not displace the next sequential stage item unless the roadmap says it
gates that item.

### S0 stage verdict — PROMOTE (2026-08-20)

All five S0 native children are closed with recorded verdicts and every promotion-gate
condition on tracker #821 is met, so **S0 is promoted** and its downstream stages S1 (#822)
and S2 (#823) are unlocked. This entry is the readable mirror of the tracker's closure
verdict; **GitHub #821 remains the source of record**.

- **All children have final verdicts.** #829 (PR #860, `d1d1f735`), #830 (PR #861,
  `bc5a2a92`), #831 (PR #862, `1c377df7`), #832 (PR #863, `70586aa0`), and #833 (PR #867,
  `1b3e46f2`; determinism/provenance chain #864 `9c1f2c10`, #866 `19465441`) all closed
  COMPLETED.
- **One normative scoring path, differentially tested.** The deployed R4G1 runtime scoring
  path is designated the single normative scorer in [ADR-0001](adr/0001-normative-r4g1-scorer.md)
  (#831); spec-vs-deployed differential checks and a planted-negative control live in
  `crates/uor-r4-graph-certify/tests/normative_scorer_831.rs` (record:
  `normative_scorer_831.md`).
- **Frozen benchmark identities and attribution.** CID-bound capability suites and per-token
  resolution attribution land in `crates/uor-r4-api/src/capability_suite.rs` with committed
  manifests (#832; record: `capability_suites_832.md`).
- **Attested post-#755 broad bundle admitted.** A source-complete, #755-native,
  byte-reproducible bundle (`smollm2-360m-broad-clean`) passes deployed R4Engine admission and
  the offline prediction canary (#833; record: `attested_broad_baseline_833.md`).
- **M.V.G. thresholds — RETAINED with dated evidence.** Gate C RATIFY (RETAIN), 2026-08-20:
  the headline causal serving rows reproduce within the predeclared `<~0.5 pp` reachability
  bound (Rule 1+2 24.30% → 24.39%, best-live 31.48% → 31.11%, TLA-3 28.21% → 28.12%). The
  teacher floor (3.6015 bits) is RETAINED by invariance; a full-population re-measurement is
  **UNAVAILABLE** at scale under the current exact-GEMM teacher path (~24 h for the held-out
  split) and is a recommended follow-up, not a promotion blocker.

The stage kill/redesign criterion is **not** triggered: the claimed signals reproduce from
pinned source inputs through the deployed path (byte-reproducible rebuild, admitted bundle,
Gate C within bound). Downstream quality, geometry, and scale promotions are therefore
authorized to proceed from this baseline.

## Dependency order

```text
F0 ─────────────────────────> #845 (geometry qualification, S4)
 └───────────────────────────> #856 (production refinement, S7)

S0 ─┬─> S1 ─> S3 ─> S4 ─> S5 ─> S6 ─> S7
    └─> S2 ─────────────────> S5
```

- **F0 (#859)** starts during S0 but does **not** gate S1–S3. It gates experiments or formal
  claims that depend on prime-router mathematics. Its Lean theorems remain **reference
  evidence** until #856 either proves a refinement to stable deployed semantics or explicitly
  excludes the component. F0 is deliberately **not** a child or promotion blocker of #821,
  because the current f64 prime router is outside the graph-migration path.
- **S2** may proceed alongside S1, but any public generative or instruction-capability
  promotion requires the relevant selective-risk gate.
- Formal work may run continuously; **S7 closes only against stable normative bytes and
  serving semantics.**

## The two-level proof strategy (F0 now, refinement later)

Formalization is deliberately split so it delivers value early as a design/assumption audit
without overstating the current router's production relevance:

- **Level 1 — reference mathematics (#859, now).** A pinned, importable Lean 4 library for
  the **stable pure mathematical kernel** of the R⁴ prime router. Its top-level target is a
  totality/determinism/boundedness/witness-replay theorem over declared configurations and
  inputs. It explicitly does **not** assert semantic quality, intelligence, collision
  freedom, Riemannian-geodesic status, Rust/`f64` equivalence, production reachability, or the
  Riemann hypothesis. Every precondition and external authority stays explicit. This is
  **reference-only** evidence (**Definition**/**Assumption** scope), never described as
  deployed-serving evidence.
- **Level 2 — production refinement (#856, later, S7).** A separate, later proof layer that
  either refines the Level-1 boundary to stable deployed semantics (Rust/`f64` or
  packed-runtime refinement) or explicitly **excludes** the component from the release claim.

### Proof-asset boundaries (do not conflate)

These are distinct artifacts with distinct scopes. A claim citing one must not borrow another's
authority:

| Asset | What it is | Scope / claim boundary |
|---|---|---|
| **R4 prime-router Lean package** (#859 deliverable) | New Lean 4 library for the pure reference kernel of the prime router | Reference mathematics of the *f64 router kernel*; **not** deployed-serving evidence, **not** the graph path |
| `research/riemann-lean` | Separate conditional Prime/Riemann bridge, Lean 4.28, supplied theorem/provider boundaries | Independent research; **not** a router specification or implementation refinement; absent from root CI |
| `proofs/wasm-gemm-gnaf` (vendored GNAF) | Lean 4.30 proof that a reference WASM GEMM/GNAF kernel is cost-optimal (#653) | Proof-**process** methodology is reusable; its theorems do **not** transfer to R4 serving or text generation |
| `crates/uor-r4-proof-model` | Rust **executable** proof obligations + proof-status matrix + a small **Kani** surface | Structural/`ExecutableSpec` obligations over the deployed runtime and format; not a Lean formalization of the router |
| Kani harnesses (`kani_proofs.rs`) | Bounded model-checking of score arithmetic safety and fixed-capacity container invariants | **Structural** guarantees on specific runtime obligations only |
| Hologram/R4 monograph (`docs/hologram_r4_formal_monograph.md`) | Normative specification and evidence map | Specification/evidence map; **not** a Lean theorem library |
| Target-binary evidence | The actual release binaries and reachable production call graph | The only evidence that supports **deployed-serving** claims (S7 / #856) |

### Mechanism boundaries (distinct components, distinct roles)

| Component | Role | Status |
|---|---|---|
| **W(3,3)** visualization / dashboard field | 96 rendered vertices in the browser dashboard; a research/visualization construct | **Not** a normative construction of the 40-point/40-line W(3,3) quadrangle; **not** an input or output of the Rust router; separately qualified research hypothesis |
| **Geometric prime router** (`crates/uor-r4-router`) | Validated `f64` content-query retrieval/routing (MRR 0.88+, #486/#490/#502) | Real, load-bearing for retrieval; runs on `f64` **outside** the P-4 kernel by design; strengthens the product, is not itself the product |
| **R4G1 graph scoring** (`uor-r4-core` `score.rs` / R4G1 adapter) | The compiled-artifact scoring path | On the normative production path; the S0 designation of *one* normative scorer is #831 |
| **Route-attention reference work** (`R4RouteAttentionV1`, #604/#605/#804) | P-4-legal integer attention analog | **Dormant**; S1 verdict FAIL (instrument vacuous); wired into no serving path; ledger-gated |
| **Normative production path** | The deployed graph runtime + one designated normative scorer | The only surface whose evidence is credited as deployed-serving |

## Programme invariants

- The deployed hot path remains **XOR/AND/OR/shift/rotate/popcount/integer add-sub/compare/
  table-read only**: no multiply, divide, or floating point.
- Prediction remains **allocation-free in steady state**, bounded, deterministic, `no_std`
  where currently normative, and free of recoverable-path panics.
- **One normative scorer** owns production serving semantics, certification, patch
  interpretation, witness replay, and proof targets.
- Identical pinned inputs produce identical artifact bytes (byte reproducibility). Corpus,
  teacher, tokenizer, compiler, benchmark, decoder, and report identities are content-bound.
- Reference, offline/compiler, certifier, dormant portable-runtime, normative runtime, and
  deployed-serving evidence are **never** conflated.
- Empirical gates use document-disjoint data, powered samples, non-degenerate nulls, negative
  controls, uncertainty, and predeclared outcome branches. Unavailable fixtures are
  `UNAVAILABLE`, never a vacuous PASS.
- Long runs obey the repository run contract (reachability arithmetic → binding cheap
  instrument → distinct positive/negative next actions; see `AGENTS.md`).
- Negative results narrow or retire claims; they are first-class completion evidence and do
  **not** silently promote downstream stages.
- This programme does **not** claim exact teacher equivalence, does not claim human-level
  reasoning, and does not treat plausible language output as evidence of coherent internal
  state transitions (`docs/formal_vocabulary.md` §2.1, §5).

## Promotion gates and kill/redesign criteria

Each stage tracker records its own promotion gate and kill/redesign criterion; the
authoritative text lives on the tracker issue. The global falsifiers that bind every stage:

- If two independently motivated conditioning mechanisms fail on document-disjoint,
  EXCT-disabled causal tests, revisit the representation/compiler model rather than continuing
  code-space tuning.
- If bounded student-prefix correction does not reduce the frozen free-running gap, classify
  the artifact as a teacher-forced retrieval/continuation system rather than a generative
  intelligence engine.
- If planning gains disappear under relabeling, unseen composition, or topology changes,
  classify them as memorization.
- If a geometry arm cannot beat Hamming/binary/VSA/spectral controls under equal bytes,
  candidates, and operation budgets, keep it out of the production runtime.
- If a claim cannot be reproduced from pinned inputs, or its serving reachability cannot be
  shown, it remains unavailable or reference-only.

## Definition of completion (root #820)

- The stable prime-router mathematical kernel has a kernel-checked theorem/assumption
  boundary, and every release component either refines that boundary or explicitly excludes it.
- Prompt meaning exerts measured causal influence beyond suffix/NGRAM and exact-context memory.
- Unsupported answers are declined at a predeclared risk/coverage operating point while
  answerable novelty remains served.
- Teacher-forced improvements transfer to coherent free-running trajectories across a frozen
  horizon ladder.
- Typed-state planning generalizes across held-out entities, compositions, topologies,
  relabelings, and horizons.
- Any promoted geometry beats non-geometric controls under equal bytes, candidates, and
  operation budgets.
- Instruction behavior survives unseen templates and does not erase base capabilities.
- Compilation and inference satisfy declared memory, latency, artifact-size, and
  deterministic-composition envelopes.
- Each guarantee has a scoped proof status and artifact; each empirical claim has a CID-bound,
  replayable report.
- The actual release binaries and reachable production call graph support the advertised
  operation contract.
- A complete bundle installs and serves offline from an empty model root on every supported
  target.

## Repository conformance

Every child declares its execution scope and maps to existing RF capability IDs (RF-01…RF-29)
or justifies a new capability ID. A new **built** capability lands in this order:

`model/ids.toml` row → tagged Gherkin → failing marker/behavior test → implementation →
regenerated `CONFORMANCE.md`.

`CONFORMANCE.md` is generated and is **never** edited directly. Dormant mechanisms remain
`open` in `model/ledger.toml` with explicit activation gates. Measurements that revise a claim
append a per-issue evidence record (`docs/<topic>_<issue>.md`) and reconcile README, ROADMAP,
RESEARCH, lifecycle/configuration, and ledger assertions.

## Historical milestone disposition

The repository carries **two distinct milestone eras**, and they must remain distinguishable.
The twelve `Phase 0`–`Phase 11` milestones belong to the original **graph-compiler engineering
plan** (`docs/r4_graph_compiler_implementation_plan.md`). The eight `R4 Intelligence / S0–S7`
milestones belong to **this** post-v0.1 programme (#820). The naming already separates them
(`Phase N` vs `R4 Intelligence / SN`); this table records the maintainer-visible disposition of
each historical milestone. **Titles are not repurposed**, and no measurement history is
rewritten. As of 2026-08-19 every historical milestone has **0 open issues**.

| # | Historical milestone | Issues | Disposition |
|---|---|---|---|
| 1 | Phase 0: Baseline and contracts | 6 closed | **Completed / historical.** Baseline, R4G1 RFC, D1–D3 delivered. Superseded for sequencing by #820. |
| 2 | Phase 1: Packed graph format and trusted parser | 3 closed | **Completed / historical.** `uor-r4-graph-format` shipped. |
| 3 | Phase 2: Overlapping multiresolution regions | 11 closed | **Completed / historical.** Cover induction / multi-membership shipped. |
| 4 | Phase 3: Semantic anti-degeneracy | 3 closed | **Completed / historical.** |
| 5 | Phase 4: Transitions and residual emission | 9 closed | **Completed / historical.** ScoreQ residuals / bounded top-K shipped. |
| 6 | Phase 5: Allocation-free graph runtime | 7 closed | **Completed / historical.** `no_std` allocation-free runtime + witness replay shipped. |
| 7 | Phase 6: Boolean routing synthesis (trigger-gated) | 1 closed | **Historical, trigger-gated / not fully triggered.** Remains dormant until its D5 trigger; not on the current sequential path. |
| 8 | Phase 7: Hardware-aware packing (trigger-gated) | 1 closed | **Historical, trigger-gated / not fully triggered.** Deferred to S6/S7-era scale work where relevant. |
| 9 | Phase 8: Long-context state | 0 issues | **Superseded for sequencing.** Long-context intelligence work is re-homed under S1/S6 of #820. |
| 10 | Phase 9: Epochs and patches | 1 closed | **Historical.** R4G1 patch chains shipped; further composition re-homed under S6 (#827). |
| 11 | Phase 10: Formal verification | 1 closed | **Historical; continued under F0/S7.** Executable spec + Kani + proof-status matrix exist; formal work continues under #859 (F0) and #828 (S7). |
| 12 | Phase 11: Architecture acceleration | 0 issues | **Superseded for sequencing.** SIMD/accel is a trigger-gated S6/S7-era concern, not next-up. |

These milestones are **retained** (not deleted, not renamed) so their closed-issue history stays
addressable. Their live descriptions carry a one-line disposition banner pointing here.

## Public entry points (all link here)

This document is the single canonical plan; the following entry points link to it rather than
duplicating a drifting phase table:

- `AGENTS.md` — operating manual (post-v0.1 sequencing points here; the graph-compiler plan
  remains the engineering plan).
- `ROADMAP.md` — product capability checklist (intelligence sequencing points here).
- `README.md` — the roadmap/plan link section.
- `docs/RESEARCH.md` — measured/refuted/open research (forward sequencing points here).
- `docs/r4_graph_compiler_implementation_plan.md` — carries a status banner: retained as the
  original graph-compiler engineering plan, superseded only for post-v0.1 sequencing.
- `.github/DOCUMENTATION_OVERHAUL_PLAN.md` — marked superseded.
- Issue forms: `.github/ISSUE_TEMPLATE/research-experiment.yml` and
  `.github/ISSUE_TEMPLATE/implementation.yml` encode the run/conformance contracts below.

## Evidence-first issue intake

Two GitHub issue forms encode the repository's run and conformance contracts so new work carries
its evidence discipline from the start:

- **Research / experiment** — hypothesis; pinned identities (corpus/teacher/tokenizer/
  compiler/decoder CIDs); reachability arithmetic; null and negative control; the binding cheap
  instrument and its required verdict; predeclared thresholds and distinct positive/negative
  outcome branches; cost estimate; evidence-record path; and claim status.
- **Implementation** — execution scope; dependencies/blockers; acceptance criteria; non-goals;
  compatibility/migration; conformance mapping (RF IDs; the build order above); verification
  (the four local gates + applicable ladder); documentation reconciliation; and claim status.

## Closure rule

A stage closes only when every native child has a recorded **completed, negative,
not-triggered, or explicitly descoped** verdict and the stage promotion decision is posted with
exact evidence links (`PROMOTE`, `REVISE`, `LIMIT`, or `RETIRE`). The root #820 closes only
after #859 has a completed or explicitly narrowed formal-foundation verdict, all eight stage
verdicts are recorded, and the final capability/open-limit record is published.
