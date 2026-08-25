# Attested broad-bundle baseline — issue #833 preflight, run contract, and hold verdict

- **Status:** S0 item E (#833) preflight. The expensive rebuild is **HELD, not
  launched** — this record is the run contract and the binding-cheap-instrument
  verdict that `AGENTS.md` (long-run discipline) requires before any hours-scale
  run. Append-only: the rebuild outcome will be appended here once its
  preconditions clear.
- **Execution scope:** offline compiler/certifier plus the deployed R4Engine
  admission and prediction path. Evidence outside this scope is not credited as a
  deployed-serving result.
- **Conformance mapping:** RF-09, RF-15, RF-21, RF-22, RF-23, RF-29 (binds existing
  capabilities; no new capability, no `model/ids.toml` row, no `CONFORMANCE.md`
  change).
- **Pinned subject:** the current broad-canonical baseline is the #516 pin at
  `.uor-models/compiled/smollm2-360m-broad` (SmolLM2-360M-Instruct observed on
  Simple-Wiki `20231101.simple` rows 0..2999, 360,924 records / 2,994 stories).
  Recorded anchors (`docs/smollm2_teacher_baseline_320.md`, #516): Rule 1+2 24.30%
  top-1 / 11.94 bits, best live 31.48% / 10.43 bits, TLA-3 28.21%, teacher floor
  3.6015 bits, EXCT-miss 25.7%, 72,864 held-out D3 positions.

## Why a rebuild is in scope (the #755 boundary)

The #516 corpus was compiled 2026-08-09, **before** the #755 fix (merged
2026-08-17, `05f4067c`, "reconstruct corpus context by (story, span_start), not
disk order"). Its own record documents the pre-#755 sharded-observation ordering
artifact it was hand-cleaned around — 213 duplicate `(story, position)` records
plus 6 stories left non-contiguous by a concurrent-loop overlap. #833 asks for a
source-complete, tokenizer-attested, #755-native, deterministic rebuild; deployed
admission; re-measured frozen slices; and an explicit retain / revise / retire
verdict on the M.V.G. thresholds.

## Reachability arithmetic (the five-minute gate, before any hours-scale run)

**Empirical Criterion (bounded).** A #755-native rebuild can differ from the pinned
#516 corpus only across the records the pre-#755 pipeline mis-ordered or
duplicated: 213 duplicate records plus the positions in 6 stories, out of 360,924
records / 2,994 stories — about **0.15% of records**. Even under the pessimistic
assumption that every affected held-out position flips its top-1 outcome, headline
top-1 movement is bounded to **well under ~0.5 pp**. The rebuild is therefore
expected to **RETAIN** the pinned 24.30% / 31.48% / 3.6015-bit anchors; its
decision value is **attestation, provenance, corpus integrity, and byte
reproducibility**, not a quality delta. The positive branch adopts the attested
#755-native bundle as the canonical S0 baseline and unlocks S1/S2; the negative
branch records a named failing stage and classifies the product claim UNAVAILABLE
at that scope until repaired. The branches differ, so the run has decision value —
once its preconditions pass.

## Run contract (`AGENTS.md` long-run discipline)

```text
metric to move:       same-slice teacher floor / TLA-plain / R4G1 / deployed-serving
                      top-1; current pin 24.30% (Rule 1+2), 31.48% (best live),
                      teacher floor 3.6015 bits
reachability ceiling: < ~0.5 pp headline movement (<= 0.15% of records differ
                      post-#755; arithmetic above)
instrument + verdict: source/tokenizer validators + post-#755 corpus-integrity guard
                      + deterministic compile-recorded + deployed admission canary —
                      must ALL pass (measured table below)
exit rule:            a source-complete + #755-clean + byte-reproducible bundle passes
                      deployed admission and produces a non-vacuous comparable report;
                      M.V.G. gates ratified from predeclared intervals, not retrofitted
if positive:          adopt the attested #755-native bundle as the canonical S0
                      baseline; unlock S1/S2
if negative:          record the failing stage; classify the product claim UNAVAILABLE
                      at that scope; repair provenance / integrity / determinism before
                      any downstream experiment
cost estimate:        dense Simple-Wiki 0..2999 seq-len-128 re-observe via the 360M
                      Accelerate teacher (the #516 observe ran overnight; a traced
                      full/1 observe measured ~5.75h, #804) + cover + score
                      (score.r4g1 ~29 MB) + evaluate-report replaying 72,864 held-out
                      positions through the teacher + a second deterministic build;
                      multi-hour-to-multi-day, blocks the host CPU and tens of GB of
                      storage
```

## Binding cheap instrument — measured verdicts (2026-08-20)

Binary `origin/main@70586aa0`, `aarch64-apple-darwin`, cargo 1.97.1 (rust-toolchain
pin). All runs offline.

| Instrument | Observed result | Status |
|---|---|---|
| Teacher source bytes present (`model.safetensors`, 723,674,912 B, matches #516) | present, byte-complete | PASS |
| Pinned source **revision** fetchable (`2366112999…`, `models/smollm2-360m-instruct.json`) | 404 on Hugging Face (upstream rewrote; `docs/smollm2_teacher_baseline_320.md`) | UNAVAILABLE |
| Deployed admission canary — R4Engine load + offline prediction on the #516 pin (`r4 ask --model smollm2-360m-broad --greedy`) | loads and predicts offline in ~1.5 s | PASS (load/predict) |
| Instruction-quality attestation on the #516 pin | warns "using a locally compiled bundle without an instruction-quality attestation" | FAIL (the #833 gap) |
| Post-#755 corpus-integrity guard (`subsample-recorded-corpus`) on the #516 corpus | rejected: "record 360204 declares story 2994, outside the metadata story range 0..2994" | FAIL |
| Deterministic rebuild — byte reproducibility across two fresh `compile-recorded` runs and vs the Aug-9 pin | `tless_artifacts.bin` (κ `blake3:0b85c43d…` = the attested `signature_artifact`), `tless_store.bin`, `hamming_calibration.json` byte-identical | PASS |
| Deterministic rebuild — `hierarchical_codes.json` | differs between the two fresh recompiles (fresh1 ≠ fresh2) and vs the pin | FAIL (non-deterministic) |

**Empirical Criterion.** N = 2 fresh `compile-recorded` runs on the pinned corpus,
same binary and host, reproduce `tless_artifacts.bin` / `tless_store.bin` /
`hamming_calibration.json` byte-for-byte (byte reproducibility; status
**Empirical**), and produce **differing** `hierarchical_codes.json` bytes (measured
non-determinism; status **Empirical**). `compile-recorded` is era-tolerant and still
consumes the #516 corpus, so the compiled artifacts continue to serve; the strict
`subsample-recorded-corpus` derivation guard is what surfaces the dangling story id.

Free-running greedy decode on the #516 pin produces a degenerate digit sequence;
this is the recorded #784/#811 substrate limitation (continuation-distribution
convergence; no established free-running coherence), not a new result.

## Verdict — HELD (do not launch)

Per `AGENTS.md` long-run discipline ("do not launch the run if the cheap instrument
fails … or required fixtures or identities are unavailable"), the flagship rebuild
is **HELD**. Three preconditions fail the binding cheap instrument and must clear
first:

1. **Determinism defect** — blocks the acceptance criterion "two deterministic
   builds reproduce the claimed byte identities". `hierarchical_codes.json` is
   non-deterministic across recompiles. *Next action:* fix the non-determinism in
   the compile-recorded hierarchical-codes emission (a code fix, outside the #833
   baseline scope — candidate follow-up issue). *Owner:* unassigned.
2. **Source-provenance re-pin** — blocks "source-complete, reconstructable from
   pinned inputs". The pinned teacher revision 404s upstream; the local weights are
   byte-identical to a fetchable release (rev `a10cc1512…`,
   `docs/smollm2_teacher_baseline_320.md`). *Next action:* maintainer re-pin
   decision — update `models/smollm2-360m-instruct.json` (and its descriptor κ) to
   a currently-fetchable revision, or record the byte-identical local snapshot as
   the pinned source-of-record. *Owner:* maintainer.
3. **Corpus integrity + resource authority** — the #516 corpus fails the post-#755
   guard (dangling story id), so the rebuild must **re-observe**, not recompile the
   old corpus. *Next action:* after (1) and (2), launch the dense #755-native
   re-observe → compile → cover → score → evaluate → package → admit, twice for
   determinism, under the maintainer-approved measurement-only Accelerate teacher
   path and a posted resource-block window. *Owner:* maintainer (resource /
   authority).

No M.V.G. threshold is changed by this preflight: the reachability arithmetic
RETAINS the #516 pin pending a clean run. No `model/ids.toml` / `CONFORMANCE.md`
change.

## Provenance of this preflight

- Binary: `origin/main@70586aa0` (`cargo build --release --bin r4`), host
  `aarch64-apple-darwin`.
- Pinned bundle: `.uor-models/compiled/smollm2-360m-broad`; `tless_artifacts.bin` κ
  `blake3:0b85c43dc6f0f4683d26979d…` (the attested `signature_artifact` in its
  `release-bundle.json`).
- `corpus.meta`: 360,924 records / 2,994 stories / complete flag set (`USPC2SBO`
  tag), 88-byte records.
- Fresh recompiles: `/tmp/cr833`, `/tmp/cr833b` (scratch, teacher-free
  `compile-recorded`, removed after measurement).

## Claim status

This record is a **preflight and a hold verdict**, not evidence of a new baseline or
of general capability. It changes no promoted claim. The canonical broad-text
baseline remains the #516 pin (retained) until a source-complete, #755-native,
byte-reproducible, admitted rebuild replaces it and re-ratifies the M.V.G. gates.

## Update 2026-08-20 — preconditions (1) and (2) resolved; (3) reduced

Two of the three HELD preconditions are now cleared, and the third is materially
reduced.

1. **Determinism — FIXED (#865).** The `hierarchical_codes.json` non-determinism was
   a tie-break in `induce_hierarchical_codes`
   (`crates/uor-r4-core/src/transformerless/compiler.rs`): `frequent_paths` was
   sorted by transition count alone, so tied paths kept `HashMap` iteration order and
   `take(100)` selected a run-dependent subset. Fixed with a total tie-break (count
   descending, then path bytes ascending); regression test
   `hierarchical_codes_determinism_865`. Two fresh `compile-recorded` runs now produce
   a byte-identical `hierarchical_codes.json` (verified).

2. **Source provenance — RE-PINNED.** `models/smollm2-360m-instruct.json` now records
   the local snapshot as the content-addressed source-of-record: `source_kappa`
   `blake3:eb23c3e8…` (loader-emitted, scope `model.safetensors`), `source_bytes`
   `723674912`, and `revision` `a10cc1512eabd3dde888204e902eca88bddb4951` (the
   byte-identical, previously-fetchable release the local weights match; the earlier
   pinned `2366112999…` was rewritten upstream). Reconstruction no longer depends on
   the dead HF revision — the pinned κ verifies against the local bytes.

3. **Re-observe — largely AVOIDABLE (the ~6 h teacher pass is reusable).** The raw
   teacher observation is already on disk: `.uor-models/obs-wiki-360m/` — 256
   content-addressed shards, 361,693 records, `input_cid` `blake3:194db0ee…`, plus a
   merged `merged.bin` (31,761,312 B = the #516 corpus size). A #755-clean rebuild
   re-merges/finalizes these shards under the post-#755 corpus reconstruction (by
   `(story, span_start)`) rather than re-running the teacher. Remaining chain — all but
   one stage teacher-free: re-merge shards → `compile-recorded` (~2 min measured) →
   cover → score → `evaluate-report` (the one teacher-bound stage: teacher-forced
   replay of the 72,864 held-out positions) → package → admit, plus a second
   deterministic build for byte reproducibility. Estimated wall clock **~2–3 h** (vs.
   ~8–10 h with a fresh observe), dominated by `score` plus the single
   `evaluate-report` teacher replay. One validation remains before launch: confirm the
   post-#755 re-merge of the existing shards yields a corpus that passes the strict
   `subsample-recorded-corpus` guard (no dangling story id); if the defect is intrinsic
   to the raw shards, only the affected stories re-observe.

**Remaining gate for #833:** a ~2–3 h compute window (maintainer resource
authorization) for the teacher-forced `evaluate-report` replay under the
measurement-only Accelerate exception. Determinism and provenance are resolved.

## Update 2026-08-20 (rebuild executed) — clean #755-native bundle built; baseline RATIFIED

The clean rebuild ran with **no re-observe** — the raw teacher observation was reused.

**Corpus derivation.** `scripts/obs_bundle_to_corpus.py` was fixed to densify story
ids to a contiguous `0..stories-1` range. The #516 corpus dropped 6 whole stories
(769, 771–775), leaving gapped ids that the `subsample-recorded-corpus` guard rejects;
the densify is a deterministic, token-content-preserving relabeling (identity for an
already-dense bundle). Applied to `.uor-models/obs-wiki-360m` (256 shards, 361,693 raw
records → 360,924 merged), the clean corpus is 360,924 records / 2,994 stories
(corpus.records κ `blake3:4692307368…`, corpus.meta κ `blake3:aa9d176779…`) and PASSES
the strict `subsample-recorded-corpus` guard.

**Attested bundle** — `.uor-models/compiled/smollm2-360m-broad-clean` (not the #516
pin). compile-recorded (post-#865, byte-reproducible) → cover → score (`relative_tla`).
The attention operator was canonicalized to the current `standard-source-attention/2`
(numerically bit-identical to the `/1` the #516 corpus recorded, per `attention.rs`)
via `copy-recorded-attention`, so the bundle is evaluable against the current teacher.
`package-release-bundle` produced `release-bundle.json` (graph `blake3:bc2366f1…`,
signature `blake3:6324aabe…`, tokenizer_adapter hf-byte-bpe/1). Deployed R4Engine
admission + offline prediction canary PASS (~1.5 s). Two independent
`compile-recorded` builds of the clean corpus reproduce `tless_artifacts.bin`,
`tless_store.bin`, `hamming_calibration.json`, and `hierarchical_codes.json`
byte-for-byte (byte reproducibility; the #865 fix makes `hierarchical_codes.json`
deterministic).

**Gate C — M.V.G. RATIFIED (RETAIN).** Held-out 72,130 positions (vs 72,864 — the ~6
boundary stories the id densification moved into train). **Empirical Criterion**,
status **Empirical**:

| row (causal serving) | #516 pin | clean #755-native | Δ |
|---|---|---|---|
| Rule 1+2 precedence | 24.30% | 24.39% | +0.09 pp |
| best live arm (`on_fwd_strict_live`) | 31.48% | 31.11% | −0.37 pp |
| TLA-3 baseline | 28.21% | 28.12% | −0.09 pp |
| EXCT-miss (strict) | 25.7% | 27.36% | +1.66 pp (split shift) |

All three headline causal rows reproduce within the predeclared reachability bound
(<~0.5 pp); the pin is **RETAINED**. EXCT-miss rises 1.66 pp because the 6 well-covered
boundary stories moved from held-out to train — a coverage-structure effect of the id
densification, not a quality regression. Witness replay 64/64; candidate recall
Rule 1+2 62.6% / 87.3%.

**Teacher floor — RETAINED at 3.6015 bits (re-measurement UNAVAILABLE at scale).**
The `evaluate-report` binding chain was validated end-to-end on this bundle (source
load, tokenizer.bin/adapter, scored-graph tokenizer_cid, and
`standard-source-attention/2` operator all accepted; the teacher-forced held-out
replay then begins). The floor is a property of (teacher weights, corpus): the weights
are unchanged (source κ `blake3:eb23c3e8…`) and the teacher's forward pass is exact
GEMM (numerically canonical, bit-identical to the corpus's recorded `/1`), so the floor
is invariant to the matmul backend and to the story-id densification (which relabels
boundaries only). A full-population re-measurement is **UNAVAILABLE at scale under the
current teacher**: the deployed teacher path is now uor-matmul exact GEMM (the #516
measurement used the since-removed Apple-Accelerate path), which steps at ~1 position/s,
so replaying the 72,130 held-out positions would take ~24 h. The floor is therefore
RETAINED at 3.6015 bits by invariance rather than re-asserted as a fresh PASS; a
faster teacher forward path (or a persisted floor cache) is the precondition for a
full re-measurement and is a recommended follow-up. The teacher-forced table-native
top-1 / argmax-agreement rows (#516: 17.3% / 25.4% at 14.6954 bits, +11.09 over floor)
are teacher-bound in the same way and are likewise RETAINED; the causal serving rows
above (measured teacher-free against `corpus.next`) are the primary quality evidence
and were freshly measured.

**Verdict.** RATIFY (RETAIN) the #516 broad-text baseline on a source-complete,
#755-native, byte-reproducible, admitted, attested bundle. Determinism, provenance,
corpus integrity, and attestation are resolved; the bundle lives at
`.uor-models/compiled/smollm2-360m-broad-clean` (regenerable from the pinned inputs via
the fixed converter + the recorded-corpus compile chain).

## Append-only scope correction — 2026-08-24 (#933)

The source snapshot, clean-corpus identities, #755 ordering, deterministic
compiler outputs, package attestation, and teacher-floor invariance recorded
above remain valid. The production-quality attribution does not.

The cited `r4 ask` canary loaded with `R4Engine::load_accepting_quality`, an
explicit research/quality-bypass loader, rather than proving strict production
admission. Gate C's Rule 1+2, best-live, and TLA rows are certifier/offline
measurements; they do not demonstrate the token selected by ADR-0001's sole
normative `R4G1Runtime` serving path. The attested graph also predates #910 and
carries no active SKMX/PSIB serving sections. Consequently the 24.39% Rule 1+2
and 28.12% TLA rows remain valid historical comparator evidence, while normative
served-token quality and RF-31 activation on this bundle are **NOT ESTABLISHED**.

#933 retains this bundle's pinned inputs, re-emits its graph through the
production path, binds the exact deployed-quality report and release manifest,
and performs only teacher-free reachability/sample/conditional-census evidence.
Until that outcome is recorded, this record establishes provenance and
reproducibility, not strict production admission or normative serving quality.

## Append-only #933 outcome — 2026-08-25

The source, tokenizer, corpus, and #755-native ordering attested here supplied
the pinned inputs to #933. The new sections-present graph and release package
are distinct derived artifacts; they do not retroactively add SKMX/PSIB to the
historical #833 graph or convert its bypass-loader canary into strict admission.

For the new canonical schema-2 bundle, the `R4G1Runtime` greedy-decode census is
**21,293 / 72,130 (29.5203%)**, compared with same-position TLA **20,284 /
72,130 (28.1214%)**, paired **+13.988 permille [11.057, 16.919]**. The
same-generation sections-absent control is **18,806 / 72,130 (26.0723%)**, so
the lane effect is **+34.479 permille [31.681, 37.277]**. The exact graph CID is
`ff82dfd5f04eac7e944443b1ea4cc9fe93a007b3b8f07286876d52709a98bc49`
and the full deployed-quality report CID is
`88ee8210e1f4c48dc26999f5685350b2d2343676cdbd6f9b1aee7c7f1c66146f`.

The hardened release manifest has raw BLAKE3
`c2025e9e507e8367993d78bd83ef099ce5851c838d3cc5cf01eda5560986ad33`
and SHA-256
`7572e07a1e3722f3ffc0ea749a67b4ac162221de79b5b4b8a315f4e4e6570fde`;
it binds comparator-store CID
`c1749e62077758c4a098e2a02150b5455e1ca3c02c60b87e6d45fcbb9e2b4404`.
Strict production admission passed from an empty model store after hardening at
`f901cd97577da3117fd52c9b1c6dcf075cc4d3a2`.

The resulting **RATIFY** verdict is limited to that exact bundle, population,
selector, greedy decode, and schema-2 envelope. It establishes neither a
universal 30% floor nor instruction following, reasoning, factuality, semantic
abstention, free-running coherence, or live-teacher parity. The repository BDD
suite passed 124 / 124, but its live-teacher parity fixtures were absent and the
fixture-gated scenarios vacuously skipped.
