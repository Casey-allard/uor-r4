# #833 — research/#821-E: rebuild a fully attested post-#755 broad bundle and re-ratify the baseline

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:34:10Z
- **Updated:** 2026-08-25T06:11:46Z
- **Closed:** 2026-08-20T08:59:16Z
- **Labels:** graph-compiler
- **Milestone:** R4 Intelligence / S0 — Truth and inference closure
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/833
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #821

## Context

The strongest positive data path came from the broad-teacher/canonical corpus work: roughly 10.2%/29.0% thin to 24.3% canonical top-1, with a best live figure of 31.48% and a recorded teacher floor near 3.6015 bits. The currently recorded 135M bundle is not source-complete enough to settle the post-#755 production baseline.

## Problem

Later experiments cannot be interpreted against an incomplete teacher/tokenizer/corpus chain or a bundle that does not pass actual model admission. Old Gate C anchors and live serving behavior must be re-read on the same fresh artifact.

## Goal

Build one source-complete, tokenizer-attested, deterministic broad canonical bundle; pass deployed admission; run the frozen evaluation slices; and explicitly retain, revise, or retire the M.V.G. thresholds.

## Scope

- [ ] Pin teacher/source, tokenizer/adapter, corpus records and partitions, compiler revision/configuration, seeds, and hardware/resource envelope.
- [ ] Validate source completeness and tokenizer compatibility before observation.
- [ ] Run observation, compilation, cover induction, scoring, evaluate-report, manifest/CID generation, and packaging with resumable checkpoints.
- [ ] Package graph, stores/signatures, tokenizer, descriptor, reports, corpus metadata, manifests, and all source/adapter provenance.
- [ ] Double-build or checkpoint/replay the decision-relevant stages to establish deterministic identity.
- [ ] Pass actual R4Engine/model-root admission and an offline startup/load/prediction canary.
- [ ] Measure teacher floor, TLA/plain, R4G1, and production serving on the same held-out partitions with EXCT-on, EXCT-miss, and EXCT-disabled slices.
- [ ] Measure teacher-forced, free-running, causal prompt, and D4/OOD baseline status separately.
- [ ] Append the resource/measurement record and re-ratify every M.V.G. gate.

## Non-goals

- Tune downstream prompt-state or abstention mechanisms during baseline compilation.
- Promote incomplete recorded-corpus remnants as canonical.
- Interpret same-corpus replay as document-disjoint generalization.

## Dependencies and entry gate

Items B–D must at least freeze status semantics, serving attribution, identities, slices, and report schema before the expensive run is launched.

## Acceptance criteria

- [ ] A complete bundle is reconstructable from pinned inputs and all expected artifacts/CIDs are present.
- [ ] Two deterministic builds/checkpoint replays reproduce the claimed byte identities within the declared platform scope.
- [ ] The empty-root offline production admission and canary pass.
- [ ] Baseline reports include uncertainty, path attribution, all declared slices/controls, and explicit PASS/FAIL/UNAVAILABLE status.
- [ ] README/ROADMAP/RESEARCH and the M.V.G. record contain one reconciled current baseline.

## Verification

- [ ] Run bundle/source/tokenizer validators and a preflight dry run.
- [ ] Run evaluate-report and actual production admission from an empty model root.
- [ ] Run deterministic rebuild/hash comparison and corrupt/missing-component negatives.
- [ ] Run the workspace gates plus wasm check for any code changed to complete the chain.
- [ ] Independently replay a sample of report rows from artifact evidence.

## Run contract

- **Metric/current value:** Re-establish same-slice teacher floor, TLA/plain, R4G1, and deployed-serving quality; current historical anchors include 10.2%/29.0% thin, 24.3% canonical, best live 31.48%, and teacher floor ≈3.6015 bits, pending exact manifest recovery.
- **Reachability ceiling:** Compute per proposed component from the fresh preflight path mix. Cover-only headline movement is bounded by the fraction that can reach the cover; do not reuse the #460 EXCT-dominated slice without recomputing.
- **Pinned identities:** Teacher/source, tokenizer/adapter, corpus and held-out partition, compiler/configuration, base revision, decoder/seeds, hardware, bundle, and report CIDs.
- **Nulls/falsifier:** TLA/plain baseline, old admitted bundle if valid, EXCT-disabled/miss slices, trivial/root prior, shuffled attribution, corrupt/incomplete bundle.
- **Binding cheap instrument:** Source/tokenizer/partition validators, fixed small-shard compile, scorer/reporter round trip, disk/RSS projection, and production admission preflight must all pass.
- **Exit rule:** A source-complete deterministic bundle passes deployed admission and produces a non-vacuous comparable report; quality gates are ratified from predeclared intervals rather than retrofitted.
- **If positive:** Adopt it as the canonical S0 baseline and unlock S1/S2 experiments.
- **If negative:** Record the failing stage and classify the current product claim as unavailable at that scope; repair provenance/admission before downstream experiments.
- **Cost estimate:** Post measured wall clock, peak RSS/storage, teacher compute, checkpoint footprint, and which shared resources are blocked before launch.

The long run does not launch until this contract is posted with exact values and the cheap instrument passes. Missing observations serialize as `UNAVAILABLE`/`null`, never zero or empirical PASS. The control arm must be shown non-degenerate before comparison.

## Compatibility and migration

This may define a new canonical artifact era. Preserve readers for supported historical bundles or document a deliberate recompile boundary; never silently reinterpret old bytes under new semantics.

## Repository conformance

- **Execution scope:** Offline compiler/certifier plus actual deployed R4Engine admission and prediction path. Evidence outside this scope is not credited as a deployed-serving result.
- Preserve the applicable P-4, zero-allocation steady state, deterministic-byte, bounded-work, focused typed-error, `no_std`, and `forbid(unsafe_code)` contracts.
- Classify normative mathematics under `docs/formal_vocabulary.md`; empirical criteria name distribution, protocol, sample count, uncertainty, provenance identities, and status.
- **Conformance mapping:** RF-09, RF-15, RF-21, RF-22, RF-23, RF-29.. Reuse/extend an existing capability unless a genuinely new capability requires a new RF ID.
- A new built capability lands in order: `model/ids.toml` row → tagged Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md`.
- `CONFORMANCE.md` is generated and must not be edited directly. Retained off-serving mechanisms stay `open` in `model/ledger.toml` with an activation gate.
- Measurements that revise a claim append a per-issue record and reconcile README, ROADMAP, RESEARCH, lifecycle/configuration, and ledger assertions.


## Claim status and next action

The outcome is a baseline/admission verdict, not evidence of general intelligence. Negative results remain published and block downstream promotion.

## Comments (3)

### comment 5352207697 — 2026-08-20T06:22:20Z — Casey-allard

### S0-E preflight, run contract, and HELD verdict (before the flagship run)

Per the `AGENTS.md` long-run discipline, the run contract and the binding
cheap-instrument verdict, posted before any hours-scale launch. Full record in
PR #864 (`docs/attested_broad_baseline_833.md`).

**Reachability arithmetic.** A #755-native rebuild can differ from the pinned #516
corpus only across the ~0.15% of records the pre-#755 pipeline mis-ordered or
duplicated (213 dup + 6 non-contiguous stories / 2,994) → headline top-1 movement
bounded to **< ~0.5 pp** → the rebuild is expected to **RETAIN** the 24.30% /
31.48% / 3.6015-bit pin. Its decision value is attestation / provenance / corpus
integrity / determinism, **not** a quality delta.

**Binding cheap instrument — measured (`origin/main@70586aa0`, offline):**

| Instrument | Result | Status |
|---|---|---|
| Teacher `model.safetensors` bytes (723,674,912 B, matches #516) | present | PASS |
| Pinned source revision `2366112999…` fetchable | 404 upstream | UNAVAILABLE |
| Deployed admission canary (`r4 ask --model smollm2-360m-broad`) | loads + predicts offline ~1.5 s | PASS (load/predict) |
| Instruction-quality attestation on the pin | "without an instruction-quality attestation" | FAIL (the #833 gap) |
| Post-#755 corpus guard (`subsample-recorded-corpus`) | "record 360204 declares story 2994, outside range 0..2994" | FAIL |
| Byte reproducibility, 2 fresh `compile-recorded` runs | `tless_artifacts.bin` (κ `blake3:0b85c43d…`, the attested signature), `tless_store.bin`, `hamming_calibration.json` byte-identical | PASS |
| Byte reproducibility, `hierarchical_codes.json` | differs across recompiles | FAIL (non-deterministic) |

**Verdict — HELD (do not launch).** The cheap instrument fails three sub-gates and a
required identity is UNAVAILABLE, so the discipline forbids launching. Preconditions,
with owners:

1. **Determinism defect** (blocks "two deterministic builds reproduce byte
   identities"): `hierarchical_codes.json` is non-deterministic across recompiles.
   *Next:* fix the compile-recorded hierarchical-codes emission (a code fix, outside
   the #833 baseline scope — candidate follow-up issue). *Owner:* unassigned.
2. **Source-provenance re-pin** (blocks "reconstructable from pinned inputs"): the
   pinned revision 404s; local weights are byte-identical to fetchable rev
   `a10cc1512…`. *Next:* maintainer re-pin of `models/smollm2-360m-instruct.json`
   (+ descriptor κ), or record the local snapshot as the source-of-record. *Owner:*
   maintainer.
3. **Corpus integrity + resource authority**: the #516 corpus fails the post-#755
   guard, so the rebuild must **re-observe** (not recompile the old corpus). *Next:*
   after (1)+(2), launch the dense #755-native re-observe → compile → cover → score
   → evaluate → package → admit, twice for determinism, under the maintainer-approved
   measurement-only Accelerate teacher path and a posted resource-block window.
   *Owner:* maintainer (resource/authority).

No M.V.G. threshold changed; the #516 pin is retained pending a clean run. I am
unassigning myself: this track parks on the maintainer decisions above, not on active
work. When (1) and (2) clear and the resource window is authorized, the run contract
above is ready to execute.


### comment 5353716637 — 2026-08-20T09:00:29Z — Casey-allard

### Definition of Done — S0 item E complete (RATIFY / RETAIN)

Merged to `main` @ `1b3e46f2` (PR #867), building on the #866 preconditions
(determinism fix #865 + source re-pin). Executed with **no re-observe** — the raw
teacher observation (`.uor-models/obs-wiki-360m`, 256 shards / 361,693 records) was
reused; only a deterministic story-id densification in `scripts/obs_bundle_to_corpus.py`
was needed (the #516 corpus dropped 6 stories, leaving gapped ids the post-#755 guard
rejects).

**Attested bundle:** `.uor-models/compiled/smollm2-360m-broad-clean` — clean corpus
(360,924 records / 2,994 stories, passes `subsample-recorded-corpus`) → compile-recorded
(byte-reproducible across two builds) → cover → score → `release-bundle.json` (graph
`blake3:bc2366f1…`, signature `blake3:6324aabe…`) → deployed R4Engine admission +
offline prediction canary PASS.

**Gate C — M.V.G. RATIFIED (RETAIN), held-out 72,130:**

| row | #516 pin | clean #755-native |
|---|---|---|
| Rule 1+2 | 24.30% | 24.39% |
| best-live | 31.48% | 31.11% |
| TLA-3 | 28.21% | 28.12% |

All headline causal rows within the predeclared <~0.5 pp reachability bound; witness
replay 64/64. Teacher floor RETAINED at 3.6015 bits by invariance (unchanged weights +
exact GEMM); a full re-measurement is UNAVAILABLE at scale under the current uor-matmul
exact-GEMM teacher (~1 pos/s → ~24 h; the #516 run used the since-removed Apple-Accelerate
path) — documented, not asserted as a fresh PASS.

Record: `docs/attested_broad_baseline_833.md`. No `ids.toml`/`CONFORMANCE` change; the
baseline is RETAINED so README/ROADMAP/RESEARCH numbers are unchanged.

**S0 (#821):** all five children (#829–#833) are now closed; the stage tracker can take
its promotion/closure verdict.

Two recommended follow-ups (out of scope here): (1) a faster teacher forward path or a
persisted floor cache so the full-population teacher floor is re-measurable; (2) `cargo
deny` `advisories` is red on `main` from a yanked `arrayref` (whole 0.3.x line yanked
upstream; pulled by `blake3` via `uor-addr`) — needs a `deny.toml` yanked-policy decision
or a `blake3`/`uor-addr` bump. Both are non-blocking (register-conformance is non-required).


### comment 5406302567 — 2026-08-25T06:11:45Z — Casey-allard

## Append-only scope correction — #933 final outcome (2026-08-25)

The #833 source, tokenizer, corpus, #755-native ordering, deterministic-build, and Gate C measurements remain valid in their recorded scope. The earlier wording that the #833 bundle passed “actual production admission” or established normative deployed-serving quality is superseded: that canary used the explicit research/quality-bypass `R4Engine` path, and the historical graph did not establish same-generation SKMX/PSIB consumption by the ADR-designated `R4G1Runtime` selector.

#933 resolved the missing boundary with a **distinct** schema-2 bundle in PR #936, merged through the protected queue as `e346816cb40b089583cbd3fff2cdd84924362c65` after the binding merge-group run passed. Its full normative census is 21,293 / 72,130 (29.5203%) versus same-position TLA 20,284 / 72,130 (28.1214%), paired +13.988 permille [11.057, 16.919]. The same-generation sections-absent runtime is 18,806 / 72,130 (26.0723%), for a bounded-lane effect of +34.479 permille [31.681, 37.277].

The new graph CID is `blake3:ff82dfd5f04eac7e944443b1ea4cc9fe93a007b3b8f07286876d52709a98bc49`; its full report CID is `blake3:88ee8210e1f4c48dc26999f5685350b2d2343676cdbd6f9b1aee7c7f1c66146f`. After verifier hardening, that exact envelope admitted through the ordinary production loader from an empty model store. This does not retrofit the new graph, sections, or admission result onto #833.

Record: https://github.com/UOR-Foundation/uor-r4/blob/main/docs/normative_r4g1_quality_933.md
PR: https://github.com/UOR-Foundation/uor-r4/pull/936
Binding queue run: https://github.com/UOR-Foundation/uor-r4/actions/runs/32814445068

The RATIFY verdict is limited to the bound artifact/population/selector/greedy-decode envelope. It is not a universal 30% floor or evidence of live-teacher parity, instruction following, reasoning, factuality, semantic abstention, or free-running coherence.


---
_Comment/review ids in this file: 5352207697, 5353716637, 5406302567_

