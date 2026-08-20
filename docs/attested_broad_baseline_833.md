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
