# Dormant-mechanism reassessment (#786)

- **Date:** 2026-08-18. **Context commits:** audit baseline `aea30bae`; docs reconciliation `4b9f0562`;
  measurements below taken on the pinned dev machine the same day.
- **Charter (maintainer, 2026-08-18):** for every dormant mechanism, establish why it is dormant, whether its
  activation gate is satisfiable today, whether it could help the current mechanisms, and whether the roadmap
  still matches the original spirit of the project (a CPU-first, transformerless LLM with the P-4 integer kernel).
- **Ground rules:** zero activations under this document; every ledger activation gate stays in force; anything
  worth re-measuring gets a pre-registered contract brought back for approval before any run
  (`docs/project_baseline_audit_2026_08_18.md` §17-A3, §18; AGENTS long-run discipline).

## 1. The corpus-era validity check (measured first, as #786 requires)

#755 established that pre-fix compiles reconstructed per-story context from physical record adjacency, and that
sharded-observation corpora can be story-scrambled on disk. The question for this reassessment: **were any
dormant-mechanism negatives measured on an ordering-corrupted corpus?**

**Measured result (2026-08-18):** the committed 500k fixture pair (`c_meta.bin`/`c_recs.bin`, 500,000 records,
2,507 stories, 48-byte records, `done=1`) is **perfectly story-contiguous — 2,506 story-boundary crossings
(the contiguous ideal) and 0 story revisits.** Every fixture-based negative therefore stands on a valid corpus.
The router-harness corpora (hopf/geometry/lexical families) never pass through `load_corpus_bytes` at all, so
the #755 mechanism cannot have touched them. The only corpora known scrambled were sharded-observation outputs
(`smollm2-135m-instruct` at 99.93% non-contiguous, per #752) — none of the ledger negatives below was measured
on one. **Conclusion: the #755 caveat invalidates no dormant negative.** (Empirical Criterion; script inline in
the #786 thread.)

## 2. Per-mechanism table (all 22 `open` ledger rows)

Statuses per `docs/formal_vocabulary.md`; "gate" = the ledger activation gate, quoted in CONFORMANCE.md.

| # | Mechanism (ledger id) | Why dormant (record) | Corpus-era valid? | Gate satisfiable today? | Bearing on current mechanisms | Verdict |
|---|---|---|---|---|---|---|
| 1 | `cd-space-dormant` (#400) | Measured structurally dead: the CD term executed **0/1,998** sampled positions before removal — the node-candidate path it lived on runs only when context rows miss | n/a (non-execution, not corpus content) | No (needs a re-grounding with a positive syntactic-morphism contribution over a pre-declared null) | None today; the #785 fixes increase node-path traffic slightly but the term is removed from scoring | **Keep dormant** |
| 2 | `tropical-route-dormant` (#626) | Measured NEGATIVE on the 500k fixtures: 1H 14.51 bits/0.44% and 2H 14.52/0.44 vs NULL floor 8.57/6.2% — the E_f seat carries no route-composable token signal on this cover | **Valid** (§1) | Only via a transition/emission structure that changes what E_f carries | None on the current cover | **Keep dormant** |
| 3 | `lie-jordan-dormant` | Never positively measured; retained behind its gate as the algebra a re-grounding would need | n/a | No (gate shared with cd-space's re-grounding) | None | **Keep dormant** |
| 4 | `bott-fock-dormant` (#424) | As-shipped decay `>> 2` retains 16% of a **measured +1.02pp long-range ceiling**, so the shipped lossless bound is +0.16pp (one SE). The record itself says retuning to `>> 7` would recover the ceiling | **Valid** (fixture-based) | **Yes — the gate is "a measured lift clearing the pre-declared bar", and the retune experiment is cheap** | Direct: +1pp-class top-1 lever on the deployed context fold | **Re-measure — shortlist S2** |
| 5 | `quantum-cover-dormant` | Alternative induction path, never adopted; measured/reported only | n/a | Needs a cover scoring at/above the shipped induction on a held-out slice | Low | Keep dormant |
| 6 | `graph-construction-dormant` (`build_graph`) | Alternative construction entry point, unreferenced; induction path is what ships | n/a | Needs a construction ≥ induction on a pre-declared slice | Low | Keep dormant |
| 7 | `packed-routing-dormant` (#159 ph.2) | Declared placeholders (ROUT bytecode / typed-transition evaluators return pre-declared successors) | n/a | Needs a routing walk beating the placeholder | Design-direction surface; the actual serving router uses `rout_probe`, not these | Keep dormant |
| 8 | `endomorphism-dormant` | Operator algebra for the lie-jordan re-grounding; referenced only by it | n/a | Shared gate with #3 | None | Keep dormant |
| 9–17 | `holographic-encoding`, `predictive-sufficiency`, `shortlist-evaluator`, `anti-degeneracy`, `fairness-provenance`, `reference-compiler-ir`, `rate-distortion-compression`, `monograph-validator`, `behavioral-probes` | Certification/measurement/analysis surfaces, never serving candidates; each gated on adoption as a required gate or a ported optimisation | n/a | Each needs an adoption decision, not a measurement | Indirect (measurement infrastructure) | Keep dormant; revisit individually only when a gate-adoption question arises |
| 18 | `r4-route-attention-dormant` (#604) | Built, packed, P-4-scanned, differentially tested bit-for-bit with witness replay and zero-alloc asserted — **never fitted against a real teacher** (that is #605's real arm) | n/a (synthetic differential) | **Partially — see S1**: the snapshot prerequisite is now met; the corpus prerequisite is genuinely open (below) | The closest transformer-shaped, goal-aligned mechanism in the tree; the ROADMAP itself named it a #745-era candidate lever | **Advance via S1 (fit first; activation stays gated)** |
| 19 | `msa-structured-selector-dormant` (#643) | Built + differentially tested; the pre-registered A/B vs route-attention was named as next action and never wired | n/a | Only meaningful after S1 produces a fitted route-attention baseline (its contract compares against it) | Same family as #18 | Keep dormant; **S3 after S1** |
| 20 | `route-fit-dormant` (#605) | Synthetic 5-stage ladder PASS with a valid instrument; real-teacher stage UNAVAILABLE ("pinned SmolLM2 snapshot absent from the build env"), real-corpus stage UNAVAILABLE ("#531 saturation corpus not yet produced") | n/a | **Prerequisite A (snapshot) is now satisfied** on the dev host (257 MB at `.uor-models/sources/smollm2-135m-instruct`). **Prerequisite B is NOT**: #531 closed COMPLETED on 2026-08-11 **without a closure record** (single comment, a dependency note; no verdict, no corpus/β CIDs) — the saturation corpus is unlocated and most likely never produced. The ledger reason-string remains substantively true despite the closure | This is the gate to any real evidence for #18 | **Re-measure — shortlist S1, with the #531 question put to the maintainer first** |
| 21 | `target-operator-certificate-dormant` (#606) | Composes #605's outputs; overall verdict NOT_PASSING by construction until a real ladder exists | n/a | Follows S1 automatically | Certification of #18 | Keep dormant (mechanically advances with S1) |
| 22 | `patch-overlay-dormant` | Incremental-update surface (patch epochs); the patch *chain* is live in serving, only the overlay emission/lifecycle is dormant | n/a | Needs an overlay path scoring ≥ full recompile | Low today | Keep dormant |

**Related non-ledger negatives checked for the same corpus-era question:** #290 FMM far-field, #393 granularity,
#395 E8 group-keying, #422/#306 Hopf transport, #457 IPF, #456 reconstruction, #460 subdivision — all either
fixture-based (**valid**, §1) or router-harness-based (out of the #755 mechanism's reach). All stand.

## 3. Re-measure shortlist (contracts to be posted for approval before any run)

**S1 — #605 real-teacher route-fit arm** (advances rows 18/20/21 from synthetic to real evidence).
Prerequisite A (pinned SmolLM2 snapshot): **met**. Prerequisite B (#531 saturation corpus): **open** — #531
closed without a record and its outputs are unlocated. Two honest ways forward, **maintainer decision required**:
(a) produce a saturation corpus per #531's own method (compute-bound: a multi-scale observation sweep, hours);
(b) amend the pre-registered #605/#643 contracts to name an existing broad corpus (e.g. the #509
`smollm2-360m-broad` corpus, 0% non-contiguous) — a contract amendment, which per this repo's discipline must be
declared on the issue before the run, never silently substituted. Either way the run needs a #603 `full/1`
trace-profile observation pass (the fit consumes attention-support traces), which is the actual compute cost.
Draft contract (to finalize on #605's thread after the (a)/(b) decision): metric = the ladder's own pre-registered
gates (support overlap vs N1/N2 nulls, teacher-forced top-1 ≥ 0.90, bits ≤ 1.10× teacher, anti-vacuity N2 < 0.5×
fitted); exit = stop at first failing stage; positive → fitted instances + #606 certificate rows go real,
`msa` A/B (S3) becomes runnable; negative → the negative report closes the question with the operator retained.

**S2 — #424 bott-fock decay retune** (`>> 2` → `>> 7`). Cheap, fixture-scale, pre-declared bar already in the
record (+1.02pp measured ceiling; as-shipped +0.16pp ≈ 1 SE). Contract: A/B on the committed fixtures at census
n; exit rule = adopt only if the retuned fold clears the #424 bar with the fixture's standard error quoted;
negative → record and keep dormant. No serving default changes without its own sign-off.

**S3 — #643 msa vs route-attention A/B**: strictly after S1 positive, per its own pre-registered contract.

## 4. Roadmap-spirit conclusion

The dormant set contains **no bypassed working mechanism**: every measured negative was taken on a corpus this
reassessment verified valid, and the surfaces that were never measured are certification scaffolding, not
candidate engines. The two genuinely untested bets — the route-attention family (S1) and the bott-fock ceiling
(S2) — are both *aligned with* the original spirit (P-4-legal integer mechanisms inside the compiled-table
runtime), not alternatives to it. Meanwhile the #783/#785 arc demonstrated that the current quality losses live
in serving-side aggregation and decoding (corpus ordering — fixed by #755; the CLI serving the emission-less
converter graph while the engine misread store-container bytes as scores — fixed by #785 C1b/C1c, PRs #792/#793;
attractor basins on real evidence — open as #784), not in the dormant substrate. **The roadmap as reconciled on
2026-08-18 is confirmed in the original spirit**: bundle quality first (#783/#784/#785), with S1/S2 as the
sanctioned probes of the largest untested mechanisms, and no dormant activation except through its own ledger gate.

*Prepared under #786. Zero mechanisms were activated; zero ledger rows changed.*
