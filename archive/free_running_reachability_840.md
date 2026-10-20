# Free-running reachability instrument — corrective student-prefix observation is not launched (#840)

- **Status:** Frozen reachability instrument + executed run for S3 item B (#840, tracker
  #824, programme #820). Companion to the #841 gap contract
  ([`docs/free_running_eval_841.md`](free_running_eval_841.md)). This record executes the
  #840 run contract's binding cheap instrument and records the do-not-launch decision; it
  does **not** change serving behavior or move a stage gate.
- **Date:** 2026-08-22.
- **Claim language:** follows [`docs/formal_vocabulary.md`](formal_vocabulary.md)
  (normative). This record establishes the reachable ceiling for corrective observation
  against the promoted skip-mix representation; it does **not** claim generation coherence.
- **Harness:** `crates/uor-r4-api/tests/free_running_reachability_840.rs` (fixture teeth
  non-ignored; `free_running_reachability_run_840` ignored/bundle-gated). Record:
  `docs/free_running_reachability_840_result.json` (CID-bound, `result_cid
  blake3:e9f48e20…`).
- **Execution scope (#830):** offline `certifier-instrument` over the `deployed-serving`
  surface; teacher-free (scored against the bundle's recorded `t_argmax`). Reference /
  offline evidence is not credited as deployed generation.

## 1. Why this instrument exists

#840 begins only against the NEW (skip-mix) representation — the maintainer HOLD (recorded
on #840, 2026-08-21) was lifted by the S1 PROMOTE (the lane is RF-31, activated in #910).
#840's scope is a bounded multi-round corrective-observation experiment governed by the
repository long-run discipline (`AGENTS.md`). That discipline requires, BEFORE any run
measured in hours: (1) reachability arithmetic, (2) a binding cheap instrument whose
verdict is a hard gate, and (3) distinct positive/negative next actions. This record is
that gate.

**Objective (what corrective observation can move).** Student-prefix corrections reshape
the offline-compiled graph/skip-mix tables; they can reduce the free-running gap only where
the trajectory (a) departs the recorded text AND (b) is driven by a correctable
(graph/skip-mix) path rather than the memoryless suffix. The frozen #841 §6 stopping rule
requires, per round: median first-divergence +≥2 steps AND diverged-at-step-0 −≥100‰, no
teacher-forced-agreement regression >10‰, at most 3 rounds else `GENERATION-NOT-ESTABLISHED`.

## 2. What the instrument measures (teacher-free)

The frozen prompt-family v1 of #841 (first 100 held-out stories with a full 8-token window
and ≥32 recorded continuation; family CID `blake3:6cad1dfe…`), driven through the normative
deployed `R4Engine`, on TWO engines built from the SAME recompiled sections (verbatim
#908/#906 machinery):

- `base` — empty SKMX/PSIB sections; `predict_decision` is byte-identical to plain base
  (absent-section identity). Base graph CID `blake3:aaf98b68…` is **byte-identical to
  #908's committed `base_artifact_cid`**, an independent reproduction of the recompile.
- `skip` — the real `skipmix_fit::fit_skipmix_tables` sections (the deployed RF-31 lane).
  `predict_decision` routes through the lane when the sections are present (#910), so
  free-running rollouts on `skip` ARE the deployed lane under the model's own prefixes.

Metrics per engine at H∈{8,32}: median first-divergence, diverged-at-step-0,
survived-full-horizon, ≤4-period cycle collapse, per-step ExactContext/NGRAM/Graph
attribution, the suffix-locality statistic (FR rollout token-identical to the last-2-token
rollout), and the cross-engine count of free-running rollouts the lane changed. Greedy;
determinism double-checked in-harness. The base engine reproduces #841 as a validation tooth.

## 3. Run contract (posted before execution; do-not-launch outcome)

    metric to move:       #841 primary — median first-divergence at H=32 (currently 0) and
                          diverged-at-step-0 (currently 590‰), under the frozen §6 bar.
    reachability ceiling: corrective observation reshapes the graph/skip-mix tables; their
                          free-running footprint bounds the reachable diverged-at-0 drop.
                          Measured ~1‰ (3/3200 served steps on the Graph path) — ~100× below
                          the 100‰ bar. Arithmetic from the skip engine's H=32 path histogram.
    instrument + verdict: this harness (free_running_reachability_run_840). PROCEED only if
                          the activated lane moves free-running TOWARD coherence (median up or
                          at0 down) AND the correctable footprint could support a 100‰ at0 drop.
    exit rule:            launch corrective rounds only on a PROCEED verdict.
    if positive:          build the corrective-observation subsystem and run ≤3 rounds under §6.
    if negative:          do not launch; record whether representation, data, or decoding
                          limits; classify generation not established if the stage gate is unmet.
    cost estimate:        instrument ~15s run (teacher-free) + ~1min in-harness recompile; the
                          averted corrective run is multi-hour (teacher-correction + rounds).

## 4. Result (2026-08-22; greedy; CID-bound record)

**Empirical Criterion. Status: Empirical.** `docs/free_running_reachability_840_result.json`
(`result_cid blake3:e9f48e20…`; corpus.meta CID `blake3:aa9d1767…`, the attested #833
broad-clean bundle).

| metric (H=32, greedy) | base (no lane) | skip-mix lane (RF-31) |
|---|---|---|
| median first-divergence | 0 | 0 |
| diverged-at-step-0 | 590‰ | 620‰ |
| ≤4-period cycle collapse | 710‰ | 1000‰ |
| matched teacher-forced agreement | 304‰ | 348‰ |
| suffix-locality (FR ≡ suffix-only, of 100) | 99 | 19 |
| free-running Graph-path footprint | ~1‰ (3/3200) | ~1‰ (3/3200) |
| free-running rollouts changed by the lane | — | 100/100 |

- **The instrument reproduces #841.** The base engine's numbers match
  `docs/free_running_841_result.json` exactly (304‰ TF, 99/100 suffix-local, path
  histograms), and the base graph CID reproduces #908's `base_artifact_cid` — the harness is
  validated against two independent prior records.
- **The lane helps teacher-forced, not free-running.** Matched teacher-forced agreement
  rises 304 → 348‰ (consistent with #908's +28.45‰ deployed causal on the held-out split).
  Under the model's own prefixes the lane instead regresses coherence: diverged-at-0
  590 → 620‰, cycle collapse 710 → 1000‰, median first-divergence unchanged at 0. This is
  exposure-bias amplification — the lane, fitted on correct (teacher) contexts, misfires on
  self-generated contexts.
- **The lane perturbs but does not correct.** It changes every free-running rollout
  (suffix-locality 99 → 19/100; 100/100 changed), so it is not inert, but the change is
  net-negative and operates through the ngram-candidate path (Graph-path footprint ~1‰).

## 5. Decision — do not launch; GENERATION-NOT-ESTABLISHED

Two independent bounds show a corrective run cannot clear the frozen §6 bar:

1. **Reachable ceiling.** Corrective observation reshapes the graph/skip-mix tables, whose
   free-running footprint is ~1‰ — roughly 100× below the §6 bar's required 100‰
   diverged-at-0 drop.
2. **Direction.** The best available representation (the promoted RF-31 lane) moves
   free-running away from coherence, so corrective rounds of the same evidence family have
   no path to the bar.

By the repository run contract the corrective run is **not launched** (the reachable ceiling
is below the target effect). The limiting factor is **representation**, not more-data or
decoding. Per the #841 §6 rule and the #840 negative branch, the disposition is
**GENERATION-NOT-ESTABLISHED** for corrective student-prefix observation against this
representation.

This is an evidence-backed negative result. It meets the programme global falsifier
([`docs/r4_intelligence_completion_plan.md`](r4_intelligence_completion_plan.md)): *if
bounded student-prefix correction does not reduce the frozen free-running gap, classify the
artifact as a teacher-forced retrieval/continuation system rather than a generative
intelligence engine.* Accordingly the free-running generation claim is **not established**
at this scope. The formal S3 generation-claim decision (promote/revise/limit/retire)
belongs to #842 (item C), which this record feeds.

## 6. Repository conformance

- **Execution scope:** offline certifier-instrument measured through the normative deployed
  `R4Engine` path; teacher-free. Not credited as deployed generation.
- **Conformance mapping:** RF-14 / RF-21 / RF-22 / RF-29 evidence language (extends existing
  capabilities). No new built capability, no `model/ids.toml` row, no `CONFORMANCE.md`
  regeneration — this is evaluation / instrument infrastructure (identical treatment to #841).
- Preserves P-4, allocation-free steady state, and determinism: no runtime, format, or
  compiler code is touched; the instrument only reads the deployed engine and recompiles
  sections through existing offline APIs. Empirical criteria bind distribution, horizons,
  decoder, n, provenance, and PASS/FAIL/UNAVAILABLE.
- Appends this record and reconciles `docs/RESEARCH.md`. It does **not** rewrite the #841
  record.

## 7. Claim status and next action

A better teacher-forced fit does not change the generation claim (#840). No predeclared
sequence-level advantage over equal-budget controls is reachable, so the generation claim is
**not established**. Next: #842 (item C) gates trajectory state and decides the free-running
generation claim; any future re-entry requires a representation with cross-step free-running
memory (a redesign-scope question, echoing the S1 REVISE pattern) and must re-enter under
the frozen #841 §6 bar.
